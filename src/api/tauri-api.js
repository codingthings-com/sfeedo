// Tauri API integration module
import { invoke } from '@tauri-apps/api/core';

/**
 * Article API functions
 */
export class ArticleAPI {
  /**
   * Get articles with pagination and filtering
   * @param {Object} params - Query parameters
   * @param {number} params.limit - Maximum number of articles to return
   * @param {number} params.offset - Number of articles to skip
   * @param {string} params.source_id - Filter by specific source ID

   * @returns {Promise<Object>} Articles response with pagination info
   */
  static async getArticles(params = {}) {
    try {
      const response = await invoke('get_articles', { params });
      return response;
    } catch (error) {
      console.error('Failed to get articles:', error);
      throw new Error(`Failed to load articles: ${error}`);
    }
  }

  /**
   * Get a specific article by ID
   * @param {string} id - Article ID
   * @returns {Promise<Object|null>} Article object or null if not found
   */
  static async getArticleById(id) {
    try {
      return await invoke('get_article_by_id', { id });
    } catch (error) {
      console.error('Failed to get article:', error);
      throw new Error(`Failed to load article: ${error}`);
    }
  }



  /**
   * Search articles by query
   * @param {string} query - Search query
   * @param {number} limit - Maximum number of results
   * @returns {Promise<Array>} Array of matching articles
   */
  static async searchArticles(query, limit = 50) {
    try {
      const params = { query, limit };
      return await invoke('search_articles', { params });
    } catch (error) {
      console.error('Failed to search articles:', error);
      throw new Error(`Search failed: ${error}`);
    }
  }

  /**
   * Get article count with optional filtering
   * @param {string} source_id - Optional source ID filter

   * @returns {Promise<number>} Article count
   */
  static async getArticleCount(source_id = null) {
    try {
      return await invoke('get_article_count', { source_id });
    } catch (error) {
      console.error('Failed to get article count:', error);
      throw new Error(`Failed to get article count: ${error}`);
    }
  }

  /**
   * Delete an article
   * @param {string} id - Article ID
   * @returns {Promise<boolean>} Success status
   */
  static async deleteArticle(id) {
    try {
      return await invoke('delete_article', { id });
    } catch (error) {
      console.error('Failed to delete article:', error);
      throw new Error(`Failed to delete article: ${error}`);
    }
  }
}

/**
 * Feed Source API functions
 */
export class FeedSourceAPI {
  /**
   * Get all feed sources
   * @returns {Promise<Array>} Array of feed sources
   */
  static async getFeedSources() {
    try {
      return await invoke('get_feed_sources');
    } catch (error) {
      console.error('Failed to get feed sources:', error);
      throw new Error(`Failed to load feed sources: ${error}`);
    }
  }

  /**
   * Get only enabled feed sources
   * @returns {Promise<Array>} Array of enabled feed sources
   */
  static async getEnabledFeedSources() {
    try {
      const sources = await invoke('get_feed_sources');
      return sources.filter(source => source.enabled);
    } catch (error) {
      console.error('Failed to get enabled feed sources:', error);
      throw new Error(`Failed to load enabled feed sources: ${error}`);
    }
  }

  /**
   * Update feed source enabled state
   * @param {string} sourceId - Feed source ID
   * @param {boolean} enabled - Whether the source should be enabled
   * @returns {Promise<void>} Success status
   */
  static async updateFeedSourceEnabled(sourceId, enabled) {
    try {
      return await invoke('update_feed_source_enabled', { 
        source_id: sourceId, 
        enabled 
      });
    } catch (error) {
      console.error('Failed to update feed source:', error);
      throw new Error(`Failed to update feed source: ${error}`);
    }
  }

  // Note: Add/Remove feed sources not implemented - only built-in sources can be enabled/disabled
  // /**
  //  * Remove a feed source
  //  * @param {string} id - Feed source ID
  //  * @returns {Promise<boolean>} Success status
  //  */
  // static async removeFeedSource(id) {
  //   try {
  //     return await invoke('remove_feed_source_db', { id });
  //   } catch (error) {
  //     console.error('Failed to remove feed source:', error);
  //     throw new Error(`Failed to remove feed source: ${error}`);
  //   }
  // }

  /**
   * Toggle feed source enabled status
   * @param {string} id - Feed source ID
   * @param {boolean} enabled - New enabled status
   * @returns {Promise<boolean>} Success status
   */
  static async toggleFeedSource(id, enabled) {
    try {
      return await invoke('update_feed_source_enabled', { 
        sourceId: id, 
        enabled 
      });
    } catch (error) {
      console.error('Failed to toggle feed source:', error);
      throw new Error(`Failed to toggle feed source: ${error}`);
    }
  }

  /**
   * Validate a feed URL
   * @param {string} url - Feed URL to validate
   * @param {number} timeoutSeconds - Validation timeout
   * @returns {Promise<Object>} Validation result
   */
  static async validateFeedSource(url, timeoutSeconds = 30) {
    try {
      const params = { url, timeout_seconds: timeoutSeconds };
      return await invoke('validate_feed_source', { params });
    } catch (error) {
      console.error('Failed to validate feed source:', error);
      throw new Error(`Failed to validate feed source: ${error}`);
    }
  }
}

