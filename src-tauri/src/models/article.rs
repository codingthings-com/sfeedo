use serde::{Deserialize, Serialize};

/// Represents a news article in the feed reader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Article {
    pub id: String,
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: String,
    pub source_id: String,
    pub published_at: String, // ISO 8601 datetime string
    pub fetched_at: String,   // ISO 8601 datetime string
}

impl Article {
    /// Create a new Article instance
    pub fn new(
        id: String,
        title: String,
        summary: Option<String>,
        content: Option<String>,
        url: String,
        source_id: String,
        published_at: String,
    ) -> Self {
        let now = chrono::Utc::now().to_rfc3339();

        Self {
            id,
            title,
            summary,
            content,
            url,
            source_id,
            published_at,
            fetched_at: now,
        }
    }

    /// Validate the article data
    pub fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("Article ID cannot be empty".to_string());
        }

        if self.title.trim().is_empty() {
            return Err("Article title cannot be empty".to_string());
        }

        if self.url.trim().is_empty() {
            return Err("Article URL cannot be empty".to_string());
        }

        if !self.url.starts_with("http://") && !self.url.starts_with("https://") {
            return Err("Article URL must be a valid HTTP/HTTPS URL".to_string());
        }

        if self.source_id.trim().is_empty() {
            return Err("Article source_id cannot be empty".to_string());
        }

        // Validate datetime format
        if chrono::DateTime::parse_from_rfc3339(&self.published_at).is_err() {
            return Err("Invalid published_at datetime format".to_string());
        }

        if chrono::DateTime::parse_from_rfc3339(&self.fetched_at).is_err() {
            return Err("Invalid fetched_at datetime format".to_string());
        }

        Ok(())
    }
}
