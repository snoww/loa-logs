#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
#[cfg(feature = "meter-core")]
mod live;
mod misc;
mod context;
mod constants;
mod data;
mod models;
mod settings;
mod local;
mod ui;
mod shell;
mod utils;
mod setup;
mod background;
mod database;
mod handlers;

use anyhow::Result;
use crate::constants::*;
use crate::context::AppContext;
use crate::data::AssetPreloader;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::misc::load_windivert;
use crate::settings::SettingsManager;
use crate::setup::setup;
use crate::ui::on_window_event;

#[tokio::main]
async fn main() -> Result<()> {
    let _ = app::logger::init()?;
    let tauri_context = tauri::generate_context!();
    let package_info = tauri_context.package_info();
    let context = AppContext::new(package_info.version.to_string())?;
    let settings_manager = SettingsManager::new(context.settings_path.clone()).expect("could not create settings");
    load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    // load meter-data
    AssetPreloader::new()?;
    let database = Database::new(
        context.database_path.clone(),
        &context.version).expect("error setting up database: {}");
    let repository = database.create_repository();

    tauri::Builder::default()
        .manage(context)
        .manage(database)
        .manage(repository)
        .manage(settings_manager)
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_process::init())
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