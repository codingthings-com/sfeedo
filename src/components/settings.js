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
          // console.log('SettingsManager: Settings nav button clicked');
          setTimeout(() => {
            // console.log('SettingsManager: Triggering render after nav click');
            this.renderSettings();
          }, 100);
        });
      }
    });
  }

  // Load configuration from backend
  async loadConfiguration() {
    // console.log('SettingsManager: Starting loadConfiguration');
    try {
      this.isLoading = true;

      // console.log('SettingsManager: Loading config and feed sources...');
      // Load both config and feed sources in parallel
      const [config, feedSources] = await Promise.all([
        TauriAPI.config.getConfig(),
        TauriAPI.feedSources.getFeedSources()
      ]);

      // console.log('SettingsManager: Loaded config:', config);
      // console.log('SettingsManager: Loaded feedSources:', feedSources);

      this.config = config;
      this.feedSources = feedSources;
      this.isLoading = false; // Set loading to false before rendering

      // Always render settings once loaded, regardless of view state
      // console.log('SettingsManager: Rendering settings...');
      this.renderSettings();
    } catch (error) {
      // console.error('SettingsManager: Failed to load configuration:', error);
      window.AppNavigation.updateStatus('Failed to load settings');
      this.showError('Failed to load settings. Please try refreshing.');
    } finally {
      this.isLoading = false;
      // console.log('SettingsManager: loadConfiguration finished');
    }
  }

  // Render the complete settings interface
  renderSettings() {
    // console.log('SettingsManager: renderSettings called');
    // console.log('SettingsManager: container:', this.container);
    // console.log('SettingsManager: isLoading:', this.isLoading);
    // console.log('SettingsManager: config:', this.config);
    // console.log('SettingsManager: feedSources:', this.feedSources);

    if (!this.container) {
      // console.error('SettingsManager: No container found!');
      return;
    }

    if (this.isLoading || !this.config || !this.feedSources) {
      // console.log('SettingsManager: Showing loading state');
      this.container.innerHTML = `
        <div class="settings-loading">
          <div class="spinner"></div>
          <p>Loading settings...</p>
        </div>
      `;
      return;
    }

    // console.log('SettingsManager: Rendering full settings');
    const settingsHTML = `
      <div class="teletext-settings-sections">
        ${this.renderFeedSourcesSection()}
        ${this.renderAutoRefreshSection()}
        ${this.renderUIPreferencesSection()}
        ${this.renderConfigurationSection()}
        ${this.renderAboutSection()}
      </div>
    `;

    this.container.innerHTML = settingsHTML;
    this.attachSettingsEventListeners();
    // console.log('SettingsManager: Settings rendered successfully');
  }

  // Render feed sources management section - teletext style
  renderFeedSourcesSection() {
    // console.log('renderFeedSourcesSection: feedSources =', this.feedSources);
    const builtinSources = this.feedSources.filter(s => s.source_type === 'builtin');
    const customSources = this.feedSources.filter(s => s.source_type === 'custom');
    // console.log('builtinSources:', builtinSources);
    // console.log('customSources:', customSources);

    const builtinSourcesHTML = builtinSources.map(source => {
      // console.log('Rendering source:', source.id, 'available_topics:', source.available_topics, 'enabled_topics:', source.enabled_topics);
      return `
      <div class="teletext-setting-item" data-source-id="${source.id}">
        <div class="teletext-setting-info">
          <label class="teletext-setting-label">${this.escapeHtml(source.name).toUpperCase()}</label>
          <p class="teletext-setting-description">
            ${source.enabled ? '[ACTIVE]' : '[DISABLED]'} - BUILT-IN READER
          </p>
          ${source.available_topics && source.available_topics.length > 0 ? `
            <div class="teletext-topics-list">
              <p class="teletext-setting-description">TOPICS (${source.enabled_topics ? source.enabled_topics.length : 0}/${source.available_topics.length}):</p>
              <button class="teletext-btn-small" data-action="manage-topics" data-source-id="${source.id}">
                MANAGE TOPICS
              </button>
            </div>
          ` : '<p class="teletext-setting-description">NO TOPICS AVAILABLE</p>'}
        </div>
        <div class="teletext-setting-control">
          <label class="teletext-toggle-switch">
            <input type="checkbox" ${source.enabled ? 'checked' : ''} 
                   data-action="toggle-source" data-source-id="${source.id}">
            <span class="teletext-toggle-slider"></span>
          </label>
        </div>
      </div>
    `;
    }).join('');

    const customSourcesHTML = customSources.length > 0 ? customSources.map(source => `
      <div class="teletext-setting-item" data-source-id="${source.id}">
        <div class="teletext-setting-info">
          <label class="teletext-setting-label">${this.escapeHtml(source.name).toUpperCase()}</label>
          <p class="teletext-setting-description">${this.escapeHtml(source.url).toUpperCase()}</p>
          <p class="teletext-setting-description">
            ${source.enabled ? '[ACTIVE]' : '[DISABLED]'} - CUSTOM RSS/ATOM FEED
          </p>
          <div class="teletext-custom-actions">
            <button class="teletext-btn-small" data-action="edit-custom" data-source-id="${source.id}">EDIT</button>
            <button class="teletext-btn-small teletext-btn-danger" data-action="delete-custom" data-source-id="${source.id}">DELETE</button>
          </div>
        </div>
        <div class="teletext-setting-control">
          <label class="teletext-toggle-switch">
            <input type="checkbox" ${source.enabled ? 'checked' : ''} 
                   data-action="toggle-custom" data-source-id="${source.id}">
            <span class="teletext-toggle-slider"></span>
          </label>
        </div>
      </div>
    `).join('') : '<p class="teletext-setting-description">NO CUSTOM FEEDS ADDED YET</p>';

    return `
      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">CUSTOM RSS/ATOM FEEDS</h3>
        </div>
        <div class="teletext-section-content">
          <p class="teletext-section-description">
            ADD YOUR OWN RSS OR ATOM FEEDS FROM ANY SOURCE.
          </p>
          <button class="teletext-btn" data-action="add-custom-feed">
            + ADD CUSTOM FEED
          </button>
          <div class="teletext-feed-sources-list">
            ${customSourcesHTML}
          </div>
        </div>
      </div>

      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">BUILT-IN FEEDS</h3>
        </div>
        <div class="teletext-section-content">
          <p class="teletext-section-description">
            SELECT TOPICS FOR EACH SOURCE.
          </p>
          <div class="teletext-feed-sources-list">
            ${builtinSourcesHTML}
          </div>
        </div>
      </div>
    `;
  }

  // Render auto-refresh configuration section - teletext style
  renderAutoRefreshSection() {
    return `
      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">AUTO-REFRESH SETTINGS</h3>
        </div>
        <div class="teletext-section-content">
          <div class="teletext-setting-item">
            <div class="teletext-setting-info">
              <label class="teletext-setting-label">ENABLE AUTO-REFRESH</label>
              <p class="teletext-setting-description">AUTOMATICALLY REFRESH FEEDS AT REGULAR INTERVALS</p>
            </div>
            <div class="teletext-setting-control">
              <label class="teletext-toggle-switch">
                <input type="checkbox" ${this.config.auto_refresh.enabled ? 'checked' : ''} 
                       data-setting="auto_refresh.enabled">
                <span class="teletext-toggle-slider"></span>
              </label>
            </div>
          </div>
          
          <div class="teletext-setting-item ${!this.config.auto_refresh.enabled ? 'disabled' : ''}">
            <div class="teletext-setting-info">
              <label class="teletext-setting-label">REFRESH INTERVAL</label>
              <p class="teletext-setting-description">HOW OFTEN TO CHECK FOR NEW ARTICLES</p>
            </div>
            <div class="teletext-setting-control">
              <select class="teletext-setting-select" data-setting="auto_refresh.interval_minutes" 
                      ${!this.config.auto_refresh.enabled ? 'disabled' : ''}>
                <option value="2" ${this.config.auto_refresh.interval_minutes === 2 ? 'selected' : ''}>2 MINUTES</option>
                <option value="5" ${this.config.auto_refresh.interval_minutes === 5 ? 'selected' : ''}>5 MINUTES</option>
                <option value="15" ${this.config.auto_refresh.interval_minutes === 15 ? 'selected' : ''}>15 MINUTES</option>
                <option value="30" ${this.config.auto_refresh.interval_minutes === 30 ? 'selected' : ''}>30 MINUTES</option>
                <option value="60" ${this.config.auto_refresh.interval_minutes === 60 ? 'selected' : ''}>1 HOUR</option>
                <option value="120" ${this.config.auto_refresh.interval_minutes === 120 ? 'selected' : ''}>2 HOURS</option>
                <option value="360" ${this.config.auto_refresh.interval_minutes === 360 ? 'selected' : ''}>6 HOURS</option>
                <option value="720" ${this.config.auto_refresh.interval_minutes === 720 ? 'selected' : ''}>12 HOURS</option>
                <option value="1440" ${this.config.auto_refresh.interval_minutes === 1440 ? 'selected' : ''}>24 HOURS</option>
              </select>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Render UI preferences section - teletext style
  renderUIPreferencesSection() {
    // Use default values if ui config is not available
    const uiConfig = this.config.ui || {
      show_notifications: true
    };

    return `
      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">DISPLAY PREFERENCES</h3>
        </div>
        <div class="teletext-section-content">
          <div class="teletext-setting-item">
            <div class="teletext-setting-info">
              <label class="teletext-setting-label">SHOW NOTIFICATIONS</label>
              <p class="teletext-setting-description">DISPLAY NOTIFICATIONS FOR NEW ARTICLES</p>
            </div>
            <div class="teletext-setting-control">
              <label class="teletext-toggle-switch">
                <input type="checkbox" ${uiConfig.show_notifications ? 'checked' : ''} 
                       data-setting="ui.show_notifications">
                <span class="teletext-toggle-slider"></span>
              </label>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Render configuration management section - teletext style
  renderConfigurationSection() {
    return `
      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">CONFIGURATION</h3>
        </div>
        <div class="teletext-section-content">
          <p class="teletext-section-description">
            MANAGE APPLICATION CONFIGURATION AND RESET TO DEFAULTS IF NEEDED.
          </p>
          <div class="teletext-config-actions">
            <div class="teletext-setting-item">
              <div class="teletext-setting-info">
                <label class="teletext-setting-label">RESET CONFIGURATION</label>
                <p class="teletext-setting-description">DELETE CONFIG FILE AND RESTORE DEFAULT SETTINGS</p>
              </div>
              <button class="teletext-btn teletext-btn-danger" data-action="reset-config">
                RESET TO DEFAULTS
              </button>
            </div>
          </div>
        </div>
      </div>
    `;
  }

  // Render about section - teletext style
  renderAboutSection() {
    return `
      <div class="teletext-settings-section">
        <div class="teletext-section-header">
          <h3 class="teletext-section-title">ABOUT</h3>
        </div>
        <div class="teletext-section-content">
          <div class="teletext-about-info">
            <div class="teletext-setting-item">
              <div class="teletext-setting-info">
                <label class="teletext-setting-label">SFEEDO DESKTOP FEED READER</label>
                <p class="teletext-setting-description">VERSION 0.1</p>
                <p class="teletext-setting-description">POWERED BY <a href="https://crates.io/crates/finance-news-aggregator-rs" target="_blank" class="teletext-link">FINANCE-NEWS-AGGREGATOR-RS</a></p>
              </div>
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

      window.AppNavigation.updateStatus(`SETTING UPDATED: ${settingPath.toUpperCase()}`);

      // Dispatch settings change event
      this.dispatchSettingsChangeEvent(settingPath, value, oldValue);
    } catch (error) {
      // console.error('Failed to save setting:', error);
      // Revert the change
      current[finalKey] = oldValue;
      element.checked = oldValue; // For checkboxes
      element.value = oldValue; // For other inputs
      window.AppNavigation.updateStatus('FAILED TO SAVE SETTING');
    }
  }

  // Update auto-refresh dependent controls
  updateAutoRefreshDependents() {
    const intervalSelect = this.container.querySelector('[data-setting="auto_refresh.interval_minutes"]');
    const intervalItem = intervalSelect?.closest('.teletext-setting-item');

    if (intervalSelect && intervalItem) {
      intervalSelect.disabled = !this.config.auto_refresh.enabled;
      intervalItem.classList.toggle('disabled', !this.config.auto_refresh.enabled);
    }
  }

  // Handle action buttons
  async handleAction(action, element) {
    switch (action) {
      case 'toggle-source':
        await this.toggleSource(element.dataset.sourceId, element.checked);
        break;
      case 'manage-topics':
        await this.showTopicsDialog(element.dataset.sourceId);
        break;
      case 'add-custom-feed':
        this.showCustomFeedDialog();
        break;
      case 'edit-custom':
        this.showCustomFeedDialog(element.dataset.sourceId);
        break;
      case 'delete-custom':
        await this.deleteCustomFeed(element.dataset.sourceId);
        break;
      case 'toggle-custom':
        await this.toggleCustomFeed(element.dataset.sourceId, element.checked);
        break;
      case 'reset-config':
        await this.resetConfiguration();
        break;
    }
  }

  // Add source functionality removed - sources are built-in

  // Show edit source dialog
  showEditSourceDialog(sourceId) {
    const source = this.feedSources.find(s => s.id === sourceId);
    if (source) {
      const dialog = this.createSourceDialog(source);
      document.body.appendChild(dialog);
    }
  }

  // Create source dialog (edit only)
  createSourceDialog(source) {
    if (!source) {
      // console.error('createSourceDialog called without source - add functionality removed');
      return;
    }
    
    const dialog = document.createElement('div');
    dialog.className = 'source-dialog';

    dialog.innerHTML = `
      <div class="dialog-overlay">
        <div class="dialog-content">
          <div class="dialog-header">
            <h3>Edit Feed Source</h3>
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
            <button class="btn btn-primary dialog-save">Update Source</button>
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

          await this.updateSource(source.id, sourceData);

          closeDialog();
        } catch (error) {
          // Error is already handled in add/update methods
          saveBtn.disabled = false;
          saveBtn.textContent = 'Update Source';
        }
      } else {
        form.reportValidity();
      }
    });

    return dialog;
  }

  // Add source functionality removed - sources are built-in

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
      // console.error('Failed to update feed source:', error);
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
      // console.error('Failed to remove feed source:', error);
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
      // console.error('Failed to toggle feed source:', error);
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
      // console.error('Failed to save configuration:', error);
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
      // console.error('Failed to validate feed URL:', error);
      return {
        is_valid: false,
        error_message: 'Failed to validate URL'
      };
    }
  }

  // Reset configuration to defaults
  async resetConfiguration() {
    if (!confirm('Are you sure you want to reset all settings to defaults? This cannot be undone.')) {
      return;
    }

    try {
      window.AppNavigation.showProgress('Resetting configuration...');
      await TauriAPI.config.deleteConfigFile();
      window.AppNavigation.hideProgress();
      window.AppNavigation.updateStatus('Configuration reset successfully');
      await this.loadConfiguration();
    } catch (error) {
      // console.error('Failed to reset configuration:', error);
      window.AppNavigation.hideProgress();
      window.AppNavigation.updateStatus('Failed to reset configuration');
    }
  }

  // Show topics management dialog
  async showTopicsDialog(sourceId) {
    // console.log('showTopicsDialog called with sourceId:', sourceId);
    const source = this.feedSources.find(s => s.id === sourceId);
    // console.log('Found source:', source);
    if (!source || source.source_type !== 'builtin') {
      // console.error('Source not found or not builtin:', source);
      return;
    }

    const dialog = document.createElement('div');
    dialog.className = 'source-dialog';

    const topicsCheckboxes = source.available_topics.map(topic => `
      <label class="teletext-checkbox-item">
        <input type="checkbox" value="${topic}" 
               ${source.enabled_topics.includes(topic) ? 'checked' : ''}>
        <span>${topic.toUpperCase().replace(/_/g, ' ')}</span>
      </label>
    `).join('');

    dialog.innerHTML = `
      <div class="dialog-overlay">
        <div class="dialog-content">
          <div class="dialog-header">
            <h3>MANAGE TOPICS - ${source.name.toUpperCase()}</h3>
            <button class="dialog-close">&times;</button>
          </div>
          <div class="dialog-body">
            <p class="dialog-description">SELECT TOPICS TO FETCH FROM THIS SOURCE:</p>
            <div class="topics-checkbox-list">
              ${topicsCheckboxes}
            </div>
          </div>
          <div class="dialog-footer">
            <button class="btn btn-outline dialog-cancel">CANCEL</button>
            <button class="btn btn-primary dialog-save">SAVE TOPICS</button>
          </div>
        </div>
      </div>
    `;

    document.body.appendChild(dialog);

    const closeDialog = () => document.body.removeChild(dialog);
    dialog.querySelector('.dialog-close').addEventListener('click', closeDialog);
    dialog.querySelector('.dialog-cancel').addEventListener('click', closeDialog);
    dialog.querySelector('.dialog-overlay').addEventListener('click', (e) => {
      if (e.target === e.currentTarget) closeDialog();
    });

    dialog.querySelector('.dialog-save').addEventListener('click', async () => {
      // console.log('Save button clicked');
      const checkboxes = dialog.querySelectorAll('input[type="checkbox"]:checked');
      // console.log('Checked checkboxes:', checkboxes);
      const enabledTopics = Array.from(checkboxes).map(cb => cb.value);
      // console.log('Enabled topics:', enabledTopics);

      if (enabledTopics.length === 0) {
        alert('Please select at least one topic');
        return;
      }

      try {
        // console.log('Calling updateSourceTopics with:', sourceId, enabledTopics);
        await TauriAPI.feedSources.updateSourceTopics(sourceId, enabledTopics);
        // console.log('Update successful');
        source.enabled_topics = enabledTopics;
        this.renderSettings();
        window.AppNavigation.updateStatus(`Updated topics for ${source.name}`);
        closeDialog();
      } catch (error) {
        // console.error('Failed to update topics:', error);
        alert('Failed to update topics: ' + error.message);
        window.AppNavigation.updateStatus('Failed to update topics');
      }
    });
  }

  // Show custom feed dialog (add or edit)
  showCustomFeedDialog(feedId = null) {
    const feed = feedId ? this.feedSources.find(s => s.id === feedId) : null;
    const isEdit = !!feed;

    const dialog = document.createElement('div');
    dialog.className = 'source-dialog';

    dialog.innerHTML = `
      <div class="dialog-overlay">
        <div class="dialog-content">
          <div class="dialog-header">
            <h3>${isEdit ? 'EDIT' : 'ADD'} CUSTOM FEED</h3>
            <button class="dialog-close">&times;</button>
          </div>
          <div class="dialog-body">
            <form class="source-form">
              <div class="form-group">
                <label class="form-label">NAME</label>
                <input type="text" class="form-input" name="name" 
                       value="${feed?.name || ''}" placeholder="e.g., Reuters Finance" required>
              </div>
              <div class="form-group">
                <label class="form-label">RSS/ATOM URL</label>
                <input type="url" class="form-input" name="url" 
                       value="${feed?.url || ''}" placeholder="https://example.com/feed.rss" required>
              </div>
            </form>
          </div>
          <div class="dialog-footer">
            <button class="btn btn-outline dialog-cancel">CANCEL</button>
            <button class="btn btn-primary dialog-save">${isEdit ? 'UPDATE' : 'ADD'} FEED</button>
          </div>
        </div>
      </div>
    `;

    document.body.appendChild(dialog);

    const closeDialog = () => document.body.removeChild(dialog);
    dialog.querySelector('.dialog-close').addEventListener('click', closeDialog);
    dialog.querySelector('.dialog-cancel').addEventListener('click', closeDialog);
    dialog.querySelector('.dialog-overlay').addEventListener('click', (e) => {
      if (e.target === e.currentTarget) closeDialog();
    });

    dialog.querySelector('.dialog-save').addEventListener('click', async () => {
      const form = dialog.querySelector('.source-form');
      const formData = new FormData(form);

      if (!form.checkValidity()) {
        form.reportValidity();
        return;
      }

      const name = formData.get('name');
      const url = formData.get('url');

      try {
        if (isEdit) {
          await TauriAPI.feedSources.updateCustomFeed(feedId, name, url);
          window.AppNavigation.updateStatus(`Updated custom feed: ${name}`);
        } else {
          await TauriAPI.feedSources.addCustomFeed(name, url);
          window.AppNavigation.updateStatus(`Added custom feed: ${name}`);
        }
        await this.loadConfiguration();
        closeDialog();
      } catch (error) {
        // console.error('Failed to save custom feed:', error);
        window.AppNavigation.updateStatus('Failed to save custom feed');
      }
    });
  }

  // Delete custom feed
  async deleteCustomFeed(feedId) {
    const feed = this.feedSources.find(s => s.id === feedId);
    if (!feed || !confirm(`Delete custom feed "${feed.name}"?`)) return;

    try {
      await TauriAPI.feedSources.deleteCustomFeed(feedId);
      window.AppNavigation.updateStatus(`Deleted custom feed: ${feed.name}`);
      await this.loadConfiguration();
    } catch (error) {
      // console.error('Failed to delete custom feed:', error);
      window.AppNavigation.updateStatus('Failed to delete custom feed');
    }
  }

  // Toggle custom feed
  async toggleCustomFeed(feedId, enabled) {
    try {
      await TauriAPI.feedSources.toggleCustomFeed(feedId, enabled);
      const feed = this.feedSources.find(s => s.id === feedId);
      if (feed) {
        feed.enabled = enabled;
        this.renderSettings();
        window.AppNavigation.updateStatus(`${enabled ? 'Enabled' : 'Disabled'} custom feed`);
      }
    } catch (error) {
      // console.error('Failed to toggle custom feed:', error);
      window.AppNavigation.updateStatus('Failed to update custom feed');
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