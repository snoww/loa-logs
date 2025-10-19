#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod background;
mod constants;
mod context;
mod data;
mod database;
mod handlers;
#[cfg(feature = "meter-core")]
mod live;
mod local;
mod misc;
mod models;
mod settings;
mod setup;
mod shell;
mod ui;
mod utils;

use crate::constants::*;
use crate::context::AppContext;
use crate::data::AssetPreloader;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::misc::load_windivert;
use crate::settings::SettingsManager;
use crate::setup::setup;
use crate::ui::on_window_event;
use crate::app::autostart::AutoLaunchManager;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = app::logger::init()?;
    app::panic::set_hook_with_logger();

    let tauri_context = tauri::generate_context!();
    let package_info = tauri_context.package_info();
    let context =
        AppContext::new(package_info.version.to_string()).expect("could not create context");
    let settings_manager =
        SettingsManager::new(context.settings_path.clone()).expect("could not create settings");
    load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    AssetPreloader::new(&context.current_dir).expect("could not load meter-data");
    let database = Database::new(context.database_path.clone(), &context.version)
        .expect("error setting up database: {}");
    let repository = database.create_repository();
    let auto_launch_manager = AutoLaunchManager::new(
        &package_info.name,
        &context.app_path,
    );
        
    tauri::Builder::default()
        .manage(auto_launch_manager)
        .manage(context)
        .manage(database)
        .manage(repository)
        .manage(settings_manager)
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .setup(setup)
        .on_window_event(on_window_event)
        .invoke_handler(generate_handlers())
        .run(tauri_context)
        .expect("error while running application");

    Ok(())
}
