use crate::models::Article;
use tokio::time::{Duration, Instant};
use finance_news_aggregator_rs::{NewsClient, NewsArticle as ExternalNewsArticle};
use chrono::{DateTime, Utc};

/// Available financial news sources
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NewsSource {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub url: String,
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

        Self {
            news_client,
        }
    }

    /// Get all available news sources
    pub fn get_available_sources() -> Vec<NewsSource> {
        vec![
            NewsSource { 
                id: "yahoo".to_string(), 
                name: "Yahoo Finance".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "cnbc".to_string(), 
                name: "CNBC Business".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "marketwatch".to_string(), 
                name: "MarketWatch".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "seeking_alpha".to_string(), 
                name: "Seeking Alpha".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "wsj".to_string(), 
                name: "Wall Street Journal".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "nasdaq".to_string(), 
                name: "NASDAQ".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
            NewsSource { 
                id: "cnn".to_string(), 
                name: "CNN Finance".to_string(), 
                enabled: true,
                url: "Built-in scraper".to_string(),
                last_fetched: None,
            },
        ]
    }

    /// Fetch articles from all available news sources
    pub async fn fetch_all_news(&mut self) -> Result<FetchResult, FeedError> {
        let mut all_articles = Vec::new();
        let mut successful_sources = Vec::new();
        let mut failed_sources = Vec::new();
        let start_time = Instant::now();

        let sources = Self::get_available_sources();
        let enabled_sources: Vec<_> = sources.iter().filter(|s| s.enabled).collect();

        log::info!("Starting fetch from {} news sources", enabled_sources.len());

        for (index, source) in enabled_sources.iter().enumerate() {
            log::debug!("Processing source {}/{}: {}", index + 1, enabled_sources.len(), source.name);

            match self.fetch_from_single_source(&source.id, &source.name).await {
                Ok(articles) => {
                    log::info!("Successfully fetched {} articles from {}", articles.len(), source.name);
                    all_articles.extend(articles);
                    successful_sources.push(source.id.clone());
                }
                Err(e) => {
                    log::error!("Failed to fetch from {}: {}", source.name, e);
                    failed_sources.push(NewsSourceError {
                        source_id: source.id.clone(),
                        source_name: source.name.clone(),
                        error: e.user_message(),
                    });
                }
            }

            // Add a small delay between requests to be respectful to servers
            if index < enabled_sources.len() - 1 {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }

        let duration = start_time.elapsed();
        
        log::info!(
            "Fetch completed in {:?}. Success: {}, Failed: {}, Total articles: {}",
            duration, successful_sources.len(), failed_sources.len(), all_articles.len()
        );

        Ok(FetchResult {
            articles: all_articles,
            successful_sources,
            failed_sources,
            duration,
        })
    }

    /// Fetch articles from a single news source using the built-in aggregator functions
    async fn fetch_from_single_source(&mut self, source_id: &str, source_name: &str) -> Result<Vec<Article>, FeedError> {
        log::info!("Fetching articles from source: {}", source_name);

        let news_articles = match source_id {
            "yahoo" => {
                self.news_client.yahoo_finance().market_summary().await
                    .map_err(|e| FeedError::NetworkError(format!("Yahoo Finance error: {}", e)))?
            }
            "cnbc" => {
                self.news_client.cnbc().business().await
                    .map_err(|e| FeedError::NetworkError(format!("CNBC error: {}", e)))?
            }
            "marketwatch" => {
                self.news_client.market_watch().market_pulse().await
                    .map_err(|e| FeedError::NetworkError(format!("MarketWatch error: {}", e)))?
            }
            "seeking_alpha" => {
                self.news_client.seeking_alpha().latest_articles().await
                    .map_err(|e| FeedError::NetworkError(format!("Seeking Alpha error: {}", e)))?
            }
            "wsj" => {
                self.news_client.wsj().market_news().await
                    .map_err(|e| FeedError::NetworkError(format!("WSJ error: {}", e)))?
            }
            "nasdaq" => {
                self.news_client.nasdaq().stocks().await
                    .map_err(|e| FeedError::NetworkError(format!("NASDAQ error: {}", e)))?
            }
            "cnn" => {
                self.news_client.cnn_finance().markets().await
                    .map_err(|e| FeedError::NetworkError(format!("CNN Finance error: {}", e)))?
            }
            _ => {
                return Err(FeedError::ConfigurationError(
                    format!("Unsupported news source: {}", source_id)
                ));
            }
        };

        log::info!("Fetched {} articles from {}", news_articles.len(), source_name);

        // Convert ExternalNewsArticle to our internal Article model
        let articles = news_articles
            .into_iter()
            .map(|mut article| {
                article.source = Some(source_name.to_string());
                self.convert_external_news_article_to_article(article, source_id)
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(articles)
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
        } else {
            trimmed.to_string()
        };

        // Try parsing the normalized string
        if normalized != trimmed {
            if let Ok(dt) = DateTime::parse_from_rfc2822(&normalized) {
                return Some(dt.with_timezone(&Utc));
            }
        }

        None
    }

    /// Convert ExternalNewsArticle to our Article model (following feed.rs.example pattern)
    fn convert_external_news_article_to_article(&self, news_article: ExternalNewsArticle, source_id: &str) -> Result<Article, FeedError> {
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
        article.validate()
            .map_err(|e| FeedError::ParseError(format!("Invalid article data: {}", e)))?;

        Ok(article)
    }

    /// Generate a unique ID for an article from ExternalNewsArticle
    fn generate_article_id_from_external(&self, news_article: &ExternalNewsArticle, source_id: &str) -> String {
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
                "Unable to connect to news sources. Please check your internet connection.".to_string()
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
