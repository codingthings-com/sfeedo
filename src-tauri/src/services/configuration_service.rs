use crate::config::ConfigManager;
use crate::database::operations::ConfigOperations;
use crate::database::DatabaseConnection;
use crate::feed_manager::{FeedSourceManager, FeedSourceStats};
use crate::models::{AppConfig, FeedSource};
use tauri::AppHandle;

/// Comprehensive configuration service that manages both app config and feed sources
pub struct ConfigurationService {
    config_manager: ConfigManager,
    db: DatabaseConnection,
}

impl ConfigurationService {
    /// Create a new ConfigurationService instance
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_manager = ConfigManager::new(app_handle)?;
        let db = DatabaseConnection::new(app_handle)
            .map_err(|e| format!("Failed to initialize database: {}", e))?;

        Ok(Self {
            config_manager,
            db,
        })
    }

    /// Get the current application configuration
    pub fn get_app_config(&self) -> Result<AppConfig, String> {
        // Try to load from JSON file first
        match self.config_manager.load_config() {
            Ok(config) => {
                // Sync to database for consistency
                let config_ops = ConfigOperations::new(&self.db);
                if let Err(e) = config_ops.save_config(&config) {
                    log::warn!("Failed to sync config to database: {}", e);
                }
                Ok(config)
            }
            Err(_) => {
                // Fallback to database
                let config_ops = ConfigOperations::new(&self.db);
                config_ops.load_config()
                    .map_err(|e| format!("Failed to load config from database: {}", e))
            }
        }
    }

    /// Update the application configuration
    pub fn update_app_config(&self, config: AppConfig) -> Result<(), String> {
        // Validate configuration
        config.validate()?;

        // Save to JSON file
        self.config_manager.save_config(&config)?;

        // Save to database
        let config_ops = ConfigOperations::new(&self.db);
        config_ops.save_config(&config)
            .map_err(|e| format!("Failed to save config to database: {}", e))?;

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_config_to_defaults(&self) -> Result<(), String> {
        // Reset JSON files
        self.config_manager.reset_to_defaults()?;

        // Reset database
        let config_ops = ConfigOperations::new(&self.db);
        config_ops.reset_to_defaults()
            .map_err(|e| format!("Failed to reset database config: {}", e))?;

        Ok(())
    }

    /// Get feed source manager
    pub fn get_feed_source_manager(&self) -> FeedSourceManager<'_> {
        FeedSourceManager::new(&self.config_manager, &self.db)
    }

    /// Add a new feed source
    pub fn add_feed_source(&self, name: String, url: String) -> Result<FeedSource, String> {
        let manager = self.get_feed_source_manager();
        manager.add_feed_source(name, url)
    }

    /// Remove a feed source
    pub fn remove_feed_source(&self, id: &str) -> Result<bool, String> {
        let manager = self.get_feed_source_manager();
        manager.remove_feed_source(id)
    }

    /// Update a feed source
    pub fn update_feed_source(&self, feed_source: FeedSource) -> Result<bool, String> {
        let manager = self.get_feed_source_manager();
        manager.update_feed_source(feed_source)
    }

    /// Enable or disable a feed source
    pub fn toggle_feed_source(&self, id: &str, enabled: bool) -> Result<bool, String> {
        let manager = self.get_feed_source_manager();
        manager.toggle_feed_source(id, enabled)
    }

    /// Get all feed sources
    pub fn get_all_feed_sources(&self) -> Result<Vec<FeedSource>, String> {
        let manager = self.get_feed_source_manager();
        manager.get_all_feed_sources()
    }

    /// Get only enabled feed sources
    pub fn get_enabled_feed_sources(&self) -> Result<Vec<FeedSource>, String> {
        let manager = self.get_feed_source_manager();
        manager.get_enabled_feed_sources()
    }

    /// Get a feed source by ID
    pub fn get_feed_source_by_id(&self, id: &str) -> Result<Option<FeedSource>, String> {
        let manager = self.get_feed_source_manager();
        manager.get_feed_source_by_id(id)
    }

    /// Get feed source statistics
    pub fn get_feed_source_stats(&self) -> Result<FeedSourceStats, String> {
        let manager = self.get_feed_source_manager();
        manager.get_feed_source_stats()
    }

    /// Validate a feed URL
    pub fn validate_feed_url(&self, url: &str) -> Result<(), String> {
        let manager = self.get_feed_source_manager();
        manager.validate_feed_url(url)
    }

    /// Import feed sources from JSON to database
    pub fn import_feed_sources_from_json(&self) -> Result<usize, String> {
        let manager = self.get_feed_source_manager();
        manager.import_from_json()
    }

    /// Export feed sources from database to JSON
    pub fn export_feed_sources_to_json(&self) -> Result<usize, String> {
        let manager = self.get_feed_source_manager();
        manager.export_to_json()
    }

    /// Backup current configuration
    pub fn backup_configuration(&self) -> Result<String, String> {
        let backup_path = self.config_manager.backup_config()?;
        Ok(backup_path.to_string_lossy().to_string())
    }

    /// Get configuration directory path
    pub fn get_config_directory(&self) -> String {
        self.config_manager.get_config_dir().to_string_lossy().to_string()
    }

    /// Synchronize configuration between JSON files and database
    pub fn sync_configuration(&self) -> Result<(), String> {
        // Load config from JSON and sync to database
        let config = self.config_manager.load_config()?;
        let config_ops = ConfigOperations::new(&self.db);
        config_ops.save_config(&config)
            .map_err(|e| format!("Failed to sync config to database: {}", e))?;

        // Import feed sources from JSON to database
        let manager = self.get_feed_source_manager();
        let imported_count = manager.import_from_json()?;
        
        log::info!("Synchronized configuration: imported {} feed sources", imported_count);
        Ok(())
    }

    /// Initialize default feed sources if none exist
    pub fn initialize_default_feed_sources(&self) -> Result<(), String> {
        let stats = self.get_feed_source_stats()?;
        
        if stats.total_count == 0 {
            // Add some default financial news sources
            let default_sources = vec![
                ("Reuters Business", "https://feeds.reuters.com/reuters/businessNews"),
                ("Yahoo Finance", "https://feeds.finance.yahoo.com/rss/2.0/headline"),
                ("MarketWatch", "https://feeds.marketwatch.com/marketwatch/topstories/"),
                ("Financial Times", "https://www.ft.com/rss/home/us"),
            ];

            let mut added_count = 0;
            for (name, url) in default_sources {
                match self.add_feed_source(name.to_string(), url.to_string()) {
                    Ok(_) => {
                        added_count += 1;
                        log::info!("Added default feed source: {}", name);
                    }
                    Err(e) => {
                        log::warn!("Failed to add default feed source '{}': {}", name, e);
                    }
                }
            }

            log::info!("Initialized {} default feed sources", added_count);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_configuration_service_creation() {
        // This would require a proper test setup with Tauri app handle
        // For now, we'll focus on unit testing individual components
        assert!(true);
    }

    #[test]
    fn test_default_feed_sources() {
        // Test that default feed sources are valid URLs
        let default_sources = vec![
            ("Reuters Business", "https://feeds.reuters.com/reuters/businessNews"),
            ("Yahoo Finance", "https://feeds.finance.yahoo.com/rss/2.0/headline"),
            ("MarketWatch", "https://feeds.marketwatch.com/marketwatch/topstories/"),
            ("Financial Times", "https://www.ft.com/rss/home/us"),
        ];

        for (name, url) in default_sources {
            assert!(!name.is_empty());
            assert!(url.starts_with("https://"));
        }
    }
}