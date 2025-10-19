use crate::models::{AppConfig, FeedSource};
use crate::services::ConfigurationService;
use crate::feed_manager::FeedSourceStats;
use tauri::AppHandle;

/// Tauri commands for configuration management

#[tauri::command]
pub async fn get_app_config(app_handle: AppHandle) -> Result<AppConfig, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_app_config()
}

#[tauri::command]
pub async fn update_app_config(config: AppConfig, app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.update_app_config(config)
}

#[tauri::command]
pub async fn reset_config_to_defaults(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.reset_config_to_defaults()
}

#[tauri::command]
pub async fn add_feed_source(name: String, url: String, app_handle: AppHandle) -> Result<FeedSource, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.add_feed_source(name, url)
}

#[tauri::command]
pub async fn remove_feed_source(id: String, app_handle: AppHandle) -> Result<bool, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.remove_feed_source(&id)
}

#[tauri::command]
pub async fn update_feed_source(feed_source: FeedSource, app_handle: AppHandle) -> Result<bool, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.update_feed_source(feed_source)
}

#[tauri::command]
pub async fn toggle_feed_source(id: String, enabled: bool, app_handle: AppHandle) -> Result<bool, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.toggle_feed_source(&id, enabled)
}

#[tauri::command]
pub async fn get_all_feed_sources(app_handle: AppHandle) -> Result<Vec<FeedSource>, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_all_feed_sources()
}

#[tauri::command]
pub async fn get_enabled_feed_sources(app_handle: AppHandle) -> Result<Vec<FeedSource>, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_enabled_feed_sources()
}

#[tauri::command]
pub async fn get_feed_source_by_id(id: String, app_handle: AppHandle) -> Result<Option<FeedSource>, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_feed_source_by_id(&id)
}

#[tauri::command]
pub async fn get_feed_source_stats(app_handle: AppHandle) -> Result<FeedSourceStats, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_feed_source_stats()
}

#[tauri::command]
pub async fn validate_feed_url(url: String, app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.validate_feed_url(&url)
}

#[tauri::command]
pub async fn backup_configuration(app_handle: AppHandle) -> Result<String, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.backup_configuration()
}

#[tauri::command]
pub async fn get_config_directory(app_handle: AppHandle) -> Result<String, String> {
    let service = ConfigurationService::new(&app_handle)?;
    Ok(service.get_config_directory())
}

#[tauri::command]
pub async fn sync_configuration(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.sync_configuration()
}

#[tauri::command]
pub async fn initialize_default_feed_sources(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.initialize_default_feed_sources()
}