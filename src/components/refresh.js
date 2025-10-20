// Refresh controls and status management
import { TauriAPI } from '../api/tauri-api.js';

class RefreshManager {
  constructor() {
    this.isRefreshing = false;
    this.autoRefreshTimer = null;
    this.autoRefreshEnabled = true;
    this.autoRefreshInterval = 30; // minutes
    this.lastRefreshTime = null;
    this.refreshProgress = 0;
    this.totalSources = 0;
    this.completedSources = 0;
    this.config = null;
    
    this.initializeElements();
    this.setupEventListeners();
    this.loadConfiguration();
  }

  initializeElements() {
    this.refreshBtn = document.getElementById('refresh-btn');
    this.refreshIcon = this.refreshBtn?.querySelector('.refresh-icon');
    this.statusText = document.getElementById('status-text');
    this.autoRefreshIndicator = document.getElementById('auto-refresh-indicator');
    this.lastUpdateElement = document.getElementById('last-update');
    this.loadingOverlay = document.getElementById('loading-overlay');
  }

  setupEventListeners() {
    // Refresh button click
    if (this.refreshBtn) {
      this.refreshBtn.addEventListener('click', () => {
        if (!this.isRefreshing) {
          this.performManualRefresh();
        }
      });
    }

    // Listen for settings changes
    document.addEventListener('settingsChanged', (e) => {
      if (e.detail.path && e.detail.path.startsWith('auto_refresh')) {
        this.loadConfiguration();
      }
    });

    // Keyboard shortcuts
    document.addEventListener('keydown', (e) => {
      // Ctrl/Cmd + R for refresh
      if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
        e.preventDefault();
        if (!this.isRefreshing) {
          this.performManualRefresh();
        }
      }
    });
  }

  // Perform manual refresh
  async performManualRefresh() {
    if (this.isRefreshing) return;

    try {
      this.startRefresh('Manual refresh in progress...');
      
      // Call Tauri backend to refresh feeds
      const result = await TauriAPI.refresh.refreshFeeds();
      
      if (result.success) {
        const message = result.new_articles > 0 
          ? `REFRESH COMPLETE - ${result.new_articles} NEW ARTICLES`
          : 'REFRESH COMPLETE - NO NEW ARTICLES';
        this.completeRefresh(message);
      } else {
        this.completeRefresh((result.message || 'REFRESH FAILED').toUpperCase(), true);
      }
      
      // Reload articles if article manager exists
      if (window.articleManager) {
        await window.articleManager.refresh();
      }
      
    } catch (error) {
      console.error('Refresh failed:', error);
      this.completeRefresh('REFRESH FAILED', true);
      this.showError('FAILED TO REFRESH FEEDS. CHECK INTERNET CONNECTION.');
    }
  }

  // Start refresh process
  startRefresh(message = 'REFRESHING...') {
    this.isRefreshing = true;
    this.refreshProgress = 0;
    this.completedSources = 0;
    
    // Update UI
    this.updateStatus(message);
    this.setRefreshButtonState(true);
    this.showProgressOverlay(message);
    
    // Start monitoring refresh status
    this.startStatusMonitoring();
    
    // Dispatch event
    this.dispatchRefreshEvent('start');
  }

  // Complete refresh process
  completeRefresh(message = 'REFRESH COMPLETE', isError = false) {
    this.isRefreshing = false;
    this.refreshProgress = 100;
    this.lastRefreshTime = new Date();
    
    // Stop monitoring refresh status
    this.stopStatusMonitoring();
    
    // Update UI
    this.updateStatus(message);
    this.setRefreshButtonState(false);
    this.hideProgressOverlay();
    this.updateLastRefreshDisplay();
    
    // Show notification if enabled
    if (!isError) {
      this.showRefreshNotification();
    }
    
    // Dispatch event
    this.dispatchRefreshEvent('complete', { success: !isError });
  }

  // Load configuration from backend
  async loadConfiguration() {
    try {
      this.config = await TauriAPI.config.getConfig();
      this.autoRefreshEnabled = this.config.auto_refresh.enabled;
      this.autoRefreshInterval = this.config.auto_refresh.interval_minutes;
      
      // Update auto-refresh based on new configuration
      if (this.autoRefreshEnabled) {
        this.startAutoRefresh();
      } else {
        this.stopAutoRefresh();
      }
    } catch (error) {
      console.error('Failed to load refresh configuration:', error);
      // Use defaults if loading fails
      this.autoRefreshEnabled = true;
      this.autoRefreshInterval = 30;
    }
  }

  // Get refresh progress from backend
  async getRefreshProgressInfo() {
    try {
      const progress = await TauriAPI.refresh.getRefreshProgress();
      this.totalSources = progress.enabled_sources;
      return progress;
    } catch (error) {
      console.error('Failed to get refresh progress:', error);
      return null;
    }
  }

  // Update progress display
  updateProgressDisplay() {
    const message = `REFRESHING... (${this.completedSources}/${this.totalSources})`;
    this.updateStatus(message);
    
    const loadingText = this.loadingOverlay?.querySelector('.teletext-loading-text');
    if (loadingText) {
      loadingText.textContent = message;
    }
  }

  // Set refresh button state
  setRefreshButtonState(isRefreshing) {
    if (!this.refreshBtn || !this.refreshIcon) return;
    
    this.refreshBtn.disabled = isRefreshing;
    
    if (isRefreshing) {
      this.refreshBtn.style.opacity = '0.5';
      this.refreshBtn.title = 'REFRESHING...';
    } else {
      this.refreshBtn.style.opacity = '1';
      this.refreshBtn.title = 'REFRESH FEEDS';
    }
  }

  // Show/hide progress overlay
  showProgressOverlay(message) {
    if (!this.loadingOverlay) return;
    
    this.loadingOverlay.classList.remove('hidden');
    const loadingText = this.loadingOverlay.querySelector('.teletext-loading-text');
    if (loadingText) {
      loadingText.textContent = message.toUpperCase();
    }
  }

  hideProgressOverlay() {
    if (this.loadingOverlay) {
      this.loadingOverlay.classList.add('hidden');
    }
  }

  // Update status text
  updateStatus(message) {
    if (this.statusText) {
      this.statusText.textContent = message;
    }
  }

  // Update last refresh display
  updateLastRefreshDisplay() {
    if (!this.lastUpdateElement || !this.lastRefreshTime) return;
    
    const timeString = this.lastRefreshTime.toLocaleTimeString();
    this.lastUpdateElement.textContent = `Last updated: ${timeString}`;
  }

  // Show refresh notification
  showRefreshNotification() {
    // Check if notifications are enabled
    const config = window.settingsManager?.getConfiguration();
    if (!config?.ui?.show_notifications) return;

    // Create notification element
    const notification = this.createNotification(
      'Feeds Updated',
      `Successfully refreshed ${this.totalSources} feed sources`,
      'success'
    );
    
    document.body.appendChild(notification);
    
    // Auto-remove after 3 seconds
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    }, 3000);
  }

  // Create notification element
  createNotification(title, message, type = 'info') {
    const notification = document.createElement('div');
    notification.className = `notification notification-${type}`;
    
    notification.innerHTML = `
      <div class="notification-content">
        <div class="notification-icon">
          ${type === 'success' ? '✅' : type === 'error' ? '❌' : 'ℹ️'}
        </div>
        <div class="notification-text">
          <div class="notification-title">${title}</div>
          <div class="notification-message">${message}</div>
        </div>
        <button class="notification-close">&times;</button>
      </div>
    `;
    
    // Add close functionality
    const closeBtn = notification.querySelector('.notification-close');
    closeBtn.addEventListener('click', () => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    });
    
    return notification;
  }

  // Show error message
  showError(message) {
    const notification = this.createNotification(
      'Refresh Error',
      message,
      'error'
    );
    
    document.body.appendChild(notification);
    
    // Auto-remove after 5 seconds for errors
    setTimeout(() => {
      if (notification.parentNode) {
        notification.parentNode.removeChild(notification);
      }
    }, 5000);
  }

  // Auto-refresh management
  startAutoRefresh() {
    this.stopAutoRefresh(); // Clear any existing timer
    
    if (!this.autoRefreshEnabled) {
      this.updateAutoRefreshIndicator(false);
      return;
    }
    
    const intervalMs = this.autoRefreshInterval * 60 * 1000;
    
    this.autoRefreshTimer = setInterval(() => {
      if (!this.isRefreshing) {
        this.performAutoRefresh();
      }
    }, intervalMs);
    
    this.updateAutoRefreshIndicator(true);
  }

  stopAutoRefresh() {
    if (this.autoRefreshTimer) {
      clearInterval(this.autoRefreshTimer);
      this.autoRefreshTimer = null;
    }
    this.updateAutoRefreshIndicator(false);
  }

  // Perform auto refresh
  async performAutoRefresh() {
    try {
      this.startRefresh('Auto-refresh in progress...');
      
      const result = await TauriAPI.refresh.refreshFeeds();
      
      if (result.success) {
        const message = result.new_articles > 0 
          ? `Auto-refresh completed - ${result.new_articles} new articles found`
          : 'Auto-refresh completed - no new articles';
        this.completeRefresh(message);
      } else {
        this.completeRefresh(result.message || 'Auto-refresh failed', true);
      }
      
      // Reload articles
      if (window.articleManager) {
        await window.articleManager.refresh();
      }
      
    } catch (error) {
      console.error('Auto-refresh failed:', error);
      this.completeRefresh('Auto-refresh failed', true);
    }
  }

  // Update auto-refresh settings
  updateAutoRefreshSettings(settings) {
    this.autoRefreshEnabled = settings.enabled;
    this.autoRefreshInterval = settings.interval_minutes;
    
    if (this.autoRefreshEnabled) {
      this.startAutoRefresh();
    } else {
      this.stopAutoRefresh();
    }
  }

  // Update auto-refresh indicator
  updateAutoRefreshIndicator(enabled) {
    if (!this.autoRefreshIndicator) return;
    
    if (enabled) {
      this.autoRefreshIndicator.textContent = `AUTO: ON (${this.autoRefreshInterval}M)`;
      this.autoRefreshIndicator.classList.remove('hidden');
    } else {
      this.autoRefreshIndicator.classList.add('hidden');
    }
  }

  // Dispatch refresh events
  dispatchRefreshEvent(type, data = {}) {
    const event = new CustomEvent('refreshEvent', {
      detail: { type, ...data }
    });
    document.dispatchEvent(event);
  }

  // Get refresh status
  getRefreshStatus() {
    return {
      isRefreshing: this.isRefreshing,
      progress: this.refreshProgress,
      lastRefreshTime: this.lastRefreshTime,
      autoRefreshEnabled: this.autoRefreshEnabled,
      autoRefreshInterval: this.autoRefreshInterval
    };
  }

  // Get refresh status from backend
  async getBackendRefreshStatus() {
    try {
      return await TauriAPI.refresh.getRefreshStatus();
    } catch (error) {
      console.error('Failed to get refresh status:', error);
      return null;
    }
  }

  // Monitor refresh status periodically
  startStatusMonitoring() {
    // Check status every 5 seconds when refreshing
    if (this.statusMonitorTimer) {
      clearInterval(this.statusMonitorTimer);
    }

    this.statusMonitorTimer = setInterval(async () => {
      if (this.isRefreshing) {
        const status = await this.getBackendRefreshStatus();
        if (status) {
          this.updateStatusFromBackend(status);
        }
      }
    }, 5000);
  }

  // Stop status monitoring
  stopStatusMonitoring() {
    if (this.statusMonitorTimer) {
      clearInterval(this.statusMonitorTimer);
      this.statusMonitorTimer = null;
    }
  }

  // Update status from backend response
  updateStatusFromBackend(status) {
    if (status.message) {
      this.updateStatus(status.message);
    }
    
    // Update auto-refresh indicator
    if (status.last_refresh_time) {
      this.lastRefreshTime = new Date(status.last_refresh_time);
      this.updateLastRefreshDisplay();
    }
  }

  // Force refresh (for external calls)
  async forceRefresh() {
    if (!this.isRefreshing) {
      await this.performManualRefresh();
    }
  }
}

// Initialize refresh manager
let refreshManager;

document.addEventListener('DOMContentLoaded', () => {
  refreshManager = new RefreshManager();
  
  // Make refresh function available globally
  window.AppNavigation.handleRefresh = () => {
    refreshManager.forceRefresh();
  };

  // Load initial configuration after a short delay
  setTimeout(() => {
    refreshManager.loadConfiguration();
  }, 100);
});

// Export for global access
window.RefreshManager = RefreshManager;
window.refreshManager = refreshManager;