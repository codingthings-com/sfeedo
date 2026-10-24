use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_refresh: AutoRefreshConfig,
    pub ui: UiConfig,
    pub feed_sources: Vec<FeedSourceConfig>,
    #[serde(default)]
    pub custom_feeds: Vec<CustomFeedConfig>,
    #[serde(default)]
    pub window_state: WindowState,
}

/// Feed source configuration (built-in sources)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSourceConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    #[serde(default)]
    pub enabled_topics: Vec<String>, // Topics enabled for this source
    pub last_fetched: Option<String>,
}

/// Custom feed configuration (user-added RSS/Atom feeds)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomFeedConfig {
    pub id: String,
    pub name: String,
    pub url: String,
    pub enabled: bool,
    pub last_fetched: Option<String>,
}

/// Auto-refresh configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutoRefreshConfig {
    pub enabled: bool,
    pub interval_minutes: u32,
}

/// UI configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    pub show_notifications: bool,
}

/// Window state configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowState {
    pub x: Option<i32>,
    pub y: Option<i32>,
    pub width: Option<u32>,
    pub height: Option<u32>,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: Some(400),
            height: Some(1000),
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_refresh: AutoRefreshConfig {
                enabled: true,
                interval_minutes: 30,
            },
            ui: UiConfig {
                show_notifications: true,
            },
            window_state: WindowState::default(),
            feed_sources: vec![
                FeedSourceConfig {
                    id: "yahoo".to_string(),
                    name: "Yahoo Finance".to_string(),
                    enabled: true,
                    enabled_topics: vec!["headlines".to_string()],
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "cnbc".to_string(),
                    name: "CNBC".to_string(),
                    enabled: true,
                    enabled_topics: vec!["business".to_string()],
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "marketwatch".to_string(),
                    name: "MarketWatch".to_string(),
                    enabled: true,
                    enabled_topics: vec!["market_pulse".to_string()],
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "seeking_alpha".to_string(),
                    name: "Seeking Alpha".to_string(),
                    enabled: true,
                    enabled_topics: vec!["latest_articles".to_string()],
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "wsj".to_string(),
                    name: "Wall Street Journal".to_string(),
                    enabled: true,
                    enabled_topics: vec!["market_news".to_string()],
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "nasdaq".to_string(),
                    name: "NASDAQ".to_string(),
                    enabled: true,
                    enabled_topics: vec!["stocks".to_string()],
                    last_fetched: None,
                },
            ],
            custom_feeds: vec![],
        }
    }
}

impl AppConfig {
    /// Create a new AppConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate auto-refresh interval (2 minutes to 24 hours)
        if self.auto_refresh.interval_minutes < 2 {
            return Err("Auto-refresh interval must be at least 2 minutes".to_string());
        }

        if self.auto_refresh.interval_minutes > 1440 {
            // 24 hours
            return Err("Auto-refresh interval cannot exceed 24 hours".to_string());
        }

        // No additional UI validation needed for teletext interface

