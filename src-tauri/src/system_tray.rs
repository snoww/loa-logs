use tauri::{AppHandle, CustomMenuItem, LogicalPosition, LogicalSize, Manager, Position, Size, SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem};
use tauri_plugin_window_state::{AppHandleExt, WindowExt};

use crate::{constants::{LOGS_WINDOW_LABEL, METER_WINDOW_LABEL, WINDOW_STATE_FLAGS}, utils::unload_driver};


pub fn create_system_tray() -> SystemTray {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show_logs = CustomMenuItem::new("show-logs".to_string(), "Show Logs");
    let show_meter = CustomMenuItem::new("show-meter".to_string(), "Show Meter");
    let hide_meter = CustomMenuItem::new("hide".to_string(), "Hide Meter");
    let load_saved_pos = CustomMenuItem::new("load".to_string(), "Load Saved");
    let save_current_pos = CustomMenuItem::new("save".to_string(), "Save Position");
    let reset = CustomMenuItem::new("reset".to_string(), "Reset Window");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show_logs)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show_meter)
        .add_item(hide_meter)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(save_current_pos)
        .add_item(load_saved_pos)
        .add_item(reset)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);


    SystemTray::new().with_menu(tray_menu)
}

pub fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    match event {
        SystemTrayEvent::LeftClick {
            position: _,
            size: _,
            ..
        } => {
            if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                meter.show().unwrap();
                meter.unminimize().unwrap();
                meter.set_ignore_cursor_events(false).unwrap()
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => {
                app.save_window_state(WINDOW_STATE_FLAGS)
                    .expect("failed to save window state");
                unload_driver();
                app.exit(0);
            }
            "hide" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter.hide().unwrap();
                }
            }
            "show-meter" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter.show().unwrap();
                    meter.unminimize().unwrap();
                    meter.set_ignore_cursor_events(false).unwrap()
                }
            }
            "load" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter.restore_state(WINDOW_STATE_FLAGS).unwrap();
                }
            }
            "save" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter
                        .app_handle()
                        .save_window_state(WINDOW_STATE_FLAGS)
                        .unwrap();
                }
            }
            "reset" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter
                        .set_size(Size::Logical(LogicalSize {
                            width: 500.0,
                            height: 350.0,
                        }))
                        .unwrap();
                    meter
                        .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                        .unwrap();
                    meter.show().unwrap();
                    meter.unminimize().unwrap();
                    meter.set_focus().unwrap();
                    meter.set_ignore_cursor_events(false).unwrap();
                }
            }
            "show-logs" => {
                if let Some(logs) = app.get_window(LOGS_WINDOW_LABEL) {
                    logs.show().unwrap();
                    logs.unminimize().unwrap();
                }
            }
            _ => {}
        },
        _ => {}
    }
}