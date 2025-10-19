use serde::{Deserialize, Serialize};

/// Represents a news feed source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_fetched: Option<String>, // ISO 8601 datetime string
    pub created_at: String,           // ISO 8601 datetime string
}

impl FeedSource {
    /// Create a new FeedSource instance
    pub fn new(id: String, name: String, url: String) -> Self {
        let now = chrono::Utc::now().to_rfc3339();
        
        Self {
            id,
            name,
            url,
            enabled: true,
            last_fetched: None,
            created_at: now,
        }
    }



    /// Validate the feed source data
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Feed source ID cannot be empty".to_string());
        }

        if self.name.trim().is_empty() {
            return Err("Feed source name cannot be empty".to_string());
        }

        if self.url.trim().is_empty() {
            return Err("Feed source URL cannot be empty".to_string());
        }

        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err("Feed source URL must be a valid HTTP/HTTPS URL".to_string());
        }

        // Validate datetime format if last_fetched is set
        if let Some(ref last_fetched) = self.last_fetched {
            if chrono::DateTime::parse_from_rfc3339(last_fetched).is_err() {
                return Err("Invalid last_fetched datetime format".to_string());
            }
        }

        if chrono::DateTime::parse_from_rfc3339(&self.created_at).is_err() {
            return Err("Invalid created_at datetime format".to_string());
        }

        Ok(())
    }

    /// Enable the feed source
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the feed source
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Update the last fetched timestamp
    pub fn update_last_fetched(&mut self) {
        self.last_fetched = Some(chrono::Utc::now().to_rfc3339());
    }

    /// Check if the feed source is active (enabled)
    pub fn is_active(&self) -> bool {
        self.enabled
    }
}