use crate::models::AppConfig;
use crate::services::ConfigurationService;
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
pub async fn sync_configuration(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.sync_configuration()
}
