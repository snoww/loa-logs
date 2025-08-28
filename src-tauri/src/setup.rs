use std::{error::Error, sync::{atomic::{AtomicBool, Ordering}, Arc}};

use log::*;
use tauri::{App, AppHandle, Manager};
use tauri_plugin_window_state::WindowExt;
use crate::{background::BackgroundWorker, context::AppContext, settings::SettingsManager, shell::ShellManager};

use crate::{constants::*};

pub fn setup(app: &mut App) -> Result<(), Box<dyn Error>> {

    // #[cfg(debug_assertions)]
    // {
    //     meter_window.open_devtools();
    // }

    let app_handle = app.handle();
    let context = app.state::<AppContext>();
    let shell_manger = ShellManager::new(app_handle.shell_scope(), context.inner().clone());
    let settings_manager = app.state::<SettingsManager>();
    let version = app_handle.package_info().version.to_string();

    info!("starting app v{}", app.package_info().version);

    let update_checked: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    let checked_clone = update_checked.clone();
    let handle = app.handle();
    check_updates(handle.clone(), checked_clone);

    let settings = settings_manager.read()?;

    let meter_window = app.get_window(METER_WINDOW_LABEL).unwrap();
    meter_window
        .restore_state(WINDOW_STATE_FLAGS)
        .expect("failed to restore window state");

    let mini_window = app.get_window(METER_MINI_WINDOW_LABEL).unwrap();
    meter_window
        .restore_state(WINDOW_STATE_FLAGS)
        .expect("failed to restore window state");

    let logs_window = app.get_window(LOGS_WINDOW_LABEL).unwrap();
    logs_window
        .restore_state(WINDOW_STATE_FLAGS)
        .expect("failed to restore window state");

    let mut port = 6040;

    info!("settings loaded");

    if settings.general.mini {
        mini_window.show().unwrap();
    } else if !settings.general.hide_meter_on_start && !settings.general.mini {
        meter_window.show().unwrap();
    }

    if !settings.general.hide_logs_on_start {
        logs_window.show().unwrap();
    }

    if !settings.general.always_on_top {
        meter_window.set_always_on_top(false).unwrap();
        mini_window.set_always_on_top(false).unwrap();
    } else {
        meter_window.set_always_on_top(true).unwrap();
        mini_window.set_always_on_top(true).unwrap();
    }

    if settings.general.auto_iface && settings.general.port > 0 {
        port = settings.general.port;
    }

    if settings.general.start_loa_on_start {
        info!("auto launch game enabled");
        shell_manger.start_loa_process();
    }

    shell_manger.remove_driver();

    // only start listening if we have live meter
    #[cfg(feature = "meter-core")]
    {
        use crate::background::BackgroundWorkerArgs;

        let mut background = BackgroundWorker::new();

        let args = BackgroundWorkerArgs {
            update_checked,
            app: app_handle.clone(),
            port,
            settings,
            region_file_path: context.region_file_path.clone(),
            version
        };

        background.start(args);
        app_handle.manage(background);
    }

    Ok(())
}

fn check_updates(handle: AppHandle, checked_clone: Arc<AtomicBool>) {
     tauri::async_runtime::spawn(async move {
        match tauri::updater::builder(handle.clone()).check().await {
            Ok(update) => {
                if update.is_update_available() {
                    let manager = handle.state::<ShellManager>();

                    #[cfg(not(debug_assertions))]
                    {
                        info!(
                            "update available, downloading update: v{}",
                            update.latest_version()
                        );

                        manager.unload_driver();
                        manager.remove_driver();

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
                    checked_clone.store(true, Ordering::Relaxed);
                }
            }
            Err(e) => {
                warn!("failed to get update: {}", e);
                checked_clone.store(true, Ordering::Relaxed);
            }
        }
    });
}