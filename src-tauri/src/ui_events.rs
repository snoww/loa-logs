use log::{error, info};
use tauri::{async_runtime, AppHandle, Window, WindowEvent};
use tauri::Manager;
use tauri_plugin_window_state::AppHandleExt;
use tauri::{menu::*, tray::{TrayIcon, TrayIconBuilder, TrayIconEvent}, App, Wry};
use crate::constants::*;
use crate::settings::SettingsManager;
use crate::shell::ShellManager;
use crate::extensions::{AppHandleExtensions, WindowExtensions};
use anyhow::Result;

pub fn create_system_tray_menu(app: &App) -> Result<()> {
        let builder = TrayIconBuilder::new();

    let items: Vec<Box<dyn IsMenuItem<Wry>>> = vec![
        Box::new(MenuItemBuilder::new("Show Logs").id("show-logs").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Show Meter").id("show-meter").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Hide Meter").id("hide").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Start Lost Ark").id("start-loa").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Reset Window").id("reset").build(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(PredefinedMenuItem::separator(app)?) as Box<dyn IsMenuItem<Wry>>,
        Box::new(MenuItemBuilder::new("Quit").id("quit").build(app)?) as Box<dyn IsMenuItem<Wry>>
    ];

    let item_refs: Vec<&dyn IsMenuItem<Wry>> = items.iter()
        .map(|b| b.as_ref())
        .collect();

    let menu = MenuBuilder::new(app)
        .items(&item_refs)
        .build()?;


    builder
        .icon(app.default_window_icon().unwrap().clone())
        .menu(&menu)
        .on_menu_event(on_menu_event)
        // .on_tray_icon_event(on_tray_icon_event)
        .build(app)?;

    Ok(())
}

pub fn on_menu_event(app: &AppHandle, event: MenuEvent) {
    if let Err(err) = on_menu_event_inner(app, event) {
        error!("An error occurred whilst handling menu event {}", err);
    }
}

pub fn on_menu_event_inner(app_handle: &AppHandle, event: MenuEvent) -> Result<()> {
    let menu_item_id = event.id().0.as_str();
    let settings_manager = app_handle.state::<SettingsManager>();

    match menu_item_id {
        "quit" => {
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;
            
            let shell_manager = app_handle.state::<ShellManager>();
            async_runtime::block_on(async {
                shell_manager.unload_driver().await;
            });
        
            app_handle.exit(0);
        }
        "hide" => {
            if let Some(meter) = app_handle.get_meter_window() {
                meter.hide()?;
            }

            if let Some(mini) = app_handle.get_mini_window() {
                mini.hide()?;
            }
        }
        "show-meter" => {
            let settings_manager = app_handle.state::<SettingsManager>();
            let settings = settings_manager.read()?;
            let window = app_handle.get_window(settings.general.mini);
            window.restore_and_focus();
        }
        "reset" => {
            let settings = settings_manager.read()?;

            if settings.general.mini {
                if let Some(mini) = app_handle.get_mini_window() {
                    mini.set_size(DEFAULT_MINI_METER_WINDOW_SIZE)?;
                    mini.set_position(WINDOW_POSITION)?;
                    mini.restore_and_focus();
                }

                return Ok(())
            }

            if let Some(meter) = app_handle.get_meter_window() {
                meter
                    .set_size(DEFAULT_METER_WINDOW_SIZE)?;
                meter.set_position(WINDOW_POSITION)?;
                meter.restore_and_focus();
            }
        }
        "show-logs" => {
            if let Some(logs) = app_handle.get_logs_window() {
                logs.show().unwrap();
                logs.unminimize().unwrap();
            }
        }
        "start-loa" => {
            let shell_manager = app_handle.state::<ShellManager>();
            shell_manager.start_loa_process();
        }
        _ => {}
    }

    Ok(())
}

// pub fn on_tray_icon_event(icon: &TrayIcon, event: TrayIconEvent) {
//     on_tray_icon_event_inner(icon, event)
//         .expect("An error occurred whilst handling tray icon event");
// }

// pub fn on_tray_icon_event_inner(icon: &TrayIcon, event: TrayIconEvent) -> Result<()> {

//     info!("{:?} {:?}", event, event.id());
//     event.
    
//     match event {
//         TrayIconEvent::DoubleClick { .. } => {
//             let app_handle = icon.app_handle();
//             let settings_manager = app_handle.state::<SettingsManager>();
//             let settings = settings_manager.read()?;
//             let window = app_handle.get_window(settings.general.mini);
//             window.restore_and_focus();

//             Ok(())
//         },
//         _ => Ok(())
//     }
// }

pub fn on_window_event(window: &Window, event: &WindowEvent) {
    let label = window.label();
    on_window_event_inner(label, window, event).expect("An error occurred whilst handling window event");
}

pub fn on_window_event_inner(label: &str, window: &Window, event: &WindowEvent) -> Result<()> {
    match event {
        WindowEvent::CloseRequested { api, .. } => {
            api.prevent_close();

            if label == LOGS_WINDOW_LABEL {
                window.hide()?;

                return Ok(())
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

            let shell_manager = app_handle.state::<ShellManager>();
               async_runtime::block_on(async {
                shell_manager.unload_driver().await;
            });

            app_handle.exit(0);

            Ok(())
        },
        WindowEvent::Focused(focused) => {
            if *focused {
                return Ok(())
            }

            let app_handle = window.app_handle();
            app_handle.save_window_state(WINDOW_STATE_FLAGS)?;

            Ok(())
        },
        _ => Ok(()),
    }
}
