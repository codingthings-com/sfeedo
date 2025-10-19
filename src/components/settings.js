// Settings and configuration management
import { TauriAPI } from '../api/tauri-api.js';

class SettingsManager {
  constructor() {
    this.config = null;
    this.feedSources = [];
    this.isLoading = false;

    this.initializeElements();
    this.setupEventListeners();
    this.loadConfiguration();
  }

  initializeElements() {
    this.container = document.querySelector('.settings-container');
  }

  setupEventListeners() {
    // Settings will be rendered when the settings view is shown
    document.addEventListener('DOMContentLoaded', () => {
      // Listen for view changes to render settings when needed
      const settingsNavBtn = document.querySelector('[data-view="settings"]');
      if (settingsNavBtn) {
        settingsNavBtn.addEventListener('click', () => {
          console.log('SettingsManager: Settings nav button clicked');
          setTimeout(() => {
            console.log('SettingsManager: Triggering render after nav click');
            this.renderSettings();
          }, 100);
        });
      }
    });
  }

  // Load configuration from backend
  async loadConfiguration() {
    console.log('SettingsManager: Starting loadConfiguration');
    try {
      this.isLoading = true;

      console.log('SettingsManager: Loading config and feed sources...');
      // Load both config and feed sources in parallel
      const [config, feedSources] = await Promise.all([
        TauriAPI.config.getConfig(),
        TauriAPI.feedSources.getFeedSources()
      ]);

      console.log('SettingsManager: Loaded config:', config);
      console.log('SettingsManager: Loaded feedSources:', feedSources);

      this.config = config;
      this.feedSources = feedSources;
      this.isLoading = false; // Set loading to false before rendering

      // Always render settings once loaded, regardless of view state
      console.log('SettingsManager: Rendering settings...');
      this.renderSettings();
    } catch (error) {
      console.error('SettingsManager: Failed to load configuration:', error);
      window.AppNavigation.updateStatus('Failed to load settings');
      this.showError('Failed to load settings. Please try refreshing.');
    } finally {
      this.isLoading = false;
      console.log('SettingsManager: loadConfiguration finished');
    }
  }

  // Render the complete settings interface
  renderSettings() {
    console.log('SettingsManager: renderSettings called');
    console.log('SettingsManager: container:', this.container);
    console.log('SettingsManager: isLoading:', this.isLoading);
    console.log('SettingsManager: config:', this.config);
    console.log('SettingsManager: feedSources:', this.feedSources);

    if (!this.container) {
      console.error('SettingsManager: No container found!');
      return;
    }

    if (this.isLoading || !this.config || !this.feedSources) {
      console.log('SettingsManager: Showing loading state');
      this.container.innerHTML = `
        <div class="settings-loading">
          <div class="spinner"></div>
          <p>Loading settings...</p>
        </div>
      `;
      return;
    }

    console.log('SettingsManager: Rendering full settings');
    const settingsHTML = `
      <div class="settings-sections">
        ${this.renderFeedSourcesSection()}
        ${this.renderAutoRefreshSection()}
        ${this.renderUIPreferencesSection()}
        ${this.renderAboutSection()}
      </div>
    `;

    this.container.innerHTML = settingsHTML;
    this.attachSettingsEventListeners();
    console.log('SettingsManager: Settings rendered successfully');
  }

  // Render feed sources management section
  renderFeedSourcesSection() {
    const feedSourcesHTML = this.feedSources.map(source => `
      <div class="feed-source-item" data-source-id="${source.id}">
        <div class="feed-source-info">
          <div class="feed-source-header">
            <h4 class="feed-source-name">${this.escapeHtml(source.name)}</h4>
            <div class="feed-source-toggle">
              <label class="toggle-switch">
                <input type="checkbox" ${source.enabled ? 'checked' : ''} 
                       data-action="toggle-source" data-source-id="${source.id}">
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
          <p class="feed-source-url">${this.escapeHtml(source.url)}</p>
          <div class="feed-source-status">
            <span class="status-indicator ${source.enabled ? 'enabled' : 'disabled'}">
              ${source.enabled ? '✓ Active' : '○ Disabled'}
            </span>
            <span class="last-fetched">Built-in financial news scraper</span>
          </div>
        </div>
      </div>
    `).join('');

    return `
      <div class="settings-section">
        <div class="section-header">
          <h3 class="section-title">Feed Sources</h3>
          <button class="btn btn-primary" data-action="add-source">
            <span class="btn-icon">+</span>
            Add Feed Source
          </button>
        </div>
        <div class="section-content">
          <p class="section-description">
            Manage your news feed sources. Enable or disable sources to customize your news feed.
          </p>
          <div class="feed-sources-list">
            ${feedSourcesHTML}
          </div>
        </div>
      </div>
    `;
  }

