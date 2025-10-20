use crate::config::ConfigManager;
use crate::models::AppConfig;
use tauri::AppHandle;

/// Comprehensive configuration service that manages both app config and feed sources
pub struct ConfigurationService {
    config_manager: ConfigManager,
}

impl ConfigurationService {
    /// Create a new ConfigurationService instance
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_manager = ConfigManager::new(app_handle)?;

        Ok(Self { config_manager })
    }

    /// Get the current application configuration
    pub fn get_app_config(&self) -> Result<AppConfig, String> {
        self.config_manager.load_config()
    }

    /// Update the application configuration
    pub fn update_app_config(&self, config: AppConfig) -> Result<(), String> {
        // Validate configuration
        config.validate()?;

        // Save to JSON file
        self.config_manager.save_config(&config)?;

        Ok(())
    }

    /// Reset configuration to defaults
    pub fn reset_config_to_defaults(&self) -> Result<(), String> {
        // Reset JSON files
        self.config_manager.reset_to_defaults()?;

        Ok(())
    }

    /// Backup current configuration
    pub fn backup_configuration(&self) -> Result<String, String> {
        let backup_path = self.config_manager.backup_config()?;
        Ok(backup_path.to_string_lossy().to_string())
    }

    /// Get configuration directory path
    pub fn get_config_directory(&self) -> String {
        self.config_manager
            .get_config_dir()
            .to_string_lossy()
            .to_string()
    }

    /// Synchronize configuration (now just validates JSON files exist)
    pub fn sync_configuration(&self) -> Result<(), String> {
        // Just ensure config is loadable
        let _config = self.config_manager.load_config()?;

        log::info!("Configuration synchronized");
        Ok(())
    }
}
