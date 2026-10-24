use crate::feed_aggregator::FeedAggregator;
use crate::models::AppConfig;
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
        // Keep the in-memory copy in sync, otherwise the next save writes the old config back
        *state.config.write().unwrap() = AppConfig::default();
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
    state.modify_config(|config| config.add_custom_feed(&name, &url))
}

#[tauri::command]
pub async fn update_custom_feed(
    id: String,
    name: String,
    url: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.modify_config(|config| config.update_custom_feed(&id, &name, &url))
}

#[tauri::command]
pub async fn delete_custom_feed(id: String, state: State<'_, AppState>) -> Result<(), String> {
    // Work out which cached articles belong to the feed while the config still describes it
    let stale_source_id = state.modify_config(|config| {
        let Some(removed) = config.remove_custom_feed(&id) else {
            log::warn!("Custom feed '{}' not found, nothing to delete", id);
            return Ok(None);
        };

        let source_id = FeedAggregator::custom_feed_source_id(&removed.name);
        let still_used = config.feed_sources.iter().any(|s| s.id == source_id)
            || config
                .custom_feeds
                .iter()
                .any(|f| FeedAggregator::custom_feed_source_id(&f.name) == source_id);

        Ok((!still_used).then_some(source_id))
    })?;

    // Drop the feed's articles now rather than leaving them on screen until the next refresh
    if let Some(source_id) = stale_source_id {
        state
            .article_cache
            .write()
            .unwrap()
            .retain(|article| article.source_id != source_id);
    }

    Ok(())
}

#[tauri::command]
pub async fn toggle_custom_feed(
    id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    state.modify_config(|config| {
        let feed = config
            .custom_feeds
            .iter_mut()
            .find(|f| f.id == id)
            .ok_or_else(|| format!("Custom feed not found: {}", id))?;
        feed.enabled = enabled;
        Ok(())
    })
}

#[tauri::command]
pub async fn get_available_topics(source_id: String) -> Result<Vec<String>, String> {
    Ok(FeedAggregator::get_available_topics_for_source(&source_id))
}
