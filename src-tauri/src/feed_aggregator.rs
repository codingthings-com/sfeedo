use crate::database::operations::ArticleOperations;
use crate::database::DatabaseConnection;
use crate::models::{Article, FeedSource};
use tokio::time::{Duration, Instant};
use url::Url;
use serde::{Deserialize, Serialize};


/// A simple news item structure for internal use
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub title: String,
    pub summary: Option<String>,
    pub content: Option<String>,
    pub url: String,
    pub published_at: Option<chrono::DateTime<chrono::Utc>>,
}

/// Feed aggregation service that fetches and processes news from multiple sources
pub struct FeedAggregator<'a> {
    db: &'a DatabaseConnection,
    article_ops: ArticleOperations<'a>,
    client: reqwest::Client,
}

impl<'a> FeedAggregator<'a> {
    /// Create a new FeedAggregator instance
    pub fn new(db: &'a DatabaseConnection) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("Sfeedo Feed Reader/1.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            db,
            article_ops: ArticleOperations::new(db),
            client,
        }
    }

    /// Fetch articles from a single feed source with retry logic
    pub async fn fetch_from_source(&self, feed_source: &FeedSource) -> Result<Vec<Article>, FeedError> {
        self.fetch_from_source_with_retry(feed_source, 3).await
    }

    /// Fetch articles from a single feed source with configurable retry attempts
    async fn fetch_from_source_with_retry(&self, feed_source: &FeedSource, max_retries: u32) -> Result<Vec<Article>, FeedError> {
        log::info!("Fetching articles from source: {} ({})", feed_source.name, feed_source.url);

        // Validate URL before attempting to fetch
        self.validate_feed_url(&feed_source.url)?;

        let mut last_error = None;
        
        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = self.calculate_retry_delay(attempt);
                log::info!("Retrying fetch from {} (attempt {}/{}) after {:?}", 
                          feed_source.name, attempt + 1, max_retries + 1, delay);
                tokio::time::sleep(delay).await;
            }

            match self.fetch_single_attempt(feed_source).await {
                Ok(articles) => {
                    if attempt > 0 {
                        log::info!("Successfully fetched from {} after {} retries", 
                                 feed_source.name, attempt);
                    }
                    return Ok(articles);
                }
                Err(e) => {
                    last_error = Some(e.clone());
                    
                    // Don't retry for certain types of errors
                    if !self.should_retry_error(&e) {
                        log::warn!("Non-retryable error from {}: {}", feed_source.name, e);
                        return Err(e);
                    }
                    
                    log::warn!("Attempt {} failed for {}: {}", attempt + 1, feed_source.name, e);
                }
            }
        }

        // All retries exhausted
        let final_error = last_error.unwrap_or_else(|| 
            FeedError::NetworkError("Unknown error during fetch".to_string())
        );
        
        log::error!("Failed to fetch from {} after {} attempts: {}", 
                   feed_source.name, max_retries + 1, final_error);
        
        Err(final_error)
    }

    /// Perform a single fetch attempt without retry logic
    async fn fetch_single_attempt(&self, feed_source: &FeedSource) -> Result<Vec<Article>, FeedError> {
        // Fetch the RSS/Atom feed with timeout
        let response = self.client
            .get(&feed_source.url)
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    FeedError::NetworkError(format!("Timeout fetching from {}", feed_source.url))
                } else if e.is_connect() {
                    FeedError::NetworkError(format!("Connection failed to {}", feed_source.url))
                } else {
                    FeedError::NetworkError(format!("Network error fetching from {}: {}", feed_source.url, e))
                }
            })?;

        // Check HTTP status
        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                404 => FeedError::ConfigurationError(format!("Feed not found (404): {}", feed_source.url)),
                403 | 401 => FeedError::ConfigurationError(format!("Access denied ({}): {}", status, feed_source.url)),
                500..=599 => FeedError::NetworkError(format!("Server error ({}): {}", status, feed_source.url)),
                _ => FeedError::NetworkError(format!("HTTP error ({}): {}", status, feed_source.url)),
            });
        }

        // Read response content
        let content = response
            .text()
            .await
            .map_err(|e| FeedError::NetworkError(format!("Failed to read response from {}: {}", feed_source.url, e)))?;

        // Validate content is not empty
        if content.trim().is_empty() {
            return Err(FeedError::ParseError(format!("Empty response from {}", feed_source.url)));
        }

        // Parse the feed content
        let news_items = self.parse_feed_content(&content, &feed_source.url)?;

        log::info!("Fetched {} articles from {}", news_items.len(), feed_source.name);

        // Convert NewsItem to Article
        let articles = news_items
            .into_iter()
            .map(|item| self.convert_news_item_to_article(item, &feed_source.id))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Calculate exponential backoff delay for retry attempts
    fn calculate_retry_delay(&self, attempt: u32) -> Duration {
        let base_delay = Duration::from_secs(2);
        let max_delay = Duration::from_secs(60);
        
        let delay = base_delay * (2_u32.pow(attempt.saturating_sub(1)));
        std::cmp::min(delay, max_delay)
    }

    /// Determine if an error should trigger a retry
    fn should_retry_error(&self, error: &FeedError) -> bool {
        match error {
            FeedError::NetworkError(msg) => {
                // Retry network errors except for DNS resolution failures
                !msg.contains("dns") && !msg.contains("name resolution")
            }
            FeedError::ParseError(_) => false, // Don't retry parse errors
            FeedError::ConfigurationError(_) => false, // Don't retry config errors (404, 403, etc.)
            FeedError::StorageError(_) => false, // Don't retry storage errors
        }
    }

    /// Parse RSS/Atom feed content into NewsItems
    fn parse_feed_content(&self, content: &str, feed_url: &str) -> Result<Vec<NewsItem>, FeedError> {
        use quick_xml::Reader;
        use quick_xml::events::Event;

        let mut reader = Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut news_items = Vec::new();
        let mut current_item: Option<NewsItem> = None;
        let mut current_element = String::new();
        let mut current_text = String::new();
        let mut in_item = false;

        loop {
            match reader.read_event() {
                Ok(Event::Start(ref e)) => {
                    current_element = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if current_element == "item" || current_element == "entry" {
                        in_item = true;
                        current_item = Some(NewsItem {
                            title: String::new(),
                            summary: None,
                            content: None,
                            url: String::new(),
                            published_at: None,
                        });
                    }
                    current_text.clear();
                }
                Ok(Event::Text(e)) => {
                    current_text.push_str(&String::from_utf8_lossy(&e));
                }
                Ok(Event::CData(e)) => {
                    current_text.push_str(&String::from_utf8_lossy(&e));
                }
                Ok(Event::End(ref e)) => {
                    let element_name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                    
                    if in_item && current_item.is_some() {
                        let item = current_item.as_mut().unwrap();
                        
                        match element_name.as_str() {
                            "title" => {
                                item.title = current_text.trim().to_string();
                            }
                            "description" | "summary" | "content" => {
                                if item.summary.is_none() {
                                    item.summary = Some(current_text.trim().to_string());
                                }
                            }
                            "link" | "guid" => {
                                if item.url.is_empty() {
                                    item.url = current_text.trim().to_string();
                                }
                            }
                            "pubDate" | "published" | "updated" => {
                                if let Ok(dt) = chrono::DateTime::parse_from_rfc2822(&current_text.trim()) {
                                    item.published_at = Some(dt.with_timezone(&chrono::Utc));
                                } else if let Ok(dt) = chrono::DateTime::parse_from_rfc3339(&current_text.trim()) {
                                    item.published_at = Some(dt.with_timezone(&chrono::Utc));
                                }
                            }
                            "item" | "entry" => {
                                if let Some(item) = current_item.take() {
                                    // Validate that we have minimum required fields
                                    if !item.title.is_empty() && !item.url.is_empty() {
                                        news_items.push(item);
                                    } else {
                                        log::warn!("Skipping incomplete item from {}: title='{}', url='{}'", 
                                                 feed_url, item.title, item.url);
                                    }
                                }
                                in_item = false;
                            }
                            _ => {}
                        }
                    }
                    current_text.clear();
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    log::error!("Error parsing XML from {}: {}", feed_url, e);
                    return Err(FeedError::ParseError(format!("XML parsing error: {}", e)));
                }
                _ => {}
            }
        }

        Ok(news_items)
    }

    /// Fetch articles from multiple feed sources with graceful error handling
    pub async fn fetch_from_sources(&self, feed_sources: &[FeedSource]) -> Result<FetchResult, FeedError> {
        let mut all_articles = Vec::new();
        let mut successful_sources = Vec::new();
        let mut failed_sources = Vec::new();
        let start_time = Instant::now();

        let enabled_sources: Vec<_> = feed_sources.iter()
            .filter(|source| source.enabled)
            .collect();

        if enabled_sources.is_empty() {
            return Err(FeedError::ConfigurationError("No enabled feed sources found".to_string()));
        }

        log::info!("Starting fetch from {} enabled sources", enabled_sources.len());

        for (index, feed_source) in enabled_sources.iter().enumerate() {
            log::debug!("Processing source {}/{}: {}", index + 1, enabled_sources.len(), feed_source.name);

            match self.fetch_from_source(feed_source).await {
                Ok(articles) => {
                    log::info!("Successfully fetched {} articles from {}", articles.len(), feed_source.name);
                    all_articles.extend(articles);
                    successful_sources.push(feed_source.id.clone());
                }
                Err(e) => {
                    log::error!("Failed to fetch from {}: {}", feed_source.name, e);
                    failed_sources.push(FeedSourceError {
                        source_id: feed_source.id.clone(),
                        source_name: feed_source.name.clone(),
                        error: e.user_message(), // Use user-friendly error message
                    });
                }
            }

            // Add a small delay between requests to be respectful to servers
            if index < enabled_sources.len() - 1 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        let duration = start_time.elapsed();
        
        // Log summary with appropriate level based on results
        if failed_sources.is_empty() {
            log::info!(
                "Fetch completed successfully in {:?}. {} sources, {} articles",
                duration, successful_sources.len(), all_articles.len()
            );
        } else if successful_sources.is_empty() {
            log::error!(
                "All sources failed in {:?}. {} failed sources",
                duration, failed_sources.len()
            );
        } else {
            log::warn!(
                "Fetch completed with partial failures in {:?}. Success: {}, Failed: {}, Total articles: {}",
                duration, successful_sources.len(), failed_sources.len(), all_articles.len()
            );
        }

        Ok(FetchResult {
            articles: all_articles,
            successful_sources,
            failed_sources,
            duration,
        })
    }

    /// Store fetched articles in the database with error recovery
    pub async fn store_articles(&self, articles: &[Article]) -> Result<StorageResult, FeedError> {
        if articles.is_empty() {
            return Ok(StorageResult {
                total_articles: 0,
                new_articles: 0,
                duplicate_articles: 0,
            });
        }

        log::info!("Storing {} articles in database", articles.len());

        // Filter out duplicates and validate articles
        let mut new_articles = Vec::new();
        let mut duplicate_count = 0;
        let mut invalid_count = 0;

        for article in articles {
            // Validate article before processing
            if let Err(e) = article.validate() {
                log::warn!("Skipping invalid article '{}': {}", article.title, e);
                invalid_count += 1;
                continue;
            }

            match self.article_ops.get_by_id(&article.id) {
                Ok(Some(_)) => {
                    duplicate_count += 1;
                    log::debug!("Duplicate article found: {}", article.title);
                }
                Ok(None) => {
                    new_articles.push(article.clone());
                }
                Err(e) => {
                    log::error!("Error checking for duplicate article {}: {}", article.id, e);
                    // Continue processing other articles instead of failing completely
                    invalid_count += 1;
                }
            }
        }

        // Store new articles in batch with retry logic
        let mut stored_count = 0;
        if !new_articles.is_empty() {
            match self.store_articles_with_retry(&new_articles, 3).await {
                Ok(count) => {
                    stored_count = count;
                    log::info!("Stored {} new articles, {} duplicates skipped, {} invalid", 
                             stored_count, duplicate_count, invalid_count);
                }
                Err(e) => {
                    log::error!("Failed to store articles after retries: {}", e);
                    return Err(e);
                }
            }
        } else {
            log::info!("No new articles to store. {} duplicates skipped, {} invalid", 
                     duplicate_count, invalid_count);
        }

        Ok(StorageResult {
            total_articles: articles.len(),
            new_articles: stored_count,
            duplicate_articles: duplicate_count,
        })
    }

    /// Store articles with retry logic for database errors
    async fn store_articles_with_retry(&self, articles: &[Article], max_retries: u32) -> Result<usize, FeedError> {
        let mut last_error = None;

        for attempt in 0..=max_retries {
            if attempt > 0 {
                let delay = Duration::from_millis(100 * (2_u64.pow(attempt)));
                log::info!("Retrying database storage (attempt {}/{}) after {:?}", 
                          attempt + 1, max_retries + 1, delay);
                tokio::time::sleep(delay).await;
            }

            match self.article_ops.insert_batch(articles) {
                Ok(_) => {
                    if attempt > 0 {
                        log::info!("Successfully stored articles after {} retries", attempt);
                    }
                    return Ok(articles.len());
                }
                Err(e) => {
                    last_error = Some(e);
                    log::warn!("Storage attempt {} failed: {}", attempt + 1, last_error.as_ref().unwrap());
                }
            }
        }

        let final_error = last_error.unwrap();
        Err(FeedError::StorageError(format!("Failed to store articles after {} attempts: {}", 
                                           max_retries + 1, final_error)))
    }

    /// Fetch and store articles from all enabled sources
    pub async fn refresh_all_feeds(&self, feed_sources: &[FeedSource]) -> Result<RefreshResult, FeedError> {
        let fetch_result = self.fetch_from_sources(feed_sources).await?;
        let storage_result = self.store_articles(&fetch_result.articles).await?;

        Ok(RefreshResult {
            fetch_result,
            storage_result,
        })
    }

    /// Convert NewsItem to our Article model
    fn convert_news_item_to_article(&self, news_item: NewsItem, source_id: &str) -> Result<Article, FeedError> {
        // Generate a unique ID for the article
        let article_id = self.generate_article_id(&news_item, source_id);

        // Convert published date to RFC3339 format
        let published_at = news_item.published_at
            .map(|dt| dt.to_rfc3339())
            .unwrap_or_else(|| chrono::Utc::now().to_rfc3339());

        let article = Article::new(
            article_id,
            news_item.title,
            news_item.summary,
            news_item.content,
            news_item.url,
            source_id.to_string(),
            published_at,
        );

        // Validate the article before returning
        article.validate()
            .map_err(|e| FeedError::ParseError(format!("Invalid article data: {}", e)))?;

        Ok(article)
    }

    /// Generate a unique ID for an article based on its content and source
    fn generate_article_id(&self, news_item: &NewsItem, source_id: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Hash the URL and title to create a unique ID
        news_item.url.hash(&mut hasher);
        news_item.title.hash(&mut hasher);
        source_id.hash(&mut hasher);
        
        let hash = hasher.finish();
        format!("{}_{:x}", source_id, hash)
    }

    /// Validate a feed URL
    fn validate_feed_url(&self, url: &str) -> Result<(), FeedError> {
        let parsed_url = Url::parse(url)
            .map_err(|_| FeedError::ConfigurationError("Invalid URL format".to_string()))?;

        match parsed_url.scheme() {
            "http" | "https" => Ok(()),
            _ => Err(FeedError::ConfigurationError(
                "URL must use HTTP or HTTPS protocol".to_string()
            )),
        }
    }
}

