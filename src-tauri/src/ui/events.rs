use std::str::FromStr;

use anyhow::Result;
use log::*;
use tauri::{
    menu::MenuEvent, tray::{MouseButton, MouseButtonState, TrayIcon, TrayIconEvent}, AppHandle, Manager,
    Window,
    WindowEvent,
};
use tauri_plugin_window_state::AppHandleExt;

use crate::{
    background::BackgroundWorker, constants::*, settings::SettingsManager, shell::ShellManager, ui::{AppHandleExtensions, TrayCommand, WindowExtensions}
};

/// Runs an async future to completion from a synchronous callback.
///
/// Intended for use in Tauri tray or window event handlers where `.await` cannot be used.
///
/// Calling `Handle::current().block_on` or `async_runtime::block_on` in this context
/// would panic because the callback is already running on a Tokio runtime thread.
/// This function executes the future safely by:
/// 1. Using `tokio::task::block_in_place` to temporarily block the current thread
///    while allowing the runtime to continue scheduling other tasks.
/// 2. Creating a temporary single-threaded (current-thread) Tokio runtime to run
///    the future without interfering with the existing runtime.
pub fn block_on_local<F, T>(future: F) -> T
where
    F: Future<Output = T>,
{
    tokio::task::block_in_place(|| {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(future)
    })
}

pub fn on_tray_icon_event(tray: &TrayIcon, event: TrayIconEvent) {
    {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app_handle = tray.app_handle();
            if let Some(meter) = app_handle.get_meter_window() {
                meter.restore_and_focus();
            }
        }
    }
}

pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    if let Err(err) = on_menu_event_inner(app, event) {
        error!("An error occurred whilst handling menu event {}", err);
    }
}

pub fn on_menu_event_inner(app_handle: &AppHandle, event: MenuEvent) -> Result<()> {
    let menu_item_id = event.id().0.as_str();
    let settings_manager = app_handle.state::<SettingsManager>();

    match TrayCommand::from_str(menu_item_id)? {
        TrayCommand::Quit => {
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;

            teardown(app_handle);
        }
        TrayCommand::Hide => {
            if let Some(meter) = app_handle.get_meter_window() {
                meter.hide()?;
            }

            if let Some(mini) = app_handle.get_mini_window() {
                mini.hide()?;
            }
        }
        TrayCommand::ShowMeter => {
            let settings_manager = app_handle.state::<SettingsManager>();
            let settings = settings_manager.read()?.unwrap_or_default();
            let window = app_handle.get_window(settings.general.mini);
            window.restore_and_focus();
        }
        TrayCommand::Reset => {
            let settings = settings_manager.read()?.unwrap_or_default();

            if settings.general.mini {
                if let Some(mini) = app_handle.get_mini_window() {
                    mini.set_size(DEFAULT_MINI_METER_WINDOW_SIZE)?;
                    mini.set_position(WINDOW_POSITION)?;
                    mini.restore_and_focus();
                }

                return Ok(());
            }

            if let Some(meter) = app_handle.get_meter_window() {
                meter.set_size(DEFAULT_METER_WINDOW_SIZE)?;
                meter.set_position(WINDOW_POSITION)?;
                meter.restore_and_focus();
            }
        }
        TrayCommand::ShowLogs => {
            if let Some(logs) = app_handle.get_logs_window() {
                logs.show().unwrap();
                logs.unminimize().unwrap();
            }
        }
        TrayCommand::StartLoa => {
            let shell_manager = app_handle.state::<ShellManager>();
            shell_manager.start_loa_process();
        }
    }

    Ok(())
}

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let label = window.label();
    on_window_event_inner(label, window, event)
        .expect("An error occurred whilst handling window event");
}

pub fn on_window_event_inner(label: &str, window: &Window, event: &WindowEvent) -> Result<()> {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            if label == LOGS_WINDOW_LABEL {
                window.hide()?;

                return Ok(());
            }

            let app_handle = window.app_handle();
            let meter_window = app_handle.get_meter_window().unwrap();
            let logs_window = app_handle.get_logs_window().unwrap();

            if logs_window.is_minimized()? {
                logs_window.unminimize()?;
            }

            if meter_window.is_minimized()? {
                meter_window.unminimize()?;
            }

            teardown(app_handle);

            Ok(())
        }
        WindowEvent::Focused(focused) => {
            if *focused {
                return Ok(());
            }

            let app_handle = window.app_handle();
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;

            Ok(())
        }
        _ => Ok(()),
    }
}

pub fn teardown(app_handle: &AppHandle) {

    let background = app_handle.state::<BackgroundWorker>();
    let shell_manager = app_handle.state::<ShellManager>();

    block_on_local(async {
        if let Err(err) = background.stop().await {
            warn!("Could not stop background worker: {}", err);
        }

        shell_manager.unload_driver().await;
    });

    log::logger().flush();
    app_handle.exit(0);
}