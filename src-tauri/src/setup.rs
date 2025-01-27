use std::error::Error;

use log::{error, info, warn};
use tauri::{App, Manager};
use tauri_plugin_window_state::WindowExt;
use tokio::task;

use crate::{constants::{LOGS_WINDOW_LABEL, METER_WINDOW_LABEL, WINDOW_STATE_FLAGS}, database::setup_db, parser, settings::read_settings, utils::{remove_driver, start_loa_process}};


pub fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {
    info!("starting app v{}", app.package_info().version.to_string());

    let resource_path = app
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");

    match setup_db(&resource_path) {
        Ok(_) => (),
        Err(e) => {
            warn!("error setting up database: {}", e);
        }
    }

    let handle = app.handle();
    tauri::async_runtime::spawn(async move {
        match tauri::updater::builder(handle).check().await {
            Ok(update) => {
                if update.is_update_available() {
                    #[cfg(not(debug_assertions))]
                    {
                        info!(
                            "update available, downloading update: v{}",
                            update.latest_version()
                        );

                        unload_driver();
                        remove_driver();

                        update
                            .download_and_install()
                            .await
                            .map_err(|e| {
                                error!("failed to download update: {}", e);
                            })
                            .ok();
                    }
                } else {
                    info!("no update available");
                }
            }
            Err(e) => {
                warn!("failed to get update: {}", e);
            }
        }
    });

    let settings = read_settings(&resource_path).ok();

    let meter_window = app.get_window(METER_WINDOW_LABEL).unwrap();
    meter_window
        .restore_state(WINDOW_STATE_FLAGS)
        .expect("failed to restore window state");
    // #[cfg(debug_assertions)]
    // {
    //     meter_window.open_devtools();
    // }

    let logs_window = app.get_window(LOGS_WINDOW_LABEL).unwrap();
    logs_window
        .restore_state(WINDOW_STATE_FLAGS)
        .expect("failed to restore window state");

    let mut port = 6040;

    if let Some(settings) = settings.clone() {
        info!("settings loaded");
        if !settings.general.hide_meter_on_start {
            meter_window.show().unwrap();
        }
        if !settings.general.hide_logs_on_start {
            logs_window.show().unwrap();
        }
        if !settings.general.always_on_top {
            meter_window.set_always_on_top(false).unwrap();
        }

        if settings.general.auto_iface && settings.general.port > 0 {
            port = settings.general.port;
        }

        if settings.general.start_loa_on_start {
            info!("auto launch game enabled");
            start_loa_process();
        }
    } else {
        meter_window.show().unwrap();
        logs_window.show().unwrap();
    }

    info!("listening on port: {}", port);
    remove_driver();
    task::spawn_blocking(move || {
        parser::start(meter_window, port, settings).map_err(|e| {
            error!("unexpected error occurred in parser: {}", e);
        })
    });

    // #[cfg(debug_assertions)]
    // {
    //     _logs_window.open_devtools();
    // }

    Ok(())
}