mod database;
mod models;
mod config;
mod feed_manager;
mod feed_aggregator;
mod refresh_manager;
mod services;
mod commands;

use database::{DatabaseConnection, run_migrations};
use services::ConfigurationService;
use commands::*;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      get_app_config,
      update_app_config,
      reset_config_to_defaults,
      add_feed_source,
      remove_feed_source,
      update_feed_source,
      toggle_feed_source,
      get_all_feed_sources,
      get_enabled_feed_sources,
      get_feed_source_by_id,
      get_feed_source_stats,
      validate_feed_url,
      backup_configuration,
      get_config_directory,
      sync_configuration,
      initialize_default_feed_sources
    ])
    .setup(|app| {
      if cfg!(debug_assertions) {
        app.handle().plugin(
          tauri_plugin_log::Builder::default()
            .level(log::LevelFilter::Info)
            .build(),
        )?;
      }

      // Initialize database
      let db = DatabaseConnection::new(&app.handle()).map_err(|e| {
        log::error!("Failed to initialize database: {}", e);
        e
      })?;

      // Run migrations
      run_migrations(&db).map_err(|e| {
        log::error!("Failed to run database migrations: {}", e);
        e
      })?;

      // Store database connection in app state
      app.manage(db);

      // Initialize configuration service and default feed sources
      let config_service = ConfigurationService::new(&app.handle()).map_err(|e| {
        log::error!("Failed to initialize configuration service: {}", e);
        e
      })?;

      // Sync configuration between JSON and database
      config_service.sync_configuration().map_err(|e| {
        log::error!("Failed to sync configuration: {}", e);
        e
      })?;

      // Initialize default feed sources if none exist
      config_service.initialize_default_feed_sources().map_err(|e| {
        log::error!("Failed to initialize default feed sources: {}", e);
        e
      })?;

      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
