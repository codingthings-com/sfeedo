// Article display and management functionality
import { TauriAPI } from '../api/tauri-api.js';

class ArticleManager {
    constructor() {
        this.articles = [];
        this.currentSort = 'newest';
        this.selectedArticle = null;
        this.totalCount = 0;
        this.isLoading = false;

        this.initializeElements();
        this.setupEventListeners();
    }

    initializeElements() {
        this.container = document.getElementById('articles-container');
        this.sortUpBtn = document.getElementById('sort-up-btn');
        this.sortDownBtn = document.getElementById('sort-down-btn');
    }

    setupEventListeners() {
        // Sort button handlers are set up in main.js
        // This method is kept for compatibility
    }

    // New method to set sort order from external calls
    setSortOrder(sortType) {
        this.currentSort = sortType;
        this.renderArticles();

        // Update button states
        const sortButtons = document.querySelectorAll('.teletext-sort-btn');
        sortButtons.forEach(btn => {
            btn.classList.toggle('active', btn.dataset.sort === sortType);
        });
    }

    // Load articles from Tauri backend
    async loadArticles(silent = false) {
        if (this.isLoading) return;

        try {
            this.isLoading = true;

            // Only show loading overlay if not silent
            if (!silent) {
                window.AppNavigation.showProgress('Loading articles...');
            }

            // Call Tauri backend to get all articles
            const response = await TauriAPI.articles.getArticles({});

            // Update local state with all articles
            this.articles = response.articles;
            this.totalCount = response.total_count;

            this.renderArticles();

            // Only update status if not silent
            if (!silent) {
                window.AppNavigation.updateStatus(`LOADED ${this.articles.length} ARTICLES`);
            }
        } catch (error) {
            // console.error('Failed to load articles:', error);
            window.AppNavigation.updateStatus('LOAD FAILED');
            this.showError('Unable to load articles. Please try refreshing.');
        } finally {
            this.isLoading = false;

            // Only hide progress if not silent
            if (!silent) {
                window.AppNavigation.hideProgress();
            }
        }
    }

