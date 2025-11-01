// Window state management - save position and size
import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

let saveTimeout = null;

async function saveWindowState() {
    try {
        const window = getCurrentWindow();
        const position = await window.outerPosition();
        const size = await window.outerSize();

        await invoke('save_window_state', {
            params: {
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
            }
        });
    } catch (error) {
        console.error('Failed to save window state:', error);
    }
}

// Debounced save - don't save on every pixel movement
function debouncedSave() {
    if (saveTimeout) {
        clearTimeout(saveTimeout);
    }
    saveTimeout = setTimeout(saveWindowState, 500);
}

// Initialize window state tracking
export async function initWindowStateTracking() {
    try {
        const window = getCurrentWindow();

        // Save on window move
        await window.onMoved(() => {
            debouncedSave();
        });

        // Save on window resize
        await window.onResized(() => {
            debouncedSave();
        });

        // Note: We don't save on close to avoid blocking the close event
        // The debounced saves from move/resize will capture the final state

        console.log('Window state tracking initialized');
    } catch (error) {
        console.error('Failed to initialize window state tracking:', error);
    }
}