        Ok(())
    }

    /// Add a custom feed, returning its generated ID
    pub fn add_custom_feed(&mut self, name: &str, url: &str) -> Result<String, String> {
        let (name, url) = Self::validate_custom_feed_input(name, url)?;
        self.ensure_custom_feed_name_unique(&name, None)?;

        // Derive a stable, attribute-safe ID from the name and make it unique
        let base_id = format!("custom_{}", slugify(&name));
        let mut id = base_id.clone();
        let mut suffix = 2;
        while self.custom_feeds.iter().any(|f| f.id == id) {
            id = format!("{}_{}", base_id, suffix);
            suffix += 1;
        }

        self.custom_feeds.push(CustomFeedConfig {
            id: id.clone(),
            name,
            url,
            enabled: true,
            last_fetched: None,
        });

        Ok(id)
    }

    /// Update the name and URL of an existing custom feed
    pub fn update_custom_feed(&mut self, id: &str, name: &str, url: &str) -> Result<(), String> {
        let (name, url) = Self::validate_custom_feed_input(name, url)?;
        self.ensure_custom_feed_name_unique(&name, Some(id))?;

        let feed = self
            .custom_feeds
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| format!("Custom feed not found: {}", id))?;
        feed.name = name;
        feed.url = url;
        Ok(())
    }

    /// Remove a custom feed, returning it if it existed
    pub fn remove_custom_feed(&mut self, id: &str) -> Option<CustomFeedConfig> {
        let index = self.custom_feeds.iter().position(|f| f.id == id)?;
        Some(self.custom_feeds.remove(index))
    }

    fn validate_custom_feed_input(name: &str, url: &str) -> Result<(String, String), String> {
        let name = name.trim();
        let url = url.trim();

        if name.is_empty() {
            return Err("Feed name cannot be empty".to_string());
        }
        if !url.starts_with("http://") && !url.starts_with("https://") {
            return Err("Feed URL must start with http:// or https://".to_string());
        }

        Ok((name.to_string(), url.to_string()))
    }

    fn ensure_custom_feed_name_unique(
        &self,
        name: &str,
        exclude_id: Option<&str>,
    ) -> Result<(), String> {
        let duplicate = self
            .custom_feeds
            .iter()
            .any(|f| Some(f.id.as_str()) != exclude_id && f.name.eq_ignore_ascii_case(name));

        if duplicate {
            Err("A feed with this name already exists".to_string())
        } else {
            Ok(())
        }
    }

    /// Convert to a flat key-value map for database storage
    pub fn to_key_value_map(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();

        map.insert(
            "auto_refresh.enabled".to_string(),
            self.auto_refresh.enabled.to_string(),
        );
        map.insert(
            "auto_refresh.interval_minutes".to_string(),
            self.auto_refresh.interval_minutes.to_string(),
        );
        map.insert(
            "ui.show_notifications".to_string(),
            self.ui.show_notifications.to_string(),
        );

        map
    }

    /// Create from a flat key-value map from database
    pub fn from_key_value_map(map: &HashMap<String, String>) -> Result<Self, String> {
        let mut config = AppConfig::default();

        // Parse auto-refresh settings
        if let Some(enabled_str) = map.get("auto_refresh.enabled") {
            config.auto_refresh.enabled = enabled_str
                .parse()
                .map_err(|_| "Invalid auto_refresh.enabled value")?;
        }

        if let Some(interval_str) = map.get("auto_refresh.interval_minutes") {
            config.auto_refresh.interval_minutes = interval_str
                .parse()
                .map_err(|_| "Invalid auto_refresh.interval_minutes value")?;
        }

        // Parse UI settings
        if let Some(notifications_str) = map.get("ui.show_notifications") {
            config.ui.show_notifications = notifications_str
                .parse()
                .map_err(|_| "Invalid ui.show_notifications value")?;
        }

        config.validate()?;
        Ok(config)
    }
}

/// Lowercase a name and collapse anything that is not alphanumeric into single underscores
fn slugify(name: &str) -> String {
    let slug = name
        .to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("_");

    if slug.is_empty() {
        "feed".to_string()
    } else {
        slug
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_custom_feed_generates_safe_unique_ids() {
        let mut config = AppConfig::default();

        let id = config
            .add_custom_feed("  Bob's \"Daily\" News ", " https://example.com/rss ")
            .unwrap();
        assert_eq!(id, "custom_bob_s_daily_news");
        assert_eq!(config.custom_feeds[0].name, "Bob's \"Daily\" News");
        assert_eq!(config.custom_feeds[0].url, "https://example.com/rss");

        // Different name that slugifies to the same ID still gets a unique ID
        let id2 = config
            .add_custom_feed("Bob s Daily News", "https://example.com/other")
            .unwrap();
        assert_eq!(id2, "custom_bob_s_daily_news_2");
    }

    #[test]
    fn test_add_custom_feed_rejects_invalid_input() {
        let mut config = AppConfig::default();

        assert!(config
            .add_custom_feed("   ", "https://example.com")
            .is_err());
        assert!(config.add_custom_feed("Feed", "example.com/rss").is_err());

        config
            .add_custom_feed("Feed", "https://example.com")
            .unwrap();
        let duplicate = config.add_custom_feed("FEED", "https://example.com/2");
        assert!(duplicate.unwrap_err().contains("already exists"));
    }

    #[test]
    fn test_update_custom_feed() {
        let mut config = AppConfig::default();
        let id = config
            .add_custom_feed("One", "https://one.example")
            .unwrap();
        config
            .add_custom_feed("Two", "https://two.example")
            .unwrap();

        // Keeping its own name is fine, taking another feed's name is not
        config
            .update_custom_feed(&id, "one", "https://one.example/rss")
            .unwrap();
        assert_eq!(config.custom_feeds[0].url, "https://one.example/rss");
        assert!(config
            .update_custom_feed(&id, "Two", "https://one.example")
            .is_err());
        assert!(config
            .update_custom_feed("missing", "Three", "https://x.example")
            .is_err());
    }

    #[test]
    fn test_remove_custom_feed() {
        let mut config = AppConfig::default();
        let id = config
            .add_custom_feed("One", "https://one.example")
            .unwrap();

        let removed = config.remove_custom_feed(&id).unwrap();
        assert_eq!(removed.name, "One");
        assert!(config.custom_feeds.is_empty());
        assert!(config.remove_custom_feed(&id).is_none());
    }
}
