use tauri::{GlobalWindowEvent, Manager};
use tauri_plugin_window_state::AppHandleExt;

use crate::{constants::{LOGS_WINDOW_LABEL, METER_WINDOW_LABEL, WINDOW_STATE_FLAGS}, utils::unload_driver};

pub fn on_window_event(event: GlobalWindowEvent) {
    match event.event() {
        tauri::WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            if event.window().label() == METER_WINDOW_LABEL {
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
                unload_driver();
                app_handle.exit(0);
            } else if event.window().label() == LOGS_WINDOW_LABEL {
                event.window().hide().unwrap();
            }
        }
        tauri::WindowEvent::Focused(focused) => {
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