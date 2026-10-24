use crate::config::ConfigManager;
use crate::models::{AppConfig, Article};
use std::sync::RwLock;
use tauri::AppHandle;

/// Application state managed by Tauri
pub struct AppState {
    pub config_manager: ConfigManager,
    pub config: RwLock<AppConfig>,
    pub article_cache: RwLock<Vec<Article>>,
}

impl AppState {
    /// Create a new AppState instance
    pub fn new(app_handle: &AppHandle) -> Result<Self, String> {
        let config_manager = ConfigManager::new(app_handle)?;
        let config = config_manager.load_config()?;

        Ok(Self {
            config_manager,
            config: RwLock::new(config),
            article_cache: RwLock::new(Vec::new()),
        })
    }

    /// Get a clone of the current configuration
    pub fn get_config(&self) -> AppConfig {
        self.config.read().unwrap().clone()
    }

    /// Update the configuration and save to disk
    pub fn update_config(&self, new_config: AppConfig) -> Result<(), String> {
        self.modify_config(|config| {
            *config = new_config;
            Ok(())
        })
    }

    /// Apply a change to the configuration and save it to disk.
    ///
    /// The write lock is held for the whole read-modify-write so concurrent
    /// commands (e.g. a window-state save racing a feed delete) cannot
    /// overwrite each other's changes. Nothing is applied if `change` fails.
    pub fn modify_config<T>(
        &self,
        change: impl FnOnce(&mut AppConfig) -> Result<T, String>,
    ) -> Result<T, String> {
        let mut current = self.config.write().unwrap();
        let mut new_config = current.clone();
        let result = change(&mut new_config)?;
        new_config.validate()?;
        self.config_manager.save_config(&new_config)?;
        *current = new_config;
        Ok(result)
    }
}