/// Result of fetching articles from feed sources
#[derive(Debug, Clone)]
pub struct FetchResult {
    pub articles: Vec<Article>,
    pub successful_sources: Vec<String>,
    pub failed_sources: Vec<FeedSourceError>,
    pub duration: Duration,
}

/// Result of storing articles in the database
#[derive(Debug, Clone)]
pub struct StorageResult {
    pub total_articles: usize,
    pub new_articles: usize,
    pub duplicate_articles: usize,
}

/// Combined result of refresh operation
#[derive(Debug, Clone)]
pub struct RefreshResult {
    pub fetch_result: FetchResult,
    pub storage_result: StorageResult,
}

/// Error information for a failed feed source
#[derive(Debug, Clone)]
pub struct FeedSourceError {
    pub source_id: String,
    pub source_name: String,
    pub error: String,
}

/// Errors that can occur during feed aggregation
#[derive(Debug, Clone, thiserror::Error)]
pub enum FeedError {
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Parse error: {0}")]
    ParseError(String),
    
    #[error("Configuration error: {0}")]
    ConfigurationError(String),
    
    #[error("Storage error: {0}")]
    StorageError(String),
}

impl FeedError {
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            FeedError::NetworkError(_) => {
                "Unable to connect to news sources. Please check your internet connection.".to_string()
            }
            FeedError::ParseError(_) => {
                "Error processing news feed. The source may be temporarily unavailable.".to_string()
            }
            FeedError::ConfigurationError(_) => {
                "Configuration error. Please check your settings.".to_string()
            }
            FeedError::StorageError(_) => {
                "Database error. Please restart the application.".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::database::DatabaseConnection;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_generate_article_id() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseConnection::new_with_path(db_path).unwrap();
        let aggregator = FeedAggregator::new(&db);

        let news_item = NewsItem {
            title: "Test Article".to_string(),
            summary: Some("Test summary".to_string()),
            content: None,
            url: "https://example.com/article1".to_string(),
            published_at: None,
        };

        let id1 = aggregator.generate_article_id(&news_item, "test-source");
        let id2 = aggregator.generate_article_id(&news_item, "test-source");
        let id3 = aggregator.generate_article_id(&news_item, "other-source");

        // Same content and source should generate same ID
        assert_eq!(id1, id2);
        
        // Different source should generate different ID
        assert_ne!(id1, id3);
        
        // ID should contain source prefix
        assert!(id1.starts_with("test-source_"));
        assert!(id3.starts_with("other-source_"));
    }

    #[test]
    fn test_validate_feed_url() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let db = DatabaseConnection::new_with_path(db_path).unwrap();
        let aggregator = FeedAggregator::new(&db);

        // Valid URLs
        assert!(aggregator.validate_feed_url("https://example.com/rss.xml").is_ok());
        assert!(aggregator.validate_feed_url("http://feeds.example.com/news").is_ok());

        // Invalid URLs
        assert!(aggregator.validate_feed_url("ftp://example.com/feed").is_err());
        assert!(aggregator.validate_feed_url("not-a-url").is_err());
    }
}