// Application state
let currentView = 'articles';
let isLoading = false;

// Import Tauri API for external URL handling
import { TauriAPI } from './api/tauri-api.js';

// DOM elements
let navButtons;
let views;
let refreshBtn;
let loadingOverlay;
let statusText;

// Initialize application
document.addEventListener("DOMContentLoaded", () => {
  console.log("Sfeedo Desktop Feed Reader initialized");
  
  // Get DOM elements
  navButtons = document.querySelectorAll('.nav-btn');
  views = document.querySelectorAll('.view');
  refreshBtn = document.getElementById('refresh-btn');
  loadingOverlay = document.getElementById('loading-overlay');
  statusText = document.getElementById('status-text');
  
  // Set up event listeners
  setupNavigation();
  setupRefreshButton();
  
  // Initialize the app
  updateStatus('Ready');
  showView('articles');
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
  
  currentView = viewName;
  updateStatus(`Viewing ${viewName}`);
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
    setLoading(true, 'Refreshing feeds...');
    
    setTimeout(() => {
      setLoading(false);
      updateStatus('Feeds refreshed successfully');
      updateLastRefreshTime();
    }, 2000);
  }
}

// Loading state management
function setLoading(loading, message = 'Loading...') {
  isLoading = loading;
  
  if (loading) {
    loadingOverlay.classList.remove('hidden');
    loadingOverlay.querySelector('.loading-text').textContent = message;
    refreshBtn.disabled = true;
    refreshBtn.querySelector('.refresh-icon').classList.add('spinning');
  } else {
    loadingOverlay.classList.add('hidden');
    refreshBtn.disabled = false;
    refreshBtn.querySelector('.refresh-icon').classList.remove('spinning');
  }
}

// Status management
function updateStatus(message) {
  statusText.textContent = message;
}

function updateLastRefreshTime() {
  const lastUpdateElement = document.getElementById('last-update');
  const now = new Date();
  const timeString = now.toLocaleTimeString();
  lastUpdateElement.textContent = `Last updated: ${timeString}`;
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
