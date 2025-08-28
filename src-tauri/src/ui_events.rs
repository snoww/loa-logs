use tauri::{AppHandle, GlobalWindowEvent, WindowEvent};
use tauri::{CustomMenuItem, LogicalPosition, LogicalSize, Manager, Position, Size, State,
    SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem,
};
use tauri_plugin_window_state::AppHandleExt;

use crate::constants::*;
use crate::parser::models::Settings;
use crate::settings::SettingsManager;
use crate::shell::ShellManager;

pub fn create_system_tray_menu() -> SystemTray {
    let tray_menu = SystemTrayMenu::new()
        .add_item(CustomMenuItem::new("show-logs".to_string(), "Show Logs"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("show-meter".to_string(), "Show Meter"))
        .add_item(CustomMenuItem::new("hide".to_string(), "Hide Meter"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new(
            "start-loa".to_string(),
            "Start Lost Ark",
        ))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("reset".to_string(), "Reset Window"))
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(CustomMenuItem::new("quit".to_string(), "Quit"));

    let system_tray = SystemTray::new().with_menu(tray_menu);

    system_tray
}

pub fn on_system_tray_event(app: &AppHandle, event: SystemTrayEvent) {
    let settings_manager = app.state::<SettingsManager>();
    let settings = settings_manager.read().expect("Could not read settings");

    let show_window = |window: &tauri::Window| {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
        if window.label() == "main" {
            window.set_ignore_cursor_events(false).unwrap();
        }
    };

    let get_meter_window =
        |app: &tauri::AppHandle, settings: &Settings| -> Option<tauri::Window> {
            let label = if settings.general.mini {
                METER_MINI_WINDOW_LABEL
            } else {
                METER_WINDOW_LABEL
            };
            app.get_window(label)
        };

    match event {
        SystemTrayEvent::LeftClick { .. } => {
            if let Some(meter) = get_meter_window(app, &settings) {
                show_window(&meter);
            }
        }
        SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
            "quit" => {
                app.save_window_state(WINDOW_STATE_FLAGS)
                    .expect("failed to save window state");
                
                let manager = app.state::<ShellManager>();
                manager.unload_driver();
                
                app.exit(0);
            }
            "hide" => {
                if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter.hide().unwrap();
                }
                if let Some(mini) = app.get_window(METER_MINI_WINDOW_LABEL) {
                    mini.hide().unwrap();
                }
            }
            "show-meter" => {
                if let Some(meter) = get_meter_window(app, &settings) {
                    show_window(&meter);
                }
            }
            "reset" => {
                if settings.general.mini {
                    if let Some(mini) = app.get_window(METER_MINI_WINDOW_LABEL) {
                        mini.set_size(Size::Logical(LogicalSize {
                            width: 1280.0,
                            height: 200.0,
                        }))
                        .unwrap();
                        mini.set_position(Position::Logical(LogicalPosition {
                            x: 100.0,
                            y: 100.0,
                        }))
                        .unwrap();
                        show_window(&mini);
                    }
                } else if let Some(meter) = app.get_window(METER_WINDOW_LABEL) {
                    meter
                        .set_size(Size::Logical(LogicalSize {
                            width: 500.0,
                            height: 350.0,
                        }))
                        .unwrap();
                    meter
                        .set_position(Position::Logical(LogicalPosition {
                            x: 100.0,
                            y: 100.0,
                        }))
                        .unwrap();
                    show_window(&meter);
                }
            }
            "show-logs" => {
                if let Some(logs) = app.get_window(LOGS_WINDOW_LABEL) {
                    logs.show().unwrap();
                    logs.unminimize().unwrap();
                }
            }
            "start-loa" => {
                let manager = app.state::<ShellManager>();
                manager.start_loa_process();
            }
            _ => {}
        },
        _ => {}
    }
}

pub fn on_window_event(event: GlobalWindowEvent) {
    match event.event() {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            if event.window().label() == METER_WINDOW_LABEL
                || event.window().label() == METER_MINI_WINDOW_LABEL
            {
                let app_handle = event.window().app_handle();
                let meter_window = app_handle.get_window(METER_WINDOW_LABEL).unwrap();
                let logs_window = app_handle.get_window(LOGS_WINDOW_LABEL).unwrap();

                if logs_window.is_minimized().unwrap() {
                    logs_window.unminimize().unwrap();
                }

                if meter_window.is_minimized().unwrap() {
                    meter_window.unminimize().unwrap();
                }

                app_handle
                    .save_window_state(WINDOW_STATE_FLAGS)
                    .expect("failed to save window state");
                
                let manager = app_handle.state::<ShellManager>();
                manager.unload_driver();

                app_handle.exit(0);
            } else if event.window().label() == LOGS_WINDOW_LABEL {
                event.window().hide().unwrap();
            }
        }
        WindowEvent::Focused(focused) => {
            if !focused {
                event
                    .window()
                    .app_handle()
                    .save_window_state(WINDOW_STATE_FLAGS)
                    .expect("failed to save window state");
            }
        }
        _ => {}
    }
}