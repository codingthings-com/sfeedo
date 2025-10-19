use crate::config::ConfigManager;
use crate::database::operations::FeedSourceOperations;
use crate::database::DatabaseConnection;
use crate::models::FeedSource;

use url::Url;

/// Feed source configuration management service
pub struct FeedSourceManager<'a> {
    config_manager: &'a ConfigManager,
    db_operations: FeedSourceOperations<'a>,
}

impl<'a> FeedSourceManager<'a> {
    /// Create a new FeedSourceManager instance
    pub fn new(config_manager: &'a ConfigManager, db: &'a DatabaseConnection) -> Self {
        Self {
            config_manager,
            db_operations: FeedSourceOperations::new(db),
        }
    }

    /// Add a new feed source
    pub fn add_feed_source(&self, name: String, url: String) -> Result<FeedSource, String> {
        // Validate URL format
        self.validate_feed_url(&url)?;

        // Check if URL already exists in database
        if self.db_operations.exists_by_url(&url).map_err(|e| format!("Database error: {}", e))? {
            return Err("A feed source with this URL already exists".to_string());
        }

        // Generate unique ID
        let id = self.generate_feed_id(&name, &url);

        // Create new feed source
        let feed_source = FeedSource::new(id, name, url);

        // Validate the feed source
        feed_source.validate()?;

        // Insert into database
        self.db_operations.insert(&feed_source)
            .map_err(|e| format!("Failed to save feed source to database: {}", e))?;

        // Sync to JSON file
        self.sync_to_json_file()?;

        Ok(feed_source)
    }

    /// Remove a feed source by ID
    pub fn remove_feed_source(&self, id: &str) -> Result<bool, String> {
        // Check if feed source exists
        let exists = self.db_operations.get_by_id(id)
            .map_err(|e| format!("Database error: {}", e))?
            .is_some();

        if !exists {
            return Ok(false);
        }

        // Delete from database (this also deletes associated articles)
        let deleted = self.db_operations.delete(id)
            .map_err(|e| format!("Failed to delete feed source from database: {}", e))?;

        if deleted {
            // Sync to JSON file
            self.sync_to_json_file()?;
        }

        Ok(deleted)
    }

    /// Update an existing feed source
    pub fn update_feed_source(&self, feed_source: FeedSource) -> Result<bool, String> {
        // Validate the updated feed source
        feed_source.validate()?;

        // If URL changed, check for duplicates
        if let Some(existing) = self.db_operations.get_by_id(&feed_source.id)
            .map_err(|e| format!("Database error: {}", e))? {
            
            if existing.url != feed_source.url {
                if self.db_operations.exists_by_url(&feed_source.url)
                    .map_err(|e| format!("Database error: {}", e))? {
                    return Err("A feed source with this URL already exists".to_string());
                }
                
                // Validate new URL
                self.validate_feed_url(&feed_source.url)?;
            }
        } else {
            return Err("Feed source not found".to_string());
        }

        // Update in database
        let updated = self.db_operations.update(&feed_source)
            .map_err(|e| format!("Failed to update feed source in database: {}", e))?;

        if updated {
            // Sync to JSON file
            self.sync_to_json_file()?;
        }

        Ok(updated)
    }

    /// Enable or disable a feed source
    pub fn toggle_feed_source(&self, id: &str, enabled: bool) -> Result<bool, String> {
        let updated = self.db_operations.set_enabled(id, enabled)
            .map_err(|e| format!("Failed to toggle feed source: {}", e))?;

        if updated {
            // Sync to JSON file
            self.sync_to_json_file()?;
        }

        Ok(updated)
    }

    /// Get all feed sources
    pub fn get_all_feed_sources(&self) -> Result<Vec<FeedSource>, String> {
        self.db_operations.get_all()
            .map_err(|e| format!("Failed to retrieve feed sources: {}", e))
    }

    /// Get only enabled feed sources
    pub fn get_enabled_feed_sources(&self) -> Result<Vec<FeedSource>, String> {
        self.db_operations.get_enabled()
            .map_err(|e| format!("Failed to retrieve enabled feed sources: {}", e))
    }

    /// Get a feed source by ID
    pub fn get_feed_source_by_id(&self, id: &str) -> Result<Option<FeedSource>, String> {
        self.db_operations.get_by_id(id)
            .map_err(|e| format!("Failed to retrieve feed source: {}", e))
    }

    /// Get feed source statistics
    pub fn get_feed_source_stats(&self) -> Result<FeedSourceStats, String> {
        let total_count = self.db_operations.count(false)
            .map_err(|e| format!("Failed to get total count: {}", e))?;
        
        let enabled_count = self.db_operations.count(true)
            .map_err(|e| format!("Failed to get enabled count: {}", e))?;

        Ok(FeedSourceStats {
            total_count,
            enabled_count,
            disabled_count: total_count - enabled_count,
        })
    }