    // Utility function to escape HTML
    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }



    // Sort articles based on current sort setting
    getSortedArticles() {
        const sorted = [...this.articles];

        switch (this.currentSort) {
            case 'newest':
                return sorted.sort((a, b) => new Date(b.published_at) - new Date(a.published_at));
            case 'oldest':
                return sorted.sort((a, b) => new Date(a.published_at) - new Date(b.published_at));
            default:
                return sorted;
        }
    }

    // Render articles in the container
    renderArticles() {
        const sortedArticles = this.getSortedArticles();

        if (sortedArticles.length === 0) {
            this.showEmptyState();
            return;
        }

        const articlesHTML = sortedArticles.map(article => this.createArticleHTML(article)).join('');
        this.container.innerHTML = articlesHTML;

        // Add event listeners to article elements
        this.attachArticleEventListeners();
    }

    // Create HTML for a single article - teletext style
    createArticleHTML(article) {
        const timeAgo = this.getTimeAgo(article.published_at);
        const source = (article.source_id || 'UNKNOWN').toUpperCase();
        const title = this.escapeHtml(article.title).toUpperCase();
        const summary = this.escapeHtml(article.summary || '').toUpperCase();

        return `
      <article class="teletext-article" data-article-id="${article.id}" tabindex="0">
        <div class="teletext-article-header">
          <span class="teletext-article-source">${source}</span>
          <span class="teletext-article-time">${timeAgo}</span>
        </div>
        <div class="teletext-article-title">${title}</div>
        <div class="teletext-article-content">${summary}</div>
      </article>
    `;
    }

    // Attach event listeners to article elements
    attachArticleEventListeners() {
        const articleItems = this.container.querySelectorAll('.teletext-article');

        articleItems.forEach(item => {
            const articleId = item.dataset.articleId;

            // Click on article content area to view details
            item.addEventListener('click', (e) => {
                this.showArticleDetail(articleId);
            });

            // Enter key on focused article
            item.addEventListener('keydown', (e) => {
                if (e.key === 'Enter') {
                    e.preventDefault();
                    this.showArticleDetail(articleId);
                }
            });
        });
    }

    // Handle article actions
    handleArticleAction(articleId, action) {
        const article = this.articles.find(a => a.id === articleId);
        if (!article) return;

        switch (action) {
            case 'view-full':
                this.showArticleDetail(articleId);
                break;
            case 'open-external':
                this.openExternalUrl(article.url);
                break;
        }
    }



    // Show article detail view
    async showArticleDetail(articleId) {
        const article = this.articles.find(a => a.id === articleId);
        if (!article) return;

        this.selectedArticle = article;
        this.showArticleModal(article);
    }

    // Show article in modal - teletext style
    showArticleModal(article) {
        const modal = this.createArticleModal(article);
        document.body.appendChild(modal);

        // Add event listeners
        const closeBtn = modal.querySelector('.teletext-modal-close');
        const overlay = modal.querySelector('.teletext-modal-overlay');

        const closeModal = () => {
            document.body.removeChild(modal);
        };

        closeBtn.addEventListener('click', closeModal);
        overlay.addEventListener('click', (e) => {
            if (e.target === overlay) closeModal();
        });

        // ESC key to close
        const handleKeyDown = (e) => {
            if (e.key === 'Escape') {
                closeModal();
                document.removeEventListener('keydown', handleKeyDown);
            }
        };
        document.addEventListener('keydown', handleKeyDown);
    }

    // Create article modal HTML - teletext style
    createArticleModal(article) {
        const modal = document.createElement('div');
        modal.className = 'teletext-modal';

        const timeAgo = this.getTimeAgo(article.published_at);
        const source = (article.source_id || 'UNKNOWN').toUpperCase();
        const title = this.escapeHtml(article.title).toUpperCase();
        const summary = this.escapeHtml(article.summary || '').toUpperCase();
        const content = this.escapeHtml(article.content || 'FULL CONTENT NOT AVAILABLE. CLICK "OPEN EXTERNAL LINK" TO VIEW ON SOURCE WEBSITE.').toUpperCase();

        modal.innerHTML = `
      <div class="teletext-modal-overlay">
        <div class="teletext-modal-content">
          <div class="teletext-modal-header">
            <div class="teletext-modal-meta">
              <span class="teletext-modal-source">${source}</span>
              <span class="teletext-modal-time">${timeAgo}</span>
            </div>
            <button class="teletext-modal-close">X</button>
          </div>
          <div class="teletext-modal-body">
            <h2 class="teletext-modal-title">${title}</h2>
            <div class="teletext-modal-article-content">
              <p class="teletext-modal-summary">${summary}</p>
              <div class="teletext-modal-full-content">
                ${content}
              </div>
            </div>
          </div>
          <div class="teletext-modal-footer">
            <button class="teletext-btn teletext-btn-primary" data-url="${article.url}">
              OPEN EXTERNAL LINK
            </button>
          </div>
        </div>
      </div>
    `;

        // Add click handler for the external link button
        const externalBtn = modal.querySelector('.teletext-btn-primary');
        externalBtn.addEventListener('click', () => {
            this.openExternalUrl(article.url);
        });

        return modal;
    }

    // Open external URL
    async openExternalUrl(url) {
        try {
            await TauriAPI.utility.openExternalUrl(url);
        } catch (error) {
            // console.error('Failed to open external URL:', error);
            // Fallback to window.open
            window.open(url, '_blank');
        }
    }

    // Show empty state - teletext style
    showEmptyState() {
        this.container.innerHTML = `
      <div class="teletext-empty-state">
        <div class="teletext-empty-icon">[ ]</div>
        <h3>NO ARTICLES FOUND</h3>
        <p>NO ARTICLES AVAILABLE. TRY REFRESHING FEEDS OR CHECK FEED SOURCES.</p>
        <button class="teletext-btn teletext-btn-primary" onclick="if(window.refreshManager) window.refreshManager.forceRefresh()">
          REFRESH FEEDS
        </button>
      </div>
    `;
    }

    // Show error message - teletext style
    showError(message) {
        this.container.innerHTML = `
      <div class="teletext-error-state">
        <div class="teletext-error-icon">[!]</div>
        <h3>ERROR LOADING ARTICLES</h3>
        <p>${message.toUpperCase()}</p>
        <button class="teletext-btn teletext-btn-primary" onclick="articleManager.loadArticles()">
          TRY AGAIN
        </button>
      </div>
    `;
    }

    // Utility function to get time ago string - teletext style
    getTimeAgo(date) {
        const now = new Date();
        const articleDate = new Date(date);
        
        // Check if article is from today
        const isToday = now.toDateString() === articleDate.toDateString();
        
        if (isToday) {
            // Show relative time for today's articles
            const diffMs = now - articleDate;
            const diffMins = Math.floor(diffMs / (1000 * 60));
            const diffHours = Math.floor(diffMs / (1000 * 60 * 60));

            if (diffMins < 1) return 'NOW';
            if (diffMins < 60) return `${diffMins}M`;
            return `${diffHours}H`;
        }
        
        return articleDate.toLocaleString('en-US');
    }

    // Refresh articles
    async refresh() {
        await this.loadArticles(); // Reload all articles
    }



    // Add real-time update capability
    onArticlesUpdated(callback) {
        // This could be enhanced with Tauri events for real-time updates
        this.updateCallback = callback;
    }

    // Trigger update callback if set
    triggerUpdate() {
        if (this.updateCallback) {
            this.updateCallback();
        }
    }


}

// Initialize article manager when DOM is ready
let articleManager;

document.addEventListener('DOMContentLoaded', () => {
    articleManager = new ArticleManager();

    // Make it available globally immediately
    window.articleManager = articleManager;

    // Set up sort buttons now that ArticleManager is ready
    if (window.setupSortButtons) {
        window.setupSortButtons();
    }

    // Show loading overlay immediately via DOM so the user never sees an empty screen.
    // We avoid showProgress() here because it sets isLoading which blocks the refresh button.
    const overlay = document.getElementById('loading-overlay');
    if (overlay) {
        overlay.classList.remove('hidden');
        const text = overlay.querySelector('.teletext-loading-text');
        if (text) text.textContent = 'REFRESHING FEEDS...';
    }

    // Wait for refreshManager to be ready, then trigger initial refresh.
    // refreshManager is created by refresh.js which loads after articles.js,
    // so we poll until it's available.
    const waitForRefreshManager = () => {
        if (window.refreshManager) {
            // Hide the temporary overlay — refreshManager.startRefresh() shows its own
            if (overlay) overlay.classList.add('hidden');
            window.refreshManager.forceRefresh();
        } else {
            setTimeout(waitForRefreshManager, 50);
        }
    };
    waitForRefreshManager();

    // Set up global refresh handler
    window.refreshArticles = () => articleManager.refresh();
});

// Export for global access
window.ArticleManager = ArticleManager;