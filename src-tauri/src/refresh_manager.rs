use crate::feed_aggregator::FetchResult;
use crate::services::ConfigurationService;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::time::{interval, Instant};
use tokio::sync::broadcast;
use serde::{Deserialize, Serialize};

/// Status of a refresh operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefreshStatus {
    Idle,
    InProgress { started_at: String, sources_total: usize, sources_completed: usize },
    Completed { duration_ms: u64, new_articles: usize, failed_sources: usize },
    Failed { error: String },
}

/// Progress update during refresh
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshProgress {
    pub status: RefreshStatus,
    pub current_source: Option<String>,
    pub timestamp: String,
}

/// Manages feed refresh operations and scheduling
pub struct RefreshManager {
    config_service: Arc<ConfigurationService>,
    status: Arc<Mutex<RefreshStatus>>,
    progress_sender: broadcast::Sender<RefreshProgress>,
    auto_refresh_handle: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
}

impl RefreshManager {
    /// Create a new RefreshManager instance
    pub fn new(
        config_service: Arc<ConfigurationService>,
    ) -> Self {
        let (progress_sender, _) = broadcast::channel(100);
        
        Self {
            config_service,
            status: Arc::new(Mutex::new(RefreshStatus::Idle)),
            progress_sender,
            auto_refresh_handle: Arc::new(Mutex::new(None)),
        }
    }

    /// Start the auto-refresh scheduler based on current configuration
    pub async fn start_auto_refresh(&self) -> Result<(), String> {
        let config = self.config_service.get_app_config()
            .map_err(|e| format!("Failed to get app config: {}", e))?;

        if !config.auto_refresh.enabled {
            log::info!("Auto-refresh is disabled");
            return Ok(());
        }

        // Stop existing auto-refresh if running
        self.stop_auto_refresh().await;

        let interval_minutes = config.auto_refresh.interval_minutes;
        if interval_minutes < 5 {
            return Err("Auto-refresh interval must be at least 5 minutes".to_string());
        }

        log::info!("Starting auto-refresh with interval: {} minutes", interval_minutes);

        let config_service = Arc::clone(&self.config_service);
        let status = Arc::clone(&self.status);
        let progress_sender = self.progress_sender.clone();

        let handle = tokio::spawn(async move {
            let mut refresh_interval = interval(Duration::from_secs(interval_minutes as u64 * 60));
            
            loop {
                refresh_interval.tick().await;
                
                log::info!("Auto-refresh triggered");
                
                // Check if auto-refresh is still enabled
                if let Ok(current_config) = config_service.get_app_config() {
                    if !current_config.auto_refresh.enabled {
                        log::info!("Auto-refresh disabled, stopping scheduler");
                        break;
                    }
                    
                    // Update interval if it changed
                    if current_config.auto_refresh.interval_minutes != interval_minutes {
                        log::info!("Auto-refresh interval changed, restarting scheduler");
                        break;
                    }
                } else {
                    log::error!("Failed to get app config during auto-refresh");
                    continue;
                }

                // Perform the refresh
                let refresh_manager = RefreshManager {
                    config_service: Arc::clone(&config_service),
                    status: Arc::clone(&status),
                    progress_sender: progress_sender.clone(),
                    auto_refresh_handle: Arc::new(Mutex::new(None)),
                };

                if let Err(e) = refresh_manager.refresh_feeds().await {
                    log::error!("Auto-refresh failed: {}", e);
                }
            }
        });

        *self.auto_refresh_handle.lock().unwrap() = Some(handle);
        Ok(())
    }

    /// Stop the auto-refresh scheduler
    pub async fn stop_auto_refresh(&self) {
        if let Some(handle) = self.auto_refresh_handle.lock().unwrap().take() {
            handle.abort();
            log::info!("Auto-refresh scheduler stopped");
        }
    }

    /// Restart auto-refresh with updated configuration
    pub async fn restart_auto_refresh(&self) -> Result<(), String> {
        self.stop_auto_refresh().await;
        self.start_auto_refresh().await
    }

    /// Perform a manual refresh of all enabled feeds
    pub async fn refresh_feeds(&self) -> Result<FetchResult, String> {
        // Check if already in progress
        {
            let current_status = self.status.lock().unwrap();
            if matches!(*current_status, RefreshStatus::InProgress { .. }) {
                return Err("Refresh already in progress".to_string());
            }
        }

        let start_time = Instant::now();
        let started_at = chrono::Utc::now().to_rfc3339();

        use crate::feed_aggregator::FeedAggregator;
        
        let sources = FeedAggregator::get_available_sources();
        let enabled_sources: Vec<_> = sources.iter().filter(|s| s.enabled).collect();

        if enabled_sources.is_empty() {
            return Err("No enabled news sources found".to_string());
        }

        // Update status to in progress
        {
            let mut status = self.status.lock().unwrap();
            *status = RefreshStatus::InProgress {
                started_at: started_at.clone(),
                sources_total: enabled_sources.len(),
                sources_completed: 0,
            };
        }

        // Send initial progress update
        self.send_progress_update(None).await;

        // Create feed aggregator and perform refresh
        let mut aggregator = FeedAggregator::new();
        
        let result = match aggregator.refresh_all_feeds().await {
            Ok(fetch_result) => {
                let duration = start_time.elapsed();
                
                // Update status to completed
                {
                    let mut status = self.status.lock().unwrap();
                    *status = RefreshStatus::Completed {
                        duration_ms: duration.as_millis() as u64,
                        new_articles: fetch_result.articles.len(),
                        failed_sources: fetch_result.failed_sources.len(),
                    };
                }

                log::info!(
                    "Refresh completed: {} articles fetched, {} failed sources, duration: {:?}",
                    fetch_result.articles.len(),
                    fetch_result.failed_sources.len(),
                    duration
                );

                Ok(fetch_result)
            }
            Err(e) => {
                // Update status to failed
                {
                    let mut status = self.status.lock().unwrap();
                    *status = RefreshStatus::Failed {
                        error: e.to_string(),
                    };
                }

                log::error!("Refresh failed: {}", e);
                Err(e.to_string())
            }
        };

        // Send final progress update
        self.send_progress_update(None).await;

        result
    }

