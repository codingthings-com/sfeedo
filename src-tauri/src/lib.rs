pub mod commands;
pub mod config;
pub mod feed_aggregator;
pub mod models;
pub mod services;

use commands::*;
use services::AppState;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
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
            get_feed_sources,
            update_feed_source_enabled,
            delete_config_file,
            get_config_file_path,
            open_url_in_browser,
            update_source_topics,
            add_custom_feed,
            update_custom_feed,
            delete_custom_feed,
            toggle_custom_feed,
            get_available_topics,
            save_window_state,
            get_window_state
        ])
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Warn)
                        .build(),
                )?;
            }

            // Initialize app state
            let app_state = AppState::new(&app.handle())
                .map_err(|e| {
                    log::error!("Failed to initialize app state: {}", e);
                    e
                })
                .unwrap(); // Or handle properly if we shouldn't crash, but typically we want to crash if we can't initialize config

            app.manage(app_state);

            // Restore window state
            if let Err(e) = commands::window_commands::restore_window_state(&app.handle()) {
                log::warn!("Failed to restore window state: {}", e);
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