/**
 * Configuration API functions
 */
export class ConfigAPI {
  /**
   * Get application configuration
   * @returns {Promise<Object>} Application configuration
   */
  static async getConfig() {
    try {
      return await invoke('get_config');
    } catch (error) {
      console.error('Failed to get configuration:', error);
      throw new Error(`Failed to load configuration: ${error}`);
    }
  }

  /**
   * Update application configuration
   * @param {Object} config - Configuration object
   * @returns {Promise<void>}
   */
  static async updateConfig(config) {
    try {
      return await invoke('update_config', { config });
    } catch (error) {
      console.error('Failed to update configuration:', error);
      throw new Error(`Failed to save configuration: ${error}`);
    }
  }

  /**
   * Update refresh configuration
   * @param {boolean} enabled - Whether auto-refresh is enabled
   * @param {number} intervalMinutes - Refresh interval in minutes
   * @returns {Promise<void>}
   */
  static async updateRefreshConfig(enabled, intervalMinutes) {
    try {
      const params = { enabled, interval_minutes: intervalMinutes };
      return await invoke('update_refresh_config', { params });
    } catch (error) {
      console.error('Failed to update refresh configuration:', error);
      throw new Error(`Failed to update refresh settings: ${error}`);
    }
  }

  /**
   * Reset configuration to defaults
   * @returns {Promise<void>}
   */
  static async resetConfig() {
    try {
      return await invoke('reset_config');
    } catch (error) {
      console.error('Failed to reset configuration:', error);
      throw new Error(`Failed to reset configuration: ${error}`);
    }
  }

  /**
   * Delete the config file to reset to defaults
   * @returns {Promise<string>} Result message
   */
  static async deleteConfigFile() {
    try {
      return await invoke('delete_config_file');
    } catch (error) {
      console.error('Failed to delete config file:', error);
      throw new Error(`Failed to delete config file: ${error}`);
    }
  }
}

/**
 * Refresh API functions
 */
export class RefreshAPI {
  /**
   * Manually refresh all feeds
   * @returns {Promise<Object>} Refresh result
   */
  static async refreshFeeds() {
    try {
      return await invoke('refresh_feeds');
    } catch (error) {
      console.error('Failed to refresh feeds:', error);
      throw new Error(`Failed to refresh feeds: ${error}`);
    }
  }

  /**
   * Get current refresh status
   * @returns {Promise<Object>} Refresh status
   */
  static async getRefreshStatus() {
    try {
      return await invoke('get_refresh_status');
    } catch (error) {
      console.error('Failed to get refresh status:', error);
      throw new Error(`Failed to get refresh status: ${error}`);
    }
  }

  /**
   * Get refresh progress information
   * @returns {Promise<Object>} Progress information
   */
  static async getRefreshProgress() {
    try {
      return await invoke('get_refresh_progress');
    } catch (error) {
      console.error('Failed to get refresh progress:', error);
      throw new Error(`Failed to get refresh progress: ${error}`);
    }
  }
}

/**
 * Utility functions for external operations
 */
export class UtilityAPI {
  /**
   * Open URL in external browser
   * @param {string} url - URL to open
   * @returns {Promise<void>}
   */
  static async openExternalUrl(url) {
    try {
      // Validate URL first
      if (!url || typeof url !== 'string') {
        throw new Error('Invalid URL provided');
      }
      
      // Ensure URL has protocol
      let validUrl = url;
      if (!url.startsWith('http://') && !url.startsWith('https://')) {
        validUrl = 'https://' + url;
      }
      
      // Open in new tab/window
      const newWindow = window.open(validUrl, '_blank', 'noopener,noreferrer');
      
      // Check if popup was blocked
      if (!newWindow || newWindow.closed || typeof newWindow.closed === 'undefined') {
        // Popup was likely blocked, show user the URL
        const userWantsToOpen = confirm(`Popup blocked. Open this URL manually?\n\n${validUrl}`);
        if (userWantsToOpen) {
          // Try to copy to clipboard if possible
          try {
            await navigator.clipboard.writeText(validUrl);
            alert('URL copied to clipboard!');
          } catch (clipboardError) {
            // Show URL for manual copying
            prompt('Copy this URL:', validUrl);
          }
        }
      }
    } catch (error) {
      console.error('Failed to open external URL:', error);
      // Show URL to user as fallback
      const userWantsToOpen = confirm(`Could not open URL automatically. Open manually?\n\n${url}`);
      if (userWantsToOpen) {
        try {
          await navigator.clipboard.writeText(url);
          alert('URL copied to clipboard!');
        } catch (clipboardError) {
          prompt('Copy this URL:', url);
        }
      }
    }
  }
}

// Export all APIs as a single object for convenience
export const TauriAPI = {
  articles: ArticleAPI,
  feedSources: FeedSourceAPI,
  config: ConfigAPI,
  refresh: RefreshAPI,
  utility: UtilityAPI,
};

export default TauriAPI;