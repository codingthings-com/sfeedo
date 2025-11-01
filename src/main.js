// Application state
let currentView = 'articles';
let isLoading = false;

// Import Tauri API for external URL handling
import { TauriAPI } from './api/tauri-api.js';
import { initWindowStateTracking } from './window-state.js';

// DOM elements
let navButtons;
let views;
let refreshBtn;
let loadingOverlay;
let statusText;
let sortButtons;

// Initialize application
document.addEventListener("DOMContentLoaded", () => {
  
  // Get DOM elements
  navButtons = document.querySelectorAll('.teletext-nav-btn');
  views = document.querySelectorAll('.teletext-view');
  refreshBtn = document.getElementById('refresh-btn');
  loadingOverlay = document.getElementById('loading-overlay');
  statusText = document.getElementById('status-text');
  sortButtons = document.querySelectorAll('.teletext-sort-btn');
  
  // Set up event listeners
  setupNavigation();
  setupRefreshButton();
  setupKeyboardControls();
  
  // Set up sort buttons after a delay to ensure ArticleManager is ready
  setTimeout(() => {
    setupSortButtons();
  }, 200);
  
  // Initialize the app
  updateStatus('READY');
  showView('articles');
  
  // Initialize window state tracking
  initWindowStateTracking();
});

// Navigation handling
function setupNavigation() {
  navButtons.forEach(button => {
    button.addEventListener('click', (e) => {
      const targetView = e.currentTarget.dataset.view;
      showView(targetView);
    });
  });
}

function showView(viewName) {
  // Update navigation buttons
  navButtons.forEach(btn => {
    btn.classList.toggle('active', btn.dataset.view === viewName);
  });
  
  // Update views
  views.forEach(view => {
    view.classList.toggle('active', view.id === `${viewName}-view`);
  });
  
  // Update page info in header
  const pageInfo = document.querySelector('.teletext-page-info');
  if (pageInfo) {
    pageInfo.textContent = viewName === 'articles' ? 'P100' : 'P200';
  }
  
  currentView = viewName;
  updateStatus(viewName === 'articles' ? 'NEWS MODE' : 'SETUP MODE');
}

// Sort button handling
function setupSortButtons() {
  if (!sortButtons || sortButtons.length === 0) {
    sortButtons = document.querySelectorAll('.teletext-sort-btn');
  }
  
  sortButtons.forEach(button => {
    button.addEventListener('click', (e) => {
      const sortType = e.currentTarget.dataset.sort;
      
      // Update active state
      sortButtons.forEach(btn => btn.classList.remove('active'));
      e.currentTarget.classList.add('active');
      
      // Trigger sort in article manager
      if (window.articleManager) {
        window.articleManager.setSortOrder(sortType);
      } else {
        // Retry after a short delay if articleManager isn't ready
        setTimeout(() => {
          if (window.articleManager) {
            window.articleManager.setSortOrder(sortType);
          }
        }, 100);
      }
    });
  });
}

// Make setupSortButtons available globally so ArticleManager can call it
window.setupSortButtons = setupSortButtons;

// Keyboard controls for teletext feel
function setupKeyboardControls() {
  document.addEventListener('keydown', (e) => {
    // Number keys for page navigation
    if (e.key === '1') {
      e.preventDefault();
      showView('articles');
    } else if (e.key === '2') {
      e.preventDefault();
      showView('settings');
    }
    // Arrow keys for sorting
    else if (e.key === 'ArrowUp') {
      e.preventDefault();
      document.getElementById('sort-up-btn')?.click();
    } else if (e.key === 'ArrowDown') {
      e.preventDefault();
      document.getElementById('sort-down-btn')?.click();
    }
    // Enter key for "View Full" on focused article
    else if (e.key === 'Enter') {
      const focusedArticle = document.activeElement;
      if (focusedArticle && focusedArticle.classList.contains('teletext-article')) {
        const articleId = focusedArticle.dataset.articleId;
        if (articleId && window.articleManager) {
          window.articleManager.showArticleDetail(articleId);
        }
      }
    }
  });
}

// Refresh button handling
function setupRefreshButton() {
  refreshBtn.addEventListener('click', () => {
    if (!isLoading) {
      handleRefresh();
    }
  });
}

function handleRefresh() {
  // This will be handled by RefreshManager
  if (window.refreshManager) {
    window.refreshManager.forceRefresh();
  } else {
    // Fallback for when refresh manager isn't loaded yet
    setLoading(true, 'REFRESHING...');
    
    setTimeout(() => {
      setLoading(false);
      updateStatus('REFRESH COMPLETE');
      updateLastRefreshTime();
    }, 2000);
  }
}

// Loading state management
function setLoading(loading, message = 'LOADING...') {
  isLoading = loading;
  
  if (loading) {
    loadingOverlay.classList.remove('hidden');
    const loadingText = loadingOverlay.querySelector('.teletext-loading-text');
    if (loadingText) {
      loadingText.textContent = message.toUpperCase();
    }
    refreshBtn.disabled = true;
    refreshBtn.style.opacity = '0.5';
  } else {
    loadingOverlay.classList.add('hidden');
    refreshBtn.disabled = false;
    refreshBtn.style.opacity = '1';
  }
}

// Status management
function updateStatus(message) {
  if (statusText) {
    statusText.textContent = message.toUpperCase();
  }
}

function updateLastRefreshTime() {
  const lastUpdateElement = document.getElementById('last-update');
  if (lastUpdateElement) {
    const now = new Date();
    const timeString = now.toLocaleTimeString('en-GB', { 
      hour12: false, 
      hour: '2-digit', 
      minute: '2-digit' 
    });
    lastUpdateElement.textContent = `LAST: ${timeString}`;
  }
}

// Progress indicator utilities
function showProgress(message) {
  setLoading(true, message);
}

function hideProgress() {
  setLoading(false);
}

// Export functions for use by other modules
window.AppNavigation = {
  showView,
  setLoading,
  updateStatus,
  showProgress,
  hideProgress,
  getCurrentView: () => currentView,
  isLoading: () => isLoading,
  handleRefresh
};