    /// Get the current refresh status
    pub fn get_refresh_status(&self) -> RefreshStatus {
        self.status.lock().unwrap().clone()
    }

    /// Subscribe to refresh progress updates
    pub fn subscribe_to_progress(&self) -> broadcast::Receiver<RefreshProgress> {
        self.progress_sender.subscribe()
    }

    /// Check if auto-refresh is currently running
    pub fn is_auto_refresh_running(&self) -> bool {
        self.auto_refresh_handle.lock().unwrap().is_some()
    }

    /// Get the next scheduled auto-refresh time (if enabled)
    pub async fn get_next_refresh_time(&self) -> Result<Option<chrono::DateTime<chrono::Utc>>, String> {
        let config = self.config_service.get_app_config()
            .map_err(|e| format!("Failed to get app config: {}", e))?;

        if !config.auto_refresh.enabled || !self.is_auto_refresh_running() {
            return Ok(None);
        }

        // Calculate next refresh time based on interval
        let now = chrono::Utc::now();
        let interval_duration = chrono::Duration::minutes(config.auto_refresh.interval_minutes as i64);
        let next_refresh = now + interval_duration;

        Ok(Some(next_refresh))
    }

    /// Send a progress update to subscribers
    async fn send_progress_update(&self, current_source: Option<String>) {
        let status = self.get_refresh_status();
        let progress = RefreshProgress {
            status,
            current_source,
            timestamp: chrono::Utc::now().to_rfc3339(),
        };

        // Send update (ignore if no subscribers)
        let _ = self.progress_sender.send(progress);
    }

    /// Update refresh progress for a specific source
    pub async fn update_source_progress(&self, source_name: String, completed_count: usize) {
        // Update the in-progress status
        {
            let mut status = self.status.lock().unwrap();
            if let RefreshStatus::InProgress { started_at, sources_total, .. } = &*status {
                *status = RefreshStatus::InProgress {
                    started_at: started_at.clone(),
                    sources_total: *sources_total,
                    sources_completed: completed_count,
                };
            }
        }

        // Send progress update
        self.send_progress_update(Some(source_name)).await;
    }
}

/// Statistics about refresh operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshStats {
    pub last_refresh_time: Option<String>,
    pub last_refresh_duration_ms: Option<u64>,
    pub last_refresh_new_articles: Option<usize>,
    pub last_refresh_failed_sources: Option<usize>,
    pub auto_refresh_enabled: bool,
    pub auto_refresh_interval_minutes: u32,
    pub next_refresh_time: Option<String>,
}

impl RefreshManager {
    /// Get refresh statistics
    pub async fn get_refresh_stats(&self) -> Result<RefreshStats, String> {
        let config = self.config_service.get_app_config()
            .map_err(|e| format!("Failed to get app config: {}", e))?;

        let (last_refresh_time, last_refresh_duration_ms, last_refresh_new_articles, last_refresh_failed_sources) = 
            match self.get_refresh_status() {
                RefreshStatus::Completed { duration_ms, new_articles, failed_sources } => {
                    (Some(chrono::Utc::now().to_rfc3339()), Some(duration_ms), Some(new_articles), Some(failed_sources))
                }
                _ => (None, None, None, None)
            };

        let next_refresh_time = self.get_next_refresh_time().await?
            .map(|dt| dt.to_rfc3339());

        Ok(RefreshStats {
            last_refresh_time,
            last_refresh_duration_ms,
            last_refresh_new_articles,
            last_refresh_failed_sources,
            auto_refresh_enabled: config.auto_refresh.enabled,
            auto_refresh_interval_minutes: config.auto_refresh.interval_minutes,
            next_refresh_time,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_refresh_progress_serialization() {
        let progress = RefreshProgress {
            status: RefreshStatus::InProgress {
                started_at: "2023-01-01T00:00:00Z".to_string(),
                sources_total: 5,
                sources_completed: 2,
            },
            current_source: Some("Reuters".to_string()),
            timestamp: "2023-01-01T00:01:00Z".to_string(),
        };

        let serialized = serde_json::to_string(&progress).unwrap();
        let deserialized: RefreshProgress = serde_json::from_str(&serialized).unwrap();

        assert!(matches!(deserialized.status, RefreshStatus::InProgress { .. }));
        assert_eq!(deserialized.current_source, Some("Reuters".to_string()));
    }
}