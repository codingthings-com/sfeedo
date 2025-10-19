use serde::{Deserialize, Serialize};
use rusqlite::{Row, Result as SqliteResult};

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
    pub is_read: bool,
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
            is_read: false,
        }
    }

    /// Create an Article from a database row
    pub fn from_row(row: &Row) -> SqliteResult<Self> {
        Ok(Article {
            id: row.get("id")?,
            title: row.get("title")?,
            summary: row.get("summary")?,
            content: row.get("content")?,
            url: row.get("url")?,
            source_id: row.get("source_id")?,
            published_at: row.get("published_at")?,
            fetched_at: row.get("fetched_at")?,
            is_read: row.get("is_read")?,
        })
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

    /// Mark the article as read
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }

    /// Mark the article as unread
    pub fn mark_as_unread(&mut self) {
        self.is_read = false;
    }
}