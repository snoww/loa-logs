use tauri::Manager;

use crate::constants::{LOGS_WINDOW_LABEL, METER_WINDOW_LABEL};

#[tauri::command]
pub fn toggle_meter_window(window: tauri::Window) {
    if let Some(meter) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        if meter.is_visible().unwrap() {
            // workaround for tauri not handling minimized state for windows without decorations
            if meter.is_minimized().unwrap() {
                meter.unminimize().unwrap();
            }
            meter.hide().unwrap();
        } else {
            meter.show().unwrap();
        }
    }
}

#[tauri::command]
pub fn toggle_logs_window(window: tauri::Window) {
    if let Some(logs) = window.app_handle().get_window(LOGS_WINDOW_LABEL) {
        if logs.is_visible().unwrap() {
            logs.hide().unwrap();
        } else {
            logs.emit("redirect-url", "logs").unwrap();
            logs.show().unwrap();
        }
    }
}