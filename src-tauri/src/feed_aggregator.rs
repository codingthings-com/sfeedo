use crate::models::Article;
use chrono::{DateTime, Utc};
use finance_news_aggregator_rs::{
    news_source::NewsSource as NewsSourceTrait, NewsArticle as ExternalNewsArticle, NewsClient,
};
use tokio::time::{Duration, Instant};

/// Available financial news sources with topics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewsSource {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub source_type: String, // "builtin" or "custom"
    pub url: Option<String>,
    pub available_topics: Vec<String>,
    pub enabled_topics: Vec<String>,
    pub last_fetched: Option<String>,
}

/// Feed aggregation service that fetches and processes news from financial sources
pub struct FeedAggregator {
    news_client: NewsClient,
}

impl FeedAggregator {
    /// Create a new FeedAggregator instance
    pub fn new() -> Self {
        let news_client = NewsClient::new();

        Self { news_client }
    }

    /// Get available topics for a built-in source dynamically
    pub fn get_available_topics_for_source(source_id: &str) -> Vec<String> {
        let mut client = NewsClient::new();

        let topics: Vec<&str> = match source_id {
            "yahoo" => client.yahoo_finance().available_topics(),
            "cnbc" => client.cnbc().available_topics(),
            "marketwatch" => client.market_watch().available_topics(),
            "seeking_alpha" => client.seeking_alpha().available_topics(),
            "wsj" => client.wsj().available_topics(),
            "nasdaq" => client.nasdaq().available_topics(),
            "cnn" => client.cnn_finance().available_topics(),
            _ => vec![],
        };

        // Convert &str to String
        let result: Vec<String> = topics.iter().map(|s| s.to_string()).collect();
        log::info!("Available topics for {}: {:?}", source_id, result);
        result
    }

    /// Convert FeedSourceConfig to NewsSource
    fn config_to_news_source(config: &crate::models::FeedSourceConfig) -> NewsSource {
        let available_topics = Self::get_available_topics_for_source(&config.id);

        NewsSource {
            id: config.id.clone(),
            name: config.name.clone(),
            enabled: config.enabled,
            source_type: "builtin".to_string(),
            url: None,
            available_topics,
            enabled_topics: config.enabled_topics.clone(),
            last_fetched: config.last_fetched.clone(),
        }
    }

    /// Convert CustomFeedConfig to NewsSource
    fn custom_to_news_source(config: &crate::models::CustomFeedConfig) -> NewsSource {
        NewsSource {
            id: config.id.clone(),
            name: config.name.clone(),
            enabled: config.enabled,
            source_type: "custom".to_string(),
            url: Some(config.url.clone()),
            available_topics: vec![],
            enabled_topics: vec![],
            last_fetched: config.last_fetched.clone(),
        }
    }

    /// Get all available news sources from configuration (built-in + custom)
    pub fn get_available_sources_from_config(config: &crate::models::AppConfig) -> Vec<NewsSource> {
        let mut sources: Vec<NewsSource> = config
            .feed_sources
            .iter()
            .map(Self::config_to_news_source)
            .collect();

        // Add custom feeds
        sources.extend(config.custom_feeds.iter().map(Self::custom_to_news_source));

        sources
    }

    /// Get all available news sources (fallback for backward compatibility)
    pub fn get_available_sources() -> Vec<NewsSource> {
        // Return default sources for backward compatibility
        let default_config = crate::models::AppConfig::default();
        Self::get_available_sources_from_config(&default_config)
    }

