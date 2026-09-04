use chrono::{DateTime, Utc};
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
    pub published_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
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
        published_at: DateTime<Utc>,
    ) -> Self {
        let now = Utc::now();

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

        Ok(())
    }
}
