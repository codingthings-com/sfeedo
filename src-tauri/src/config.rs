use crate::models::{AppConfig, FeedSource};
use serde_json;
use std::fs;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};

/// Configuration manager for handling JSON file operations
pub struct ConfigManager {
    config_dir: PathBuf,
    config_file: PathBuf,
}

impl ConfigManager {
    /// Create a new ConfigManager instance
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_dir = app_handle
            .path()
            .app_config_dir()
            .map_err(|e| format!("Failed to get config directory: {}", e))?;

        let config_file = config_dir.join("config.json");

        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(Self {
            config_dir,
            config_file,
        })
    }

    /// Create a new ConfigManager instance with a custom path (for testing)
    pub fn new_with_path(config_dir: &Path) -> Result<Self, String> {
        let config_file = config_dir.join("config.json");

        // Ensure config directory exists
        if !config_dir.exists() {
            fs::create_dir_all(config_dir)
                .map_err(|e| format!("Failed to create config directory: {}", e))?;
        }

        Ok(Self {
            config_dir: config_dir.to_path_buf(),
            config_file,
        })
    }

    /// Load configuration from JSON file
    pub fn load_config(&self) -> Result<AppConfig, String> {
        if !self.config_file.exists() {
            // Create default configuration if file doesn't exist
            let default_config = AppConfig::default();
            self.save_config(&default_config)?;
            return Ok(default_config);
        }

        let content = fs::read_to_string(&self.config_file)
            .map_err(|e| format!("Failed to read config file: {}", e))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse config JSON: {}", e))?;

        // Validate the loaded configuration
        config.validate()?;

        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn save_config(&self, config: &AppConfig) -> Result<(), String> {
        // Validate configuration before saving
        config.validate()?;

        let json_content = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize config to JSON: {}", e))?;

        fs::write(&self.config_file, json_content)
            .map_err(|e| format!("Failed to write config file: {}", e))?;

        Ok(())
    }

    /// Load feed sources from JSON file
    pub fn load_feed_sources(&self) -> Result<Vec<FeedSource>, String> {
        let feed_sources_file = self.config_dir.join("feed_sources.json");

        if !feed_sources_file.exists() {
            // Return empty list if file doesn't exist
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(&feed_sources_file)
            .map_err(|e| format!("Failed to read feed sources file: {}", e))?;

        let feed_sources: Vec<FeedSource> = serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse feed sources JSON: {}", e))?;

        // Validate all feed sources
        for feed_source in &feed_sources {
            feed_source.validate()?;
        }

        Ok(feed_sources)
    }

    /// Save feed sources to JSON file
    pub fn save_feed_sources(&self, feed_sources: &[FeedSource]) -> Result<(), String> {
        // Validate all feed sources before saving
        for feed_source in feed_sources {
            feed_source.validate()?;
        }

        let feed_sources_file = self.config_dir.join("feed_sources.json");

        let json_content = serde_json::to_string_pretty(feed_sources)
            .map_err(|e| format!("Failed to serialize feed sources to JSON: {}", e))?;

        fs::write(&feed_sources_file, json_content)
            .map_err(|e| format!("Failed to write feed sources file: {}", e))?;

        Ok(())
    }

    /// Get the configuration directory path
    pub fn get_config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get the configuration file path
    pub fn get_config_file(&self) -> &Path {
        &self.config_file
    }

    /// Check if configuration file exists
    pub fn config_exists(&self) -> bool {
        self.config_file.exists()
    }

    /// Reset configuration to defaults
    pub fn reset_to_defaults(&self) -> Result<(), String> {
        let default_config = AppConfig::default();
        self.save_config(&default_config)?;
        
        // Also reset feed sources to empty
        self.save_feed_sources(&[])?;
        
        Ok(())
    }

    /// Backup current configuration
    pub fn backup_config(&self) -> Result<PathBuf, String> {
        if !self.config_file.exists() {
            return Err("No configuration file to backup".to_string());
        }

        let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
        let backup_file = self.config_dir.join(format!("config_backup_{}.json", timestamp));

        fs::copy(&self.config_file, &backup_file)
            .map_err(|e| format!("Failed to backup config file: {}", e))?;

        Ok(backup_file)
    }

    /// Restore configuration from backup
    pub fn restore_from_backup(&self, backup_path: &Path) -> Result<(), String> {
        if !backup_path.exists() {
            return Err("Backup file does not exist".to_string());
        }

        // Validate the backup file by trying to load it
        let content = fs::read_to_string(backup_path)
            .map_err(|e| format!("Failed to read backup file: {}", e))?;

        let config: AppConfig = serde_json::from_str(&content)
            .map_err(|e| format!("Invalid backup file format: {}", e))?;

        config.validate()?;

        // Copy backup to current config
        fs::copy(backup_path, &self.config_file)
            .map_err(|e| format!("Failed to restore from backup: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

    fn create_test_config_manager() -> (ConfigManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let config_manager = ConfigManager {
            config_dir: temp_dir.path().to_path_buf(),
            config_file: temp_dir.path().join("config.json"),
        };
        (config_manager, temp_dir)
    }

    #[test]
    fn test_load_config_creates_default_when_missing() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let config = config_manager.load_config().unwrap();
        assert_eq!(config.auto_refresh.enabled, true);
        assert_eq!(config.auto_refresh.interval_minutes, 30);
        assert!(config_manager.config_file.exists());
    }

    #[test]
    fn test_save_and_load_config() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let mut config = AppConfig::default();
        config.auto_refresh.interval_minutes = 60;
        config.ui.articles_per_page = 25;

        config_manager.save_config(&config).unwrap();
        let loaded_config = config_manager.load_config().unwrap();

        assert_eq!(loaded_config.auto_refresh.interval_minutes, 60);
        assert_eq!(loaded_config.ui.articles_per_page, 25);
    }

    #[test]
    fn test_config_validation() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let mut invalid_config = AppConfig::default();
        invalid_config.auto_refresh.interval_minutes = 2; // Too low

        let result = config_manager.save_config(&invalid_config);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("at least 5 minutes"));
    }

    #[test]
    fn test_feed_sources_operations() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        let feed_sources = vec![
            FeedSource::new("test1".to_string(), "Test Feed 1".to_string(), "https://example.com/feed1".to_string()),
            FeedSource::new("test2".to_string(), "Test Feed 2".to_string(), "https://example.com/feed2".to_string()),
        ];

        config_manager.save_feed_sources(&feed_sources).unwrap();
        let loaded_sources = config_manager.load_feed_sources().unwrap();

        assert_eq!(loaded_sources.len(), 2);
        assert_eq!(loaded_sources[0].id, "test1");
        assert_eq!(loaded_sources[1].id, "test2");
    }

    #[test]
    fn test_backup_and_restore() {
        let (config_manager, _temp_dir) = create_test_config_manager();
        
        // Create and save a config
        let mut config = AppConfig::default();
        config.auto_refresh.interval_minutes = 120;
        config_manager.save_config(&config).unwrap();

        // Backup the config
        let backup_path = config_manager.backup_config().unwrap();
        assert!(backup_path.exists());

        // Modify the config
        config.auto_refresh.interval_minutes = 240;
        config_manager.save_config(&config).unwrap();

        // Restore from backup
        config_manager.restore_from_backup(&backup_path).unwrap();
        let restored_config = config_manager.load_config().unwrap();

        assert_eq!(restored_config.auto_refresh.interval_minutes, 120);
    }
}