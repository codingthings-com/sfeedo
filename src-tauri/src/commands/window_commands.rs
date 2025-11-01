use crate::services::ConfigurationService;
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager, PhysicalPosition, PhysicalSize, Monitor};

#[derive(Debug, Serialize, Deserialize)]
pub struct WindowStateParams {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Save the current window position and size
#[tauri::command]
pub async fn save_window_state(
    params: WindowStateParams,
    app_handle: AppHandle,
) -> Result<(), String> {
    let service = ConfigurationService::new(&app_handle)?;
    let mut config = service.get_app_config()?;

    config.window_state.x = Some(params.x);
    config.window_state.y = Some(params.y);
    config.window_state.width = Some(params.width);
    config.window_state.height = Some(params.height);

    service.update_app_config(config)?;
    Ok(())
}

/// Get the saved window state
#[tauri::command]
pub async fn get_window_state(app_handle: AppHandle) -> Result<WindowStateParams, String> {
    let service = ConfigurationService::new(&app_handle)?;
    let config = service.get_app_config()?;

    Ok(WindowStateParams {
        x: config.window_state.x.unwrap_or(100),
        y: config.window_state.y.unwrap_or(100),
        width: config.window_state.width.unwrap_or(400),
        height: config.window_state.height.unwrap_or(1000),
    })
}

/// Check if a position is visible on any monitor
fn is_position_visible(x: i32, y: i32, width: u32, height: u32, monitors: &[Monitor]) -> bool {
    // Check if at least part of the window would be visible on any monitor
    for monitor in monitors {
        let monitor_pos = monitor.position();
        let monitor_size = monitor.size();
        
        let monitor_x = monitor_pos.x;
        let monitor_y = monitor_pos.y;
        let monitor_width = monitor_size.width as i32;
        let monitor_height = monitor_size.height as i32;
        
        // Check if window overlaps with this monitor
        // Window is visible if it overlaps by at least 100 pixels
        let overlap_x = (x + width as i32).max(monitor_x) - x.max(monitor_x);
        let overlap_y = (y + height as i32).max(monitor_y) - y.max(monitor_y);
        
        if overlap_x >= 100 && overlap_y >= 100 {
            return true;
        }
    }
    
    false
}

/// Restore window position and size from saved state
pub fn restore_window_state(app_handle: &AppHandle) -> Result<(), String> {
    let service = ConfigurationService::new(app_handle)?;
    let config = service.get_app_config()?;

    if let Some(window) = app_handle.get_webview_window("main") {
        // Set size if saved (with reasonable bounds)
        let (safe_width, safe_height) = if let (Some(width), Some(height)) = 
            (config.window_state.width, config.window_state.height) {
            // Ensure size is within reasonable bounds
            let w = width.max(300).min(3840); // Min 300, max 4K width
            let h = height.max(700).min(2160); // Min 700, max 4K height
            let _ = window.set_size(PhysicalSize::new(w, h));
            (w, h)
        } else {
            (400, 1000) // Default size
        };

        // Set position if saved (with validation against available monitors)
        if let (Some(x), Some(y)) = (config.window_state.x, config.window_state.y) {
            // Get available monitors
            match window.available_monitors() {
                Ok(monitors) => {
                    if !monitors.is_empty() && is_position_visible(x, y, safe_width, safe_height, &monitors) {
                        let _ = window.set_position(PhysicalPosition::new(x, y));
                    } else {
                        // Position is off-screen, center the window
                        log::warn!(
                            "Saved window position ({}, {}) is not visible on any monitor, centering window", 
                            x, y
                        );
                        let _ = window.center();
                    }
                }
                Err(e) => {
                    log::warn!("Failed to get monitors: {}, using saved position anyway", e);
                    let _ = window.set_position(PhysicalPosition::new(x, y));
                }
            }
        }
    }

    Ok(())
}
