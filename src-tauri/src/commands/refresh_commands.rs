use crate::feed_aggregator::FeedAggregator;
use crate::models::AppConfig;
use crate::services::AppState;
use serde::{Deserialize, Serialize};
use tauri::State;

/// Response for refresh operations
#[derive(Debug, Serialize)]
pub struct RefreshResponse {
    pub success: bool,
    pub message: String,
    pub new_articles: Option<usize>,
    pub failed_sources: Option<usize>,
    pub duration_ms: Option<u64>,
}

/// Parameters for updating refresh configuration
#[derive(Debug, Deserialize)]
pub struct UpdateRefreshConfigParams {
    pub enabled: bool,
    pub interval_minutes: u32,
}

/// Simple refresh status for commands
#[derive(Debug, Serialize)]
pub struct SimpleRefreshStatus {
    pub is_refreshing: bool,
    pub last_refresh_time: Option<String>,
    pub message: Option<String>,
}

/// Tauri commands for refresh and configuration management

#[tauri::command]
pub async fn refresh_feeds(state: State<'_, AppState>) -> Result<RefreshResponse, String> {
    let start_time = std::time::Instant::now();

    // Get current configuration
    let config = state.get_config();

    // Create feed aggregator and perform refresh
    let aggregator = FeedAggregator::new();

    match aggregator.refresh_all_feeds_with_config(&config).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            let num_articles = result.articles.len();
            let num_successful = result.successful_sources.len();
            let num_failed = result.failed_sources.len();

            // Update the cache
            *state.article_cache.write().unwrap() = result.articles;

            Ok(RefreshResponse {
                success: true,
                message: format!(
                    "Successfully fetched {} articles from {} sources",
                    num_articles, num_successful
                ),
                new_articles: Some(num_articles),
                failed_sources: Some(num_failed),
                duration_ms: Some(duration.as_millis() as u64),
            })
        }
        Err(e) => Ok(RefreshResponse {
            success: false,
            message: format!("Refresh failed: {}", e),
            new_articles: None,
            failed_sources: None,
            duration_ms: Some(start_time.elapsed().as_millis() as u64),
        }),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetArticlesParams {
    pub source_id: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ArticlesResponse {
    pub articles: Vec<crate::models::Article>,
    pub total_count: u32,
    pub has_more: bool,
}

#[tauri::command]
pub async fn get_articles(
    params: GetArticlesParams,
    state: State<'_, AppState>,
) -> Result<ArticlesResponse, String> {
    let cache = state.article_cache.read().unwrap();
    let mut all_articles = cache.clone();

    // Filter by source if specified
    if let Some(source_id) = &params.source_id {
        all_articles.retain(|article| article.source_id == *source_id);
    }

    let total_count = all_articles.len() as u32;

    log::info!("Returning {} cached articles", total_count);

    Ok(ArticlesResponse {
        articles: all_articles,
        total_count,
        has_more: false, // Always false since we return everything
    })
}

#[tauri::command]
pub async fn get_refresh_status(
    _state: State<'_, AppState>,
) -> Result<SimpleRefreshStatus, String> {
    Ok(SimpleRefreshStatus {
        is_refreshing: false,
        last_refresh_time: None,
        message: None,
    })
}

#[tauri::command]
pub async fn get_refresh_progress(
    state: State<'_, AppState>,
) -> Result<RefreshProgressInfo, String> {
    use crate::feed_aggregator::FeedAggregator;

    let config = state.get_config();

    let sources = FeedAggregator::get_available_sources_from_config(&config);
    let enabled_sources = sources.iter().filter(|s| s.enabled).count() as u32;

    let total_articles = state.article_cache.read().unwrap().len() as u32;

    Ok(RefreshProgressInfo {
        total_sources: sources.len() as u32,
        enabled_sources,
        total_articles,
        last_refresh: None,
    })
}

// Configuration management commands

#[tauri::command]
pub async fn get_config(state: State<'_, AppState>) -> Result<AppConfig, String> {
    Ok(state.get_config())
}

#[tauri::command]
pub async fn update_config(config: AppConfig, state: State<'_, AppState>) -> Result<(), String> {
    state.update_config(config)
}

#[tauri::command]
pub async fn update_refresh_config(
    params: UpdateRefreshConfigParams,
    state: State<'_, AppState>,
) -> Result<(), String> {
    let mut config = state.get_config();

    // Validate interval
    if params.interval_minutes < 5 {
        return Err("Refresh interval must be at least 5 minutes".to_string());
    }

    if params.interval_minutes > 1440 {
        return Err("Refresh interval cannot exceed 24 hours".to_string());
    }

    // Update refresh settings
    config.auto_refresh.enabled = params.enabled;
    config.auto_refresh.interval_minutes = params.interval_minutes;

    state.update_config(config)
}

#[tauri::command]
pub async fn reset_config(state: State<'_, AppState>) -> Result<(), String> {
    let default_config = AppConfig::default();
    state.update_config(default_config)
}

#[tauri::command]
pub async fn export_config(state: State<'_, AppState>) -> Result<String, String> {
    let backup_path = state.config_manager.backup_config()?;
    Ok(backup_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_config_info(state: State<'_, AppState>) -> Result<ConfigInfo, String> {
    use crate::feed_aggregator::FeedAggregator;

    let config = state.get_config();
    let config_dir = state
        .config_manager
        .get_config_dir()
        .to_string_lossy()
        .to_string();

    let sources = FeedAggregator::get_available_sources_from_config(&config);
    let enabled_sources = sources.iter().filter(|s| s.enabled).count();

    Ok(ConfigInfo {
        config_directory: config_dir,
        auto_refresh_enabled: config.auto_refresh.enabled,
        auto_refresh_interval: config.auto_refresh.interval_minutes,
        total_feed_sources: sources.len(),
        enabled_feed_sources: enabled_sources,
    })
}

#[tauri::command]
pub async fn get_feed_sources(
    state: State<'_, AppState>,
) -> Result<Vec<crate::feed_aggregator::NewsSource>, String> {
    use crate::feed_aggregator::FeedAggregator;

    let config = state.get_config();

    log::info!(
        "Getting feed sources from config, found {} sources",
        config.feed_sources.len()
    );

    let sources = FeedAggregator::get_available_sources_from_config(&config);
    Ok(sources)
}

/// Information about refresh progress
#[derive(Debug, Serialize)]
pub struct RefreshProgressInfo {
    pub total_sources: u32,
    pub enabled_sources: u32,
    pub total_articles: u32,
    pub last_refresh: Option<String>,
}

/// Information about the current configuration
#[derive(Debug, Serialize)]
pub struct ConfigInfo {
    pub config_directory: String,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_interval: u32,
    pub total_feed_sources: usize,
    pub enabled_feed_sources: usize,
}

#[tauri::command]
pub async fn update_feed_source_enabled(
    source_id: String,
    enabled: bool,
    state: State<'_, AppState>,
) -> Result<(), String> {
    log::info!(
        "Updating feed source '{}' to enabled={}",
        source_id,
        enabled
    );

    let mut config = state.get_config();

    log::info!(
        "Current config has {} feed sources",
        config.feed_sources.len()
    );

    // Find and update the feed source
    if let Some(source) = config.feed_sources.iter_mut().find(|s| s.id == source_id) {
        log::info!(
            "Found source '{}', updating enabled from {} to {}",
            source.name,
            source.enabled,
            enabled
        );
        source.enabled = enabled;
        state.update_config(config)?;
        log::info!("Successfully updated feed source '{}'", source_id);
        Ok(())
    } else {
        let available_sources: Vec<String> =
            config.feed_sources.iter().map(|s| s.id.clone()).collect();
        log::error!(
            "Feed source '{}' not found. Available sources: {:?}",
            source_id,
            available_sources
        );
        Err(format!("Feed source '{}' not found", source_id))
    }
}