    /// Validate a feed URL
    pub fn validate_feed_url(&self, url: &str) -> Result<(), String> {
        // Basic URL validation
        let parsed_url = Url::parse(url)
            .map_err(|_| "Invalid URL format".to_string())?;

        // Check protocol
        match parsed_url.scheme() {
            "http" | "https" => {},
            _ => return Err("URL must use HTTP or HTTPS protocol".to_string()),
        }

        // Check if host exists
        if parsed_url.host().is_none() {
            return Err("URL must have a valid host".to_string());
        }

        // Additional validation for common feed patterns
        let path = parsed_url.path().to_lowercase();
        let is_likely_feed = path.contains("rss") 
            || path.contains("feed") 
            || path.contains("atom")
            || path.ends_with(".xml")
            || path.ends_with(".rss");

        if !is_likely_feed {
            // This is a warning, not an error - allow but inform user
            log::warn!("URL '{}' doesn't appear to be a typical feed URL", url);
        }

        Ok(())
    }

    /// Import feed sources from JSON file to database
    pub fn import_from_json(&self) -> Result<usize, String> {
        let feed_sources = self.config_manager.load_feed_sources()?;
        let mut imported_count = 0;

        for feed_source in feed_sources {
            // Check if already exists
            if !self.db_operations.exists_by_url(&feed_source.url)
                .map_err(|e| format!("Database error: {}", e))? {
                
                self.db_operations.insert(&feed_source)
                    .map_err(|e| format!("Failed to import feed source '{}': {}", feed_source.name, e))?;
                imported_count += 1;
            }
        }

        Ok(imported_count)
    }

    /// Export feed sources from database to JSON file
    pub fn export_to_json(&self) -> Result<usize, String> {
        let feed_sources = self.get_all_feed_sources()?;
        self.config_manager.save_feed_sources(&feed_sources)?;
        Ok(feed_sources.len())
    }

    /// Sync database feed sources to JSON file
    fn sync_to_json_file(&self) -> Result<(), String> {
        let feed_sources = self.get_all_feed_sources()?;
        self.config_manager.save_feed_sources(&feed_sources)
    }

    /// Generate a unique ID for a feed source
    fn generate_feed_id(&self, name: &str, url: &str) -> String {
        // Create a base ID from name
        let base_id = name
            .to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
            .collect::<String>()
            .replace(' ', "-");

        // If base_id is empty or too short, use URL host
        let base_id = if base_id.len() < 3 {
            if let Ok(parsed_url) = Url::parse(url) {
                if let Some(host) = parsed_url.host_str() {
                    host.replace('.', "-")
                } else {
                    "feed".to_string()
                }
            } else {
                "feed".to_string()
            }
        } else {
            base_id
        };

        // Check if ID already exists and append number if needed
        let mut counter = 1;
        let mut candidate_id = base_id.clone();

        while self.db_operations.get_by_id(&candidate_id).unwrap_or(None).is_some() {
            candidate_id = format!("{}-{}", base_id, counter);
            counter += 1;
        }

        candidate_id
    }
}

/// Statistics about feed sources
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FeedSourceStats {
    pub total_count: u32,
    pub enabled_count: u32,
    pub disabled_count: u32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConnection;
    use tempfile::TempDir;

    fn create_test_setup() -> (FeedSourceManager<'static>, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        
        // This is a simplified test setup - in real tests you'd need proper lifetime management
        // For now, we'll focus on the logic validation
        todo!("Implement proper test setup with lifetime management")
    }

    #[test]
    fn test_validate_feed_url() {
        // Test valid URLs
        let manager = create_test_setup().0;
        
        assert!(manager.validate_feed_url("https://example.com/rss.xml").is_ok());
        assert!(manager.validate_feed_url("http://feeds.example.com/news").is_ok());
        assert!(manager.validate_feed_url("https://example.com/atom.xml").is_ok());
        
        // Test invalid URLs
        assert!(manager.validate_feed_url("not-a-url").is_err());
        assert!(manager.validate_feed_url("ftp://example.com/feed").is_err());
        assert!(manager.validate_feed_url("https://").is_err());
    }

    #[test]
    fn test_generate_feed_id() {
        let manager = create_test_setup().0;
        
        let id1 = manager.generate_feed_id("Reuters Finance", "https://feeds.reuters.com/reuters/businessNews");
        assert_eq!(id1, "reuters-finance");
        
        let id2 = manager.generate_feed_id("BBC News", "https://feeds.bbci.co.uk/news/rss.xml");
        assert_eq!(id2, "bbc-news");
        
        // Test with special characters
        let id3 = manager.generate_feed_id("Tech@News!", "https://example.com/feed");
        assert_eq!(id3, "technews");
    }
}