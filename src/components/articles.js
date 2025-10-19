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
        this.sortSelect = document.getElementById('sort-select');
    }

    setupEventListeners() {
        // Sort change handler
        this.sortSelect.addEventListener('change', (e) => {
            this.currentSort = e.target.value;
            this.renderArticles();
        });
    }

    // Load articles from Tauri backend
    async loadArticles() {
        if (this.isLoading) return;
        
        try {
            this.isLoading = true;
            window.AppNavigation.showProgress('Loading articles...');

            // Call Tauri backend to get all articles
            const response = await TauriAPI.articles.getArticles({});
            
            // Update local state with all articles
            this.articles = response.articles;
            this.totalCount = response.total_count;

            this.renderArticles();
            window.AppNavigation.updateStatus(`Loaded ${this.articles.length} articles`);
        } catch (error) {
            console.error('Failed to load articles:', error);
            window.AppNavigation.updateStatus('Failed to load articles');
            this.showError('Unable to load articles. Please try refreshing.');
        } finally {
            this.isLoading = false;
            window.AppNavigation.hideProgress();
        }
    }

    // Utility function to escape HTML
    escapeHtml(text) {
        if (!text) return '';
        const div = document.createElement('div');
        div.textContent = text;
        return div.innerHTML;
    }



    // Search articles
    async searchArticles(query) {
        if (!query.trim()) {
            // If empty query, reload all articles
            await this.loadArticles();
            return;
        }

        try {
            this.isLoading = true;
            window.AppNavigation.showProgress('Searching articles...');

            const results = await TauriAPI.articles.searchArticles(query, 100);
            this.articles = results;
            this.totalCount = results.length;

            
            this.renderArticles();
            window.AppNavigation.updateStatus(`Found ${results.length} articles matching "${query}"`);
        } catch (error) {
            console.error('Failed to search articles:', error);
            window.AppNavigation.updateStatus('Search failed');
            this.showError('Search failed. Please try again.');
        } finally {
            this.isLoading = false;
            window.AppNavigation.hideProgress();
        }
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
        const startIndex = (this.currentPage - 1) * this.articlesPerPage;
        const endIndex = startIndex + this.articlesPerPage;
        const pageArticles = sortedArticles.slice(startIndex, endIndex);

        if (sortedArticles.length === 0) {
            this.showEmptyState();
            return;
        }

        const articlesHTML = sortedArticles.map(article => this.createArticleHTML(article)).join('');
        this.container.innerHTML = articlesHTML;



        // Add event listeners to article elements
        this.attachArticleEventListeners();
    }

    // Create HTML for a single article
    createArticleHTML(article) {
        const timeAgo = this.getTimeAgo(article.published_at);
        const source = article.source_id || 'Unknown Source';

        return `
      <article class="article-item" data-article-id="${article.id}">
        <div class="article-header">
          <div class="article-meta">
            <span class="article-source">${source}</span>
            <span class="article-time">${timeAgo}</span>
          </div>
        </div>
        <div class="article-content">
          <h3 class="article-title">${this.escapeHtml(article.title)}</h3>
          <p class="article-summary">${this.escapeHtml(article.summary || '')}</p>
        </div>
        <div class="article-actions">
          <button class="action-btn view-btn" data-action="view-full">
            View Full
          </button>
          <button class="action-btn external-btn" data-action="open-external">
            Open Source
          </button>
        </div>
      </article>
    `;
    }

    // Attach event listeners to article elements
    attachArticleEventListeners() {
        const articleItems = this.container.querySelectorAll('.article-item');

        articleItems.forEach(item => {
            const articleId = item.dataset.articleId;

            // Click on article to view details
            item.addEventListener('click', (e) => {
                if (!e.target.closest('.article-actions')) {
                    this.showArticleDetail(articleId);
                }
            });

            // Action buttons
            const actionButtons = item.querySelectorAll('[data-action]');
            actionButtons.forEach(btn => {
                btn.addEventListener('click', (e) => {
                    e.stopPropagation();
                    const action = btn.dataset.action;
                    this.handleArticleAction(articleId, action);
                });
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

    // Show article in modal
    showArticleModal(article) {
        const modal = this.createArticleModal(article);
        document.body.appendChild(modal);

        // Add event listeners
        const closeBtn = modal.querySelector('.modal-close');
        const overlay = modal.querySelector('.modal-overlay');

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

    // Create article modal HTML
    createArticleModal(article) {
        const modal = document.createElement('div');
        modal.className = 'article-modal';

        const timeAgo = this.getTimeAgo(article.published_at);
        const source = article.source_id || 'Unknown Source';

        modal.innerHTML = `
      <div class="modal-overlay">
        <div class="modal-content">
          <div class="modal-header">
            <div class="modal-meta">
              <span class="modal-source">${this.escapeHtml(source)}</span>
              <span class="modal-time">${timeAgo}</span>
            </div>
            <button class="modal-close">&times;</button>
          </div>
          <div class="modal-body">
            <h2 class="modal-title">${this.escapeHtml(article.title)}</h2>
            <div class="modal-article-content">
              <p class="modal-summary">${this.escapeHtml(article.summary || '')}</p>
              <div class="modal-full-content">
                ${this.escapeHtml(article.content || 'Full content not available. Click "Read Full Article" to view on the source website.')}
              </div>
            </div>
          </div>
          <div class="modal-footer">
            <button class="btn btn-primary" data-url="${article.url}">
              Read Full Article
            </button>
          </div>
        </div>
      </div>
    `;

        // Add click handler for the external link button
        const externalBtn = modal.querySelector('.btn-primary');
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
            console.error('Failed to open external URL:', error);
            // Fallback to window.open
            window.open(url, '_blank');
        }
    }

    // Show empty state
    showEmptyState() {
        this.container.innerHTML = `
      <div class="empty-state">
        <div class="empty-icon">📰</div>
        <h3>No Articles Found</h3>
        <p>No articles are available. Try refreshing your feeds or check your feed sources.</p>
        <button class="btn btn-primary" onclick="window.AppNavigation.handleRefresh()">
          Refresh Feeds
        </button>
      </div>
    `;
    }

    // Show error message
    showError(message) {
        this.container.innerHTML = `
      <div class="error-state">
        <div class="error-icon">⚠️</div>
        <h3>Error Loading Articles</h3>
        <p>${message}</p>
        <button class="btn btn-primary" onclick="articleManager.loadArticles()">
          Try Again
        </button>
      </div>
    `;
    }

    // Utility function to get time ago string
    getTimeAgo(date) {
        const now = new Date();
        const diffMs = now - new Date(date);
        const diffMins = Math.floor(diffMs / (1000 * 60));
        const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
        const diffDays = Math.floor(diffMs / (1000 * 60 * 60 * 24));

        if (diffMins < 1) return 'Just now';
        if (diffMins < 60) return `${diffMins}m ago`;
        if (diffHours < 24) return `${diffHours}h ago`;
        if (diffDays < 7) return `${diffDays}d ago`;

        return new Date(date).toLocaleDateString();
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

    // Load articles initially after a short delay to ensure UI is ready
    setTimeout(() => {
        articleManager.loadArticles();
    }, 100);

    // Set up global refresh handler
    window.refreshArticles = () => articleManager.refresh();
});

// Export for global access
window.ArticleManager = ArticleManager;
window.articleManager = articleManager;