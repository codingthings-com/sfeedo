pub mod commands;
pub mod config;
pub mod feed_aggregator;
pub mod models;
pub mod refresh_manager;
pub mod services;

use commands::*;
use services::ConfigurationService;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_app_config,
            update_app_config,
            reset_config_to_defaults,
            backup_configuration,
            get_config_directory,
            sync_configuration,
            initialize_default_feed_sources,
            refresh_feeds,
            get_articles,
            get_refresh_status,
            get_refresh_progress,
            get_config,
            update_config,
            update_refresh_config,
            reset_config,
            export_config,
            get_config_info,
            get_feed_sources
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }

            // Initialize configuration service and default feed sources
            let config_service = ConfigurationService::new(&app.handle()).map_err(|e| {
                log::error!("Failed to initialize configuration service: {}", e);
                e
            })?;

            // Initialize default feed sources if none exist
            config_service
                .initialize_default_feed_sources()
                .map_err(|e| {
                    log::error!("Failed to initialize default feed sources: {}", e);
                    e
                })?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
