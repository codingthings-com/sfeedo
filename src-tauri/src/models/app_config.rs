use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Application configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_refresh: AutoRefreshConfig,
    pub ui: UiConfig,
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
    pub theme: Theme,
    pub articles_per_page: u32,
    pub show_notifications: bool,
}

/// Theme options
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    Light,
    Dark,
    System,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_refresh: AutoRefreshConfig {
                enabled: true,
                interval_minutes: 5,
            },
            ui: UiConfig {
                theme: Theme::System,
                articles_per_page: 50,
                show_notifications: true,
            },
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

        // Validate articles per page (1 to 200)
        if self.ui.articles_per_page < 1 {
            return Err("Articles per page must be at least 1".to_string());
        }

        if self.ui.articles_per_page > 200 {
            return Err("Articles per page cannot exceed 200".to_string());
        }

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
            "ui.theme".to_string(),
            serde_json::to_string(&self.ui.theme).unwrap_or_default(),
        );
        map.insert(
            "ui.articles_per_page".to_string(),
            self.ui.articles_per_page.to_string(),
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
        if let Some(theme_str) = map.get("ui.theme") {
            config.ui.theme =
                serde_json::from_str(theme_str).map_err(|_| "Invalid ui.theme value")?;
        }

        if let Some(articles_str) = map.get("ui.articles_per_page") {
            config.ui.articles_per_page = articles_str
                .parse()
                .map_err(|_| "Invalid ui.articles_per_page value")?;
        }

        if let Some(notifications_str) = map.get("ui.show_notifications") {
            config.ui.show_notifications = notifications_str
                .parse()
                .map_err(|_| "Invalid ui.show_notifications value")?;
        }

        config.validate()?;
        Ok(config)
    }
}
