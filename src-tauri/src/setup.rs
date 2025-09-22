use std::{error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use log::*;
use tauri::{App, AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::{background::{BackgroundWorker, BackgroundWorkerArgs}, constants::DEFAULT_PORT, context::AppContext, settings::*, shell::ShellManager, ui::{setup_tray, AppHandleExtensions, WindowExtensions}};

pub fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {

    let app_handle = app.handle();

    let context = app.state::<AppContext>();
    let shell_manger = ShellManager::new(app_handle.clone(), context.inner().clone());
    let settings_manager = app.state::<SettingsManager>();
    
    info!("starting app v{}", context.version);
    setup_tray(app_handle)?;
    let update_checked: Arc<AtomicBool> = check_updates(app_handle);

    let settings = settings_manager.read().expect("Could not read settings");

    let port = initialize_windows_and_settings(
        &app_handle,
        settings.as_ref(),
        &shell_manger
    );

    app_handle.manage(shell_manger);

    let mut background = BackgroundWorker::new();

    let args = BackgroundWorkerArgs {
        app_handle: app_handle.clone(),
        update_checked,
        port,
        settings,
        region_file_path: context.region_file_path.clone(),
        version: context.version.clone()
    };

    background.start(args)?;
    app_handle.manage(background);

    // #[cfg(debug_assertions)]
    // {
    //     _logs_window.open_devtools();
    // }

    Ok(())
}

fn check_updates(app_handle: &AppHandle) -> Arc<AtomicBool> {
    let update_checked = Arc::new(AtomicBool::new(false));

    {
        let update_checked = update_checked.clone();
        let app_handle = app_handle.clone();

        let check_update = async move {
            match app_handle.updater().unwrap().check().await {
                #[cfg(not(debug_assertions))]
                Ok(Some(update)) => {
                    info!("update available, downloading update: v{}", update.version);

                    let shell_manager = app_handle.state::<ShellManager>();
                    shell_manager.unload_driver().await;
                    shell_manager.remove_driver().await;

                    if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                        error!("failed to download update: {}", e);
                    }
                }
                Err(e) => {
                    warn!("failed to get update: {e}");
                    update_checked.store(true, Ordering::Relaxed);
                }
                _ => {
                    info!("no update available");
                    update_checked.store(true, Ordering::Relaxed);
                }
            }
        };

        tauri::async_runtime::spawn(check_update);
    }

    update_checked
}

fn initialize_windows_and_settings(
    app_handle: &AppHandle,
    settings: Option<&Settings>,
    shell_manger: &ShellManager) -> u16 {

    let mut port = DEFAULT_PORT;
    let meter_window = app_handle.get_meter_window().unwrap();
    let mini_window = app_handle.get_mini_window().unwrap();
    let logs_window = app_handle.get_logs_window().unwrap();
    
    if let Some(settings) = settings.clone() {

        info!("settings loaded");
        if settings.general.mini {
            mini_window.restore_default_state();
            mini_window.show().unwrap();
        } else if !settings.general.hide_meter_on_start && !settings.general.mini {
            meter_window.restore_default_state();
            meter_window.show().unwrap();
        }
        
        if !settings.general.hide_logs_on_start {
            logs_window.restore_default_state();
            logs_window.show().unwrap();
        }

        if settings.general.always_on_top {
            meter_window.set_always_on_top(true).unwrap();
            mini_window.set_always_on_top(true).unwrap();
        } else {
            meter_window.set_always_on_top(false).unwrap();
            mini_window.set_always_on_top(false).unwrap();    
        }

        if settings.general.auto_iface && settings.general.port > 0 {
            port = settings.general.port;
        }

        if settings.general.start_loa_on_start {
            info!("auto launch game enabled");
            shell_manger.start_loa_process();
        }
    }
    else {
        meter_window.show().unwrap();
        logs_window.show().unwrap();
    }

    port
}