    /// Fetch articles from all available news sources using configuration
    pub async fn fetch_all_news_with_config(
        &mut self,
        config: &crate::models::AppConfig,
    ) -> Result<FetchResult, FeedError> {
        let mut all_articles = Vec::new();
        let mut successful_sources = Vec::new();
        let mut failed_sources = Vec::new();
        let start_time = Instant::now();

        let sources = Self::get_available_sources_from_config(config);
        let enabled_sources: Vec<_> = sources.iter().filter(|s| s.enabled).collect();

        log::info!(
            "Starting fetch from {} enabled news sources (out of {} total)",
            enabled_sources.len(),
            sources.len()
        );

        if enabled_sources.is_empty() {
            log::warn!("No enabled news sources found");
            return Ok(FetchResult {
                articles: all_articles,
                successful_sources,
                failed_sources,
                duration: start_time.elapsed(),
            });
        }

        for (index, source_config) in enabled_sources.iter().enumerate() {
            log::debug!(
                "Processing source {}/{}: {}",
                index + 1,
                enabled_sources.len(),
                source_config.name
            );

            if source_config.source_type == "builtin" {
                // Find the built-in config for this source
                let builtin_config = config
                    .feed_sources
                    .iter()
                    .find(|s| s.id == source_config.id);

                if let Some(builtin_config) = builtin_config {
                    // Fetch from all enabled topics
                    for topic in &builtin_config.enabled_topics {
                        log::debug!("Fetching topic '{}' from {}", topic, source_config.name);

                        match self
                            .fetch_from_builtin_source(
                                &source_config.id,
                                &source_config.name,
                                topic,
                            )
                            .await
                        {
                            Ok(articles) => {
                                log::info!(
                                    "Successfully fetched {} articles from {} ({})",
                                    articles.len(),
                                    source_config.name,
                                    topic
                                );
                                all_articles.extend(articles);
                            }
                            Err(e) => {
                                log::error!(
                                    "Failed to fetch {} from {}: {}",
                                    topic,
                                    source_config.name,
                                    e
                                );
                                // Continue with other topics even if one fails
                            }
                        }

                        // Small delay between topics
                        tokio::time::sleep(Duration::from_millis(300)).await;
                    }
                    successful_sources.push(source_config.id.clone());
                }
            } else if source_config.source_type == "custom" {
                // Find the custom feed config
                let custom_config = config
                    .custom_feeds
                    .iter()
                    .find(|s| s.id == source_config.id);

                if let Some(custom_config) = custom_config {
                    match self
                        .fetch_from_custom_feed(&custom_config.url, &source_config.name)
                        .await
                    {
                        Ok(articles) => {
                            log::info!(
                                "Successfully fetched {} articles from custom feed {}",
                                articles.len(),
                                source_config.name
                            );
                            all_articles.extend(articles);
                            successful_sources.push(source_config.id.clone());
                        }
                        Err(e) => {
                            log::error!(
                                "Failed to fetch custom feed {}: {}",
                                source_config.name,
                                e
                            );
                            failed_sources.push(NewsSourceError {
                                source_id: source_config.id.clone(),
                                source_name: source_config.name.clone(),
                                error: e.user_message(),
                            });
                        }
                    }
                }
            }

            // Add a small delay between sources to be respectful to servers
            if index < enabled_sources.len() - 1 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        let duration = start_time.elapsed();

        log::info!(
            "Fetch completed in {:?}. Success: {}, Failed: {}, Total articles: {}",
            duration,
            successful_sources.len(),
            failed_sources.len(),
            all_articles.len()
        );

        Ok(FetchResult {
            articles: all_articles,
            successful_sources,
            failed_sources,
            duration,
        })
    }

    /// Fetch articles from a built-in news source with specific topic
    async fn fetch_from_builtin_source(
        &mut self,
        source_id: &str,
        source_name: &str,
        topic: &str,
    ) -> Result<Vec<Article>, FeedError> {
        log::info!("Fetching topic '{}' from source: {}", topic, source_name);

        // Use fetch_topic method from the library
        let news_articles = match source_id {
            "yahoo" => self.news_client.yahoo_finance().fetch_topic(topic).await,
            "cnbc" => self.news_client.cnbc().fetch_topic(topic).await,
            "marketwatch" => self.news_client.market_watch().fetch_topic(topic).await,
            "seeking_alpha" => self.news_client.seeking_alpha().fetch_topic(topic).await,
            "wsj" => self.news_client.wsj().fetch_topic(topic).await,
            "nasdaq" => self.news_client.nasdaq().fetch_topic(topic).await,
            "cnn" => self.news_client.cnn_finance().fetch_topic(topic).await,
            _ => {
                return Err(FeedError::ConfigurationError(format!(
                    "Unknown source: {}",
                    source_id
                )))
            }
        }
        .map_err(|e| {
            FeedError::NetworkError(format!("{} error ({}): {}", source_name, topic, e))
        })?;

        log::info!(
            "Fetched {} articles from {} ({})",
            news_articles.len(),
            source_name,
            topic
        );

        // Convert ExternalNewsArticle to our internal Article model
        let articles = news_articles
            .into_iter()
            .map(|mut article| {
                article.source = Some(format!("{} - {}", source_name, topic));
                self.convert_external_news_article_to_article(article, source_id)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Fetch articles from a custom RSS/Atom feed using generic source
    async fn fetch_from_custom_feed(
        &mut self,
        url: &str,
        source_name: &str,
    ) -> Result<Vec<Article>, FeedError> {
        log::info!("Fetching custom feed: {} ({})", source_name, url);

        // Use the generic source to fetch any RSS/Atom feed
        let news_articles = self
            .news_client
            .generic()
            .fetch_feed_by_url(url)
            .await
            .map_err(|e| FeedError::NetworkError(format!("Custom feed error: {}", e)))?;

        log::info!(
            "Fetched {} articles from custom feed {}",
            news_articles.len(),
            source_name
        );

        // Generate a simple ID from the source name
        let source_id = source_name.to_lowercase().replace(" ", "_");

        // Convert ExternalNewsArticle to our internal Article model
        let articles = news_articles
            .into_iter()
            .map(|mut article| {
                article.source = Some(source_name.to_string());
                self.convert_external_news_article_to_article(article, &source_id)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
    }

    /// Fetch articles from all available news sources (backward compatibility)
    pub async fn fetch_all_news(&mut self) -> Result<FetchResult, FeedError> {
        let default_config = crate::models::AppConfig::default();
        self.fetch_all_news_with_config(&default_config).await
    }

    /// Refresh all feeds using configuration
    pub async fn refresh_all_feeds_with_config(
        &mut self,
        config: &crate::models::AppConfig,
    ) -> Result<FetchResult, FeedError> {
        self.fetch_all_news_with_config(config).await
    }

    /// Refresh all feeds (alias for fetch_all_news for backward compatibility)
    pub async fn refresh_all_feeds(&mut self) -> Result<FetchResult, FeedError> {
        self.fetch_all_news().await
    }

    /// Parse date string using the same logic as the inspiration code
    fn parse_date(&self, date_str: &str) -> Option<DateTime<Utc>> {
        if date_str.is_empty() {
            return None;
        }

        let trimmed = date_str.trim();

        // Try parsing with chrono's built-in RFC 2822 parser first
        if let Ok(dt) = DateTime::parse_from_rfc2822(trimmed) {
            return Some(dt.with_timezone(&Utc));
        }

        // Try parsing with RFC 3339 (ISO 8601) format
        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Some(dt.with_timezone(&Utc));
        }

        // Handle common timezone abbreviations
        let normalized = if trimmed.ends_with(" UTC") {
            trimmed.replace(" UTC", " +0000")
        } else if trimmed.ends_with(" EST") {
            trimmed.replace(" EST", " -0500")
        } else if trimmed.ends_with(" PST") {
            trimmed.replace(" PST", " -0800")
        } else if trimmed.ends_with(" EDT") {
            trimmed.replace(" EDT", " -0400")
        } else if trimmed.ends_with(" PDT") {
            trimmed.replace(" PDT", " -0700")
        } else if trimmed.ends_with(" CST") {
            trimmed.replace(" CST", " -0600")
        } else if trimmed.ends_with(" CDT") {
            trimmed.replace(" CDT", " -0500")
        } else if trimmed.ends_with(" MST") {
            trimmed.replace(" MST", " -0700")
        } else if trimmed.ends_with(" MDT") {
            trimmed.replace(" MDT", " -0600")
        } else {
            trimmed.to_string()
        };

        // Try parsing the normalized string
        if normalized != trimmed {
            if let Ok(dt) = DateTime::parse_from_rfc2822(&normalized) {
                return Some(dt.with_timezone(&Utc));
            }
        }

        // Try additional common formats
        use chrono::NaiveDateTime;

        // ISO 8601 without timezone (assume UTC)
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
            return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
        }

        // ISO 8601 with milliseconds but no timezone
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S%.f") {
            return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
        }

        // Common date-only format (assume midnight UTC)
        if let Ok(date) = chrono::NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            if let Some(dt) = date.and_hms_opt(0, 0, 0) {
                return Some(DateTime::from_naive_utc_and_offset(dt, Utc));
            }
        }

        // Log only if we couldn't parse the date
        log::warn!("Failed to parse date: '{}'", date_str);
        None
    }

    /// Convert ExternalNewsArticle to our Article model (following feed.rs.example pattern)
    fn convert_external_news_article_to_article(
        &self,
        news_article: ExternalNewsArticle,
        source_id: &str,
    ) -> Result<Article, FeedError> {
        // Generate a unique ID for the article
        let article_id = self.generate_article_id_from_external(&news_article, source_id);

        // Extract fields from ExternalNewsArticle following the example pattern
        let title = news_article.title.unwrap_or_default().trim().to_string();
        let url = news_article.link.unwrap_or_default().trim().to_string();
        let summary = news_article.description.map(|s| s.trim().to_string());
        let content = None; // ExternalNewsArticle doesn't have a content field

        // Parse published date using the same logic as feed.rs.example
        let published_at = if let Some(pub_date) = news_article.pub_date {
            self.parse_date(&pub_date)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_else(|| chrono::Utc::now().to_rfc3339())
        } else {
            chrono::Utc::now().to_rfc3339()
        };

        // Debug log to check for title truncation like in the example
        if title.is_empty() {
            log::warn!("Empty title found for article from {}", source_id);
        } else if title.len() < 10 {
            log::warn!("Very short title from {}: '{}'", source_id, title);
        }

        let article = Article::new(
            article_id,
            title,
            summary,
            content,
            url,
            source_id.to_string(),
            published_at,
        );

        // Validate the article before returning
        article
            .validate()
            .map_err(|e| FeedError::ParseError(format!("Invalid article data: {}", e)))?;

        Ok(article)
    }

    /// Generate a unique ID for an article from ExternalNewsArticle
    fn generate_article_id_from_external(
        &self,
        news_article: &ExternalNewsArticle,
        source_id: &str,
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();

        // Hash the URL and title to create a unique ID
        if let Some(ref url) = news_article.link {
            url.hash(&mut hasher);
        }
        if let Some(ref title) = news_article.title {
            title.hash(&mut hasher);
        }
        source_id.hash(&mut hasher);

        let hash = hasher.finish();
        format!("{}_{:x}", source_id, hash)
    }
}

/// Result of fetching articles from feed sources
#[derive(Debug, Clone)]
pub struct FetchResult {
    pub articles: Vec<Article>,
    pub successful_sources: Vec<String>,
    pub failed_sources: Vec<NewsSourceError>,
    pub duration: Duration,
}

/// Error information for a failed news source
#[derive(Debug, Clone)]
pub struct NewsSourceError {
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
}

impl FeedError {
    /// Get a user-friendly error message
    pub fn user_message(&self) -> String {
        match self {
            FeedError::NetworkError(_) => {
                "Unable to connect to news sources. Please check your internet connection."
                    .to_string()
            }
            FeedError::ParseError(_) => {
                "Error processing news feed. The source may be temporarily unavailable.".to_string()
            }
            FeedError::ConfigurationError(_) => {
                "Configuration error. Please check your settings.".to_string()
            }
        }
    }
}