  // Render auto-refresh configuration section
  renderAutoRefreshSection() {
    return `
      <div class="settings-section">
        <div class="section-header">
          <h3 class="section-title">Auto-Refresh Settings</h3>
        </div>
        <div class="section-content">
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Enable Auto-Refresh</label>
              <p class="setting-description">Automatically refresh feeds at regular intervals</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" ${this.config.auto_refresh.enabled ? 'checked' : ''} 
                       data-setting="auto_refresh.enabled">
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
          
          <div class="setting-item ${!this.config.auto_refresh.enabled ? 'disabled' : ''}">
            <div class="setting-info">
              <label class="setting-label">Refresh Interval</label>
              <p class="setting-description">How often to check for new articles</p>
            </div>
            <div class="setting-control">
              <select class="setting-select" data-setting="auto_refresh.interval_minutes" 
                      ${!this.config.auto_refresh.enabled ? 'disabled' : ''}>
                <option value="2" ${this.config.auto_refresh.interval_minutes === 2 ? 'selected' : ''}>2 minutes</option>
                <option value="5" ${this.config.auto_refresh.interval_minutes === 5 ? 'selected' : ''}>5 minutes</option>
                <option value="15" ${this.config.auto_refresh.interval_minutes === 15 ? 'selected' : ''}>15 minutes</option>
                <option value="30" ${this.config.auto_refresh.interval_minutes === 30 ? 'selected' : ''}>30 minutes</option>
                <option value="60" ${this.config.auto_refresh.interval_minutes === 60 ? 'selected' : ''}>1 hour</option>
                <option value="120" ${this.config.auto_refresh.interval_minutes === 120 ? 'selected' : ''}>2 hours</option>
                <option value="360" ${this.config.auto_refresh.interval_minutes === 360 ? 'selected' : ''}>6 hours</option>
                <option value="720" ${this.config.auto_refresh.interval_minutes === 720 ? 'selected' : ''}>12 hours</option>
                <option value="1440" ${this.config.auto_refresh.interval_minutes === 1440 ? 'selected' : ''}>24 hours</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Render UI preferences section
  renderUIPreferencesSection() {
    // Use default values if ui config is not available
    const uiConfig = this.config.ui || {
      theme: 'system',
      articles_per_page: 20,
      show_notifications: true
    };

    return `
      <div class="settings-section">
        <div class="section-header">
          <h3 class="section-title">Display Preferences</h3>
        </div>
        <div class="section-content">
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Theme</label>
              <p class="setting-description">Choose your preferred color scheme</p>
            </div>
            <div class="setting-control">
              <select class="setting-select" data-setting="ui.theme">
                <option value="light" ${uiConfig.theme === 'light' ? 'selected' : ''}>Light</option>
                <option value="dark" ${uiConfig.theme === 'dark' ? 'selected' : ''}>Dark</option>
                <option value="system" ${uiConfig.theme === 'system' ? 'selected' : ''}>System</option>
              </select>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Articles Per Page</label>
              <p class="setting-description">Number of articles to display at once</p>
            </div>
            <div class="setting-control">
              <select class="setting-select" data-setting="ui.articles_per_page">
                <option value="10" ${uiConfig.articles_per_page === 10 ? 'selected' : ''}>10</option>
                <option value="20" ${uiConfig.articles_per_page === 20 ? 'selected' : ''}>20</option>
                <option value="50" ${uiConfig.articles_per_page === 50 ? 'selected' : ''}>50</option>
                <option value="100" ${uiConfig.articles_per_page === 100 ? 'selected' : ''}>100</option>
              </select>
            </div>
          </div>
          
          <div class="setting-item">
            <div class="setting-info">
              <label class="setting-label">Show Notifications</label>
              <p class="setting-description">Display notifications for new articles</p>
            </div>
            <div class="setting-control">
              <label class="toggle-switch">
                <input type="checkbox" ${uiConfig.show_notifications ? 'checked' : ''} 
                       data-setting="ui.show_notifications">
                <span class="toggle-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Render about section
  renderAboutSection() {
    return `
      <div class="settings-section">
        <div class="section-header">
          <h3 class="section-title">About</h3>
        </div>
        <div class="section-content">
          <div class="about-info">
            <h4>Sfeedo Desktop Feed Reader</h4>
            <p>Version 0.1.0</p>
            <p>A modern RSS/Atom feed aggregator built with Tauri and Rust.</p>
            <div class="about-actions">
              <button class="btn btn-outline" data-action="check-updates">
                Check for Updates
              </button>
              <button class="btn btn-outline" data-action="view-logs">
                View Logs
              </button>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Attach event listeners to settings elements
  attachSettingsEventListeners() {
    // Setting controls
    const settingControls = this.container.querySelectorAll('[data-setting]');
    settingControls.forEach(control => {
      control.addEventListener('change', (e) => {
        this.handleSettingChange(e.target.dataset.setting, e.target);
      });
    });

    // Action buttons
    const actionButtons = this.container.querySelectorAll('[data-action]');
    actionButtons.forEach(button => {
      button.addEventListener('click', (e) => {
        this.handleAction(e.target.dataset.action, e.target);
      });
    });
  }

  // Handle setting changes
  async handleSettingChange(settingPath, element) {
    const keys = settingPath.split('.');
    let current = this.config;

    // Navigate to the parent object
    for (let i = 0; i < keys.length - 1; i++) {
      current = current[keys[i]];
    }

    // Set the value
    const finalKey = keys[keys.length - 1];
    let value = element.value;

    // Convert to appropriate type
    if (element.type === 'checkbox') {
      value = element.checked;
    } else if (element.type === 'number' || !isNaN(Number(value))) {
      value = Number(value);
    }

    const oldValue = current[finalKey];
    current[finalKey] = value;

    try {
      // Handle dependent settings
      if (settingPath === 'auto_refresh.enabled') {
        this.updateAutoRefreshDependents();
      }

      // Save configuration to backend
      await this.saveConfiguration();

      window.AppNavigation.updateStatus(`Setting updated: ${settingPath}`);

      // Dispatch settings change event
      this.dispatchSettingsChangeEvent(settingPath, value, oldValue);
    } catch (error) {
      console.error('Failed to save setting:', error);
      // Revert the change
      current[finalKey] = oldValue;
      element.checked = oldValue; // For checkboxes
      element.value = oldValue; // For other inputs
      window.AppNavigation.updateStatus('Failed to save setting');
    }
  }

  // Update auto-refresh dependent controls
  updateAutoRefreshDependents() {
    const intervalSelect = this.container.querySelector('[data-setting="auto_refresh.interval_minutes"]');
    const intervalItem = intervalSelect?.closest('.setting-item');

    if (intervalSelect && intervalItem) {
      intervalSelect.disabled = !this.config.auto_refresh.enabled;
      intervalItem.classList.toggle('disabled', !this.config.auto_refresh.enabled);
    }
  }

  // Handle action buttons
  async handleAction(action, element) {
    switch (action) {
      case 'add-source':
        this.showAddSourceDialog();
        break;
      case 'edit-source':
        this.showEditSourceDialog(element.dataset.sourceId);
        break;
      case 'remove-source':
        await this.removeSource(element.dataset.sourceId);
        break;
      case 'toggle-source':
        await this.toggleSource(element.dataset.sourceId, element.checked);
        break;
      case 'check-updates':
        this.checkForUpdates();
        break;
      case 'view-logs':
        this.viewLogs();
        break;
    }
  }

  // Show add source dialog
  showAddSourceDialog() {
    const dialog = this.createSourceDialog();
    document.body.appendChild(dialog);
  }

  // Show edit source dialog
  showEditSourceDialog(sourceId) {
    const source = this.feedSources.find(s => s.id === sourceId);
    if (source) {
      const dialog = this.createSourceDialog(source);
      document.body.appendChild(dialog);
    }
  }

  // Create source dialog
  createSourceDialog(source = null) {
    const isEdit = !!source;
    const dialog = document.createElement('div');
    dialog.className = 'source-dialog';

    dialog.innerHTML = `
      <div class="dialog-overlay">
        <div class="dialog-content">
          <div class="dialog-header">
            <h3>${isEdit ? 'Edit' : 'Add'} Feed Source</h3>
            <button class="dialog-close">&times;</button>
          </div>
          <div class="dialog-body">
            <form class="source-form">
              <div class="form-group">
                <label class="form-label">Name</label>
                <input type="text" class="form-input" name="name" 
                       value="${source?.name || ''}" placeholder="e.g., Reuters Finance" required>
              </div>
              <div class="form-group">
                <label class="form-label">URL</label>
                <input type="url" class="form-input" name="url" 
                       value="${source?.url || ''}" placeholder="https://example.com/feed.rss" required>
              </div>
              <div class="form-group">
                <label class="form-checkbox">
                  <input type="checkbox" name="enabled" ${source?.enabled !== false ? 'checked' : ''}>
                  <span class="checkbox-label">Enable this source</span>
                </label>
              </div>
            </form>
          </div>
          <div class="dialog-footer">
            <button class="btn btn-outline dialog-cancel">Cancel</button>
            <button class="btn btn-primary dialog-save">${isEdit ? 'Update' : 'Add'} Source</button>
          </div>
        </div>
      </div>
    `;

    // Add event listeners
    const closeBtn = dialog.querySelector('.dialog-close');
    const cancelBtn = dialog.querySelector('.dialog-cancel');
    const saveBtn = dialog.querySelector('.dialog-save');
    const overlay = dialog.querySelector('.dialog-overlay');

    const closeDialog = () => {
      document.body.removeChild(dialog);
    };

    closeBtn.addEventListener('click', closeDialog);
    cancelBtn.addEventListener('click', closeDialog);
    overlay.addEventListener('click', (e) => {
      if (e.target === overlay) closeDialog();
    });

    saveBtn.addEventListener('click', async () => {
      const form = dialog.querySelector('.source-form');
      const formData = new FormData(form);

      if (form.checkValidity()) {
        const sourceData = {
          name: formData.get('name'),
          url: formData.get('url'),
          enabled: formData.has('enabled')
        };

        try {
          saveBtn.disabled = true;
          saveBtn.textContent = 'Saving...';

          if (isEdit) {
            await this.updateSource(source.id, sourceData);
          } else {
            await this.addSource(sourceData);
          }

          closeDialog();
        } catch (error) {
          // Error is already handled in add/update methods
          saveBtn.disabled = false;
          saveBtn.textContent = isEdit ? 'Update Source' : 'Add Source';
        }
      } else {
        form.reportValidity();
      }
    });

    return dialog;
  }

  // Add new source
  async addSource(sourceData) {
    try {
      const newSource = await TauriAPI.feedSources.addFeedSource(
        sourceData.name,
        sourceData.url,
        sourceData.enabled
      );

      this.feedSources.push(newSource);
      this.renderSettings();
      window.AppNavigation.updateStatus(`Added feed source: ${sourceData.name}`);
    } catch (error) {
      console.error('Failed to add feed source:', error);
      window.AppNavigation.updateStatus('Failed to add feed source');
      throw error;
    }
  }

  // Update existing source
  async updateSource(sourceId, sourceData) {
    try {
      const sourceIndex = this.feedSources.findIndex(s => s.id === sourceId);
      if (sourceIndex === -1) return;

      const updatedSource = {
        ...this.feedSources[sourceIndex],
        ...sourceData
      };

      await TauriAPI.feedSources.updateFeedSource(updatedSource);

      this.feedSources[sourceIndex] = updatedSource;
      this.renderSettings();
      window.AppNavigation.updateStatus(`Updated feed source: ${sourceData.name}`);
    } catch (error) {
      console.error('Failed to update feed source:', error);
      window.AppNavigation.updateStatus('Failed to update feed source');
      throw error;
    }
  }

  // Remove source
  async removeSource(sourceId) {
    const source = this.feedSources.find(s => s.id === sourceId);
    if (!source) return;

    if (!confirm(`Are you sure you want to remove "${source.name}"?`)) {
      return;
    }

    try {
      await TauriAPI.feedSources.removeFeedSource(sourceId);

      this.feedSources = this.feedSources.filter(s => s.id !== sourceId);
      this.renderSettings();
      window.AppNavigation.updateStatus(`Removed feed source: ${source.name}`);
    } catch (error) {
      console.error('Failed to remove feed source:', error);
      window.AppNavigation.updateStatus('Failed to remove feed source');
    }
  }

  // Toggle source enabled state
  async toggleSource(sourceId, enabled) {
    const source = this.feedSources.find(s => s.id === sourceId);
    if (!source) return;

    try {
      await TauriAPI.feedSources.toggleFeedSource(sourceId, enabled);

      source.enabled = enabled;
      this.renderSettings();
      window.AppNavigation.updateStatus(`${enabled ? 'Enabled' : 'Disabled'} feed source: ${source.name}`);
    } catch (error) {
      console.error('Failed to toggle feed source:', error);
      window.AppNavigation.updateStatus('Failed to update feed source');
      // Revert the toggle
      const toggle = this.container.querySelector(`[data-source-id="${sourceId}"]`);
      if (toggle) {
        toggle.checked = !enabled;
      }
    }
  }

  // Save configuration to backend
  async saveConfiguration() {
    try {
      await TauriAPI.config.updateConfig(this.config);
    } catch (error) {
      console.error('Failed to save configuration:', error);
      throw error;
    }
  }

  // Check for updates
  checkForUpdates() {
    window.AppNavigation.showProgress('Checking for updates...');

    setTimeout(() => {
      window.AppNavigation.hideProgress();
      window.AppNavigation.updateStatus('No updates available');
    }, 2000);
  }

  // View logs
  viewLogs() {
    window.AppNavigation.updateStatus('Opening logs...');
    // This will be implemented with Tauri shell API
  }

  // Get current configuration
  getConfiguration() {
    return this.config ? { ...this.config } : null;
  }

  // Get feed sources
  getFeedSources() {
    return [...this.feedSources];
  }

  // Utility function to escape HTML
  escapeHtml(text) {
    if (!text) return '';
    const div = document.createElement('div');
    div.textContent = text;
    return div.innerHTML;
  }

  // Show error message
  showError(message) {
    if (!this.container) return;

    this.container.innerHTML = `
      <div class="error-state">
        <div class="error-icon">⚠️</div>
        <h3>Error Loading Settings</h3>
        <p>${message}</p>
        <button class="btn btn-primary" onclick="settingsManager.loadConfiguration()">
          Try Again
        </button>
      </div>
    `;
  }

  // Dispatch settings change event
  dispatchSettingsChangeEvent(settingPath, newValue, oldValue) {
    const event = new CustomEvent('settingsChanged', {
      detail: {
        path: settingPath,
        newValue,
        oldValue,
        config: this.config
      }
    });
    document.dispatchEvent(event);
  }

  // Validate feed URL before adding
  async validateFeedUrl(url) {
    try {
      const result = await TauriAPI.feedSources.validateFeedSource(url);
      return result;
    } catch (error) {
      console.error('Failed to validate feed URL:', error);
      return {
        is_valid: false,
        error_message: 'Failed to validate URL'
      };
    }
  }
}

// Initialize settings manager
let settingsManager;

document.addEventListener('DOMContentLoaded', () => {
  settingsManager = new SettingsManager();
});

// Export for global access
window.SettingsManager = SettingsManager;