use crate::feed_aggregator::FeedAggregator;
use crate::models::CustomFeedConfig;
use crate::services::AppState;
use tauri::State;

/// Tauri commands for configuration management

#[tauri::command]
pub async fn delete_config_file(state: State<'_, AppState>) -> Result<String, String> {
    let config_dir = state.config_manager.get_config_dir();
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
pub async fn get_config_file_path(state: State<'_, AppState>) -> Result<String, String> {
    Ok(state
        .config_manager
        .get_config_file()
        .to_string_lossy()
        .to_string())
}

#[tauri::command]
pub async fn update_source_topics(
    source_id: String,
    enabled_topics: Vec<String>,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Updating topics for source '{}': {:?}",
        source_id,
        enabled_topics
    );

    let mut config = state.get_config();

    // Find and update the source
    if let Some(source) = config.feed_sources.iter_mut().find(|s| s.id == source_id) {
        log::info!(
            "Found source '{}', updating topics from {:?} to {:?}",
            source.name,
            source.enabled_topics,
            enabled_topics
        );
        source.enabled_topics = enabled_topics;
        state.update_config(config)?;
        log::info!("Successfully updated topics for source '{}'", source_id);
        Ok(())
    } else {
        log::error!("Source '{}' not found", source_id);
        Err(format!("Source not found: {}", source_id))
    }
}

#[tauri::command]
pub async fn add_custom_feed(
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut config = state.get_config();

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
    state.update_config(config)?;

    Ok(id)
}

#[tauri::command]
pub async fn update_custom_feed(
    id: String,
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.get_config();

    if let Some(feed) = config.custom_feeds.iter_mut().find(|f| f.id == id) {
        feed.name = name;
        feed.url = url;
        state.update_config(config)?;
        Ok(())
    } else {
        Err(format!("Custom feed not found: {}", id))
    }
}

#[tauri::command]
pub async fn delete_custom_feed(id: String, state: State<'_, AppState>) -> Result<(), String> {
    let mut config = state.get_config();

    config.custom_feeds.retain(|f| f.id != id);
    state.update_config(config)?;

    Ok(())
}

#[tauri::command]
pub async fn toggle_custom_feed(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.get_config();

    if let Some(feed) = config.custom_feeds.iter_mut().find(|f| f.id == id) {
        feed.enabled = enabled;
        state.update_config(config)?;
        Ok(())
    } else {
        Err(format!("Custom feed not found: {}", id))
    }
}

#[tauri::command]
pub async fn get_available_topics(source_id: String) -> Result<Vec<String>, String> {
    Ok(FeedAggregator::get_available_topics_for_source(&source_id))
}
