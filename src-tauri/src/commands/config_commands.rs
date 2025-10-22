use crate::models::{AppConfig, CustomFeedConfig};
use crate::services::ConfigurationService;
use crate::feed_aggregator::FeedAggregator;
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
pub async fn delete_config_file(app_handle: AppHandle) -> Result<String, String> {
    let service = ConfigurationService::new(&app_handle)?;
    let config_dir = service.get_config_directory();
    let config_file = std::path::Path::new(&config_dir).join("config.json");
    
    if config_file.exists() {
        std::fs::remove_file(&config_file)
            .map_err(|e| format!("Failed to delete config file: {}", e))?;
        Ok(format!("Config file deleted: {}", config_file.display()))
    } else {
        Ok("Config file does not exist".to_string())
    }
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
pub async fn get_config_file_path(app_handle: AppHandle) -> Result<String, String> {
    use crate::config::ConfigManager;
    let config_manager = ConfigManager::new(&app_handle)?;
    Ok(config_manager.get_config_file().to_string_lossy().to_string())
}

#[tauri::command]
pub async fn sync_configuration(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.sync_configuration()
}

#[tauri::command]
pub async fn update_source_topics(
    source_id: String,
    enabled_topics: Vec<String>,
    app_handle: AppHandle,
) -> Result<(), String> {
    log::info!("Updating topics for source '{}': {:?}", source_id, enabled_topics);
    
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;
    
    // Find and update the source
    if let Some(source) = config.feed_sources.iter_mut().find(|s| s.id == source_id) {
        log::info!("Found source '{}', updating topics from {:?} to {:?}", 
                   source.name, source.enabled_topics, enabled_topics);
        source.enabled_topics = enabled_topics;
        service.update_app_config(config)?;
        log::info!("Successfully updated topics for source '{}'", source_id);
        Ok(())
    } else {
        log::error!("Source '{}' not found", source_id);
        Err(format!("Source not found: {}", source_id))
    }
}

#[tauri::command]
pub async fn add_custom_feed(
    app_handle: AppHandle,
    name: String,
    url: String,
) -> Result<String, String> {
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;
    
    // Generate ID from name
    let id = format!("custom_{}", name.to_lowercase().replace(" ", "_"));
    
    // Check if ID already exists
    if config.custom_feeds.iter().any(|f| f.id == id) {
        return Err("A feed with this name already exists".to_string());
    }
    
    let custom_feed = CustomFeedConfig {
        id: id.clone(),
        name,
        url,
        enabled: true,
        last_fetched: None,
    };
    
    config.custom_feeds.push(custom_feed);
    service.update_app_config(config)?;
    
    Ok(id)
}

#[tauri::command]
pub async fn update_custom_feed(
    app_handle: AppHandle,
    id: String,
    name: String,
    url: String,
) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;
    
    if let Some(feed) = config.custom_feeds.iter_mut().find(|f| f.id == id) {
        feed.name = name;
        feed.url = url;
        service.update_app_config(config)?;
        Ok(())
    } else {
        Err(format!("Custom feed not found: {}", id))
    }
}

#[tauri::command]
pub async fn delete_custom_feed(
    app_handle: AppHandle,
    id: String,
) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;
    
    config.custom_feeds.retain(|f| f.id != id);
    service.update_app_config(config)?;
    
    Ok(())
}

#[tauri::command]
pub async fn toggle_custom_feed(
    app_handle: AppHandle,
    id: String,
    enabled: bool,
) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;
    
    if let Some(feed) = config.custom_feeds.iter_mut().find(|f| f.id == id) {
        feed.enabled = enabled;
        service.update_app_config(config)?;
        Ok(())
    } else {
        Err(format!("Custom feed not found: {}", id))
    }
}

#[tauri::command]
pub async fn get_available_topics(source_id: String) -> Result<Vec<String>, String> {
    Ok(FeedAggregator::get_available_topics_for_source(&source_id))
}
