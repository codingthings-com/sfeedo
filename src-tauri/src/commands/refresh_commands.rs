use crate::feed_aggregator::FeedAggregator;
use crate::models::AppConfig;
use crate::services::ConfigurationService;
use serde::{Deserialize, Serialize};
use tauri::AppHandle;

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
pub async fn refresh_feeds(app_handle: AppHandle) -> Result<RefreshResponse, String> {
    let start_time = std::time::Instant::now();

    // Get current configuration
    let config_service = ConfigurationService::new(&app_handle)?;
    let config = config_service.get_app_config()?;

    // Create feed aggregator and perform refresh
    let mut aggregator = FeedAggregator::new();

    match aggregator.refresh_all_feeds_with_config(&config).await {
        Ok(result) => {
            let duration = start_time.elapsed();
            Ok(RefreshResponse {
                success: true,
                message: format!(
                    "Successfully fetched {} articles from {} sources",
                    result.articles.len(),
                    result.successful_sources.len()
                ),
                new_articles: Some(result.articles.len()),
                failed_sources: Some(result.failed_sources.len()),
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
    app_handle: AppHandle,
) -> Result<ArticlesResponse, String> {
    // Get current configuration
    let config_service = ConfigurationService::new(&app_handle)?;
    let config = config_service.get_app_config()?;

    // Create feed aggregator and fetch fresh articles
    let mut aggregator = FeedAggregator::new();

    match aggregator.refresh_all_feeds_with_config(&config).await {
        Ok(result) => {
            log::info!("Fetched {} articles for frontend", result.articles.len());

            // Apply filtering and pagination based on params
            let mut all_articles = result.articles;

            // Filter by source if specified
            if let Some(source_id) = &params.source_id {
                all_articles.retain(|article| article.source_id == *source_id);
            }

            let total_count = all_articles.len() as u32;

            // Return all articles - no pagination for desktop app
            log::info!("Returning all {} articles", total_count);

            Ok(ArticlesResponse {
                articles: all_articles,
                total_count,
                has_more: false, // Always false since we return everything
            })
        }
        Err(e) => {
            log::error!("Failed to fetch articles: {}", e);
            Err(format!("Failed to fetch articles: {}", e))
        }
    }
}

#[tauri::command]
pub async fn get_refresh_status(app_handle: AppHandle) -> Result<SimpleRefreshStatus, String> {
    // For now, return a simple status
    // In a full implementation, this would track actual refresh state
    let config_service = ConfigurationService::new(&app_handle)?;
    let config = config_service.get_app_config()?;

    Ok(SimpleRefreshStatus {
        is_refreshing: false,    // Would be tracked by a global state manager
        last_refresh_time: None, // Could be stored in config
        message: if config.auto_refresh.enabled {
            Some(format!(
                "Auto-refresh enabled (every {} minutes)",
                config.auto_refresh.interval_minutes
            ))
        } else {
            Some("Auto-refresh disabled".to_string())
        },
    })
}

#[tauri::command]
pub async fn get_refresh_progress(app_handle: AppHandle) -> Result<RefreshProgressInfo, String> {
    use crate::feed_aggregator::FeedAggregator;

    let config_service = ConfigurationService::new(&app_handle)?;
    let config = config_service.get_app_config()?;
    
    let sources = FeedAggregator::get_available_sources_from_config(&config);
    let enabled_sources = sources.iter().filter(|s| s.enabled).count() as u32;

    Ok(RefreshProgressInfo {
        total_sources: sources.len() as u32,
        enabled_sources,
        total_articles: 0,  // No longer storing articles
        last_refresh: None, // Could be tracked in config
    })
}

// Configuration management commands (these complement the existing config commands)

#[tauri::command]
pub async fn get_config(app_handle: AppHandle) -> Result<AppConfig, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.get_app_config()
}

#[tauri::command]
pub async fn update_config(config: AppConfig, app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.update_app_config(config)
}

#[tauri::command]
pub async fn update_refresh_config(
    params: UpdateRefreshConfigParams,
    app_handle: AppHandle,
) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;

    // Get current config
    let mut config = service.get_app_config()?;

    // Validate interval
    if params.interval_minutes < 5 {
        return Err("Refresh interval must be at least 5 minutes".to_string());
    }

    if params.interval_minutes > 1440 {
        // 24 hours
        return Err("Refresh interval cannot exceed 24 hours".to_string());
    }

    // Update refresh settings
    config.auto_refresh.enabled = params.enabled;
    config.auto_refresh.interval_minutes = params.interval_minutes;

    // Save updated config
    service.update_app_config(config)
}

#[tauri::command]
pub async fn reset_config(app_handle: AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.reset_config_to_defaults()
}

#[tauri::command]
pub async fn export_config(app_handle: AppHandle) -> Result<String, String> {
    let service = ConfigurationService::new(&app_handle)?;
    service.backup_configuration()
}

#[tauri::command]
pub async fn get_config_info(app_handle: AppHandle) -> Result<ConfigInfo, String> {
    use crate::feed_aggregator::FeedAggregator;

    let service = ConfigurationService::new(&app_handle)?;
    let config = service.get_app_config()?;
    let config_dir = service.get_config_directory();

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
    app_handle: AppHandle,
) -> Result<Vec<crate::feed_aggregator::NewsSource>, String> {
    use crate::feed_aggregator::FeedAggregator;

    let config_service = ConfigurationService::new(&app_handle)?;
    let config = config_service.get_app_config()?;
    
    log::info!("Getting feed sources from config, found {} sources", config.feed_sources.len());
    
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
    app_handle: AppHandle,
) -> Result<(), String> {
    log::info!("Updating feed source '{}' to enabled={}", source_id, enabled);
    
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;

    log::info!("Current config has {} feed sources", config.feed_sources.len());

    // Find and update the feed source
    if let Some(source) = config.feed_sources.iter_mut().find(|s| s.id == source_id) {
        log::info!("Found source '{}', updating enabled from {} to {}", source.name, source.enabled, enabled);
        source.enabled = enabled;
        service.update_app_config(config)?;
        log::info!("Successfully updated feed source '{}'", source_id);
        Ok(())
    } else {
        let available_sources: Vec<String> = config.feed_sources.iter().map(|s| s.id.clone()).collect();
        log::error!("Feed source '{}' not found. Available sources: {:?}", source_id, available_sources);
        Err(format!("Feed source '{}' not found", source_id))
    }
}
