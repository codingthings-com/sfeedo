use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_refresh: AutoRefreshConfig,
    pub ui: UiConfig,
    pub feed_sources: Vec<FeedSourceConfig>,
}

/// Feed source configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSourceConfig {
    pub id: String,
    pub name: String,
    pub enabled: bool,
    pub url: String,
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

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_refresh: AutoRefreshConfig {
                enabled: true,
                interval_minutes: 5,
            },
            ui: UiConfig {
                show_notifications: true,
            },
            feed_sources: vec![
                FeedSourceConfig {
                    id: "yahoo".to_string(),
                    name: "Yahoo Finance".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "cnbc".to_string(),
                    name: "CNBC Business".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "marketwatch".to_string(),
                    name: "MarketWatch".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "seeking_alpha".to_string(),
                    name: "Seeking Alpha".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "wsj".to_string(),
                    name: "Wall Street Journal".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "nasdaq".to_string(),
                    name: "NASDAQ".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
                FeedSourceConfig {
                    id: "cnn".to_string(),
                    name: "CNN Finance".to_string(),
                    enabled: true,
                    url: "Built-in scraper".to_string(),
                    last_fetched: None,
                },
            ],
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
