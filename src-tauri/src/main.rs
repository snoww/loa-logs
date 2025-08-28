#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

#[cfg(feature = "meter-core")]
mod live;
mod parser;
mod setup;
mod constants;
mod database;
mod ui_events;
mod handlers;
mod settings;
mod background;
mod shell;
mod context;
mod logger;
mod autostart;
mod extensions;
mod windivert;

use anyhow::Result;

use crate::autostart::AutoLaunchManager;
use crate::constants::*;
use crate::context::AppContext;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::live::local::LocalPlayerRepository;
use crate::logger::{setup_logger, setup_panic_hook};
use crate::parser::data::AssetPreloader;
use crate::settings::SettingsManager;
use crate::ui_events::*;
use crate::windivert::load_windivert;

#[tokio::main]
async fn main() -> Result<()> {
    let tauri_context = tauri::generate_context!();
    let context = AppContext::new()?;
    let auto_launch_manager = AutoLaunchManager::new(
        &tauri_context.package_info().name,
        &context.app_path.display().to_string());
    let settings_manager = SettingsManager::new(context.settings_path.clone());
    let loader = AssetPreloader::new();
    let database = Database::new(context.database_path.clone()).expect("error setting up database: {}");
    let repository = database.create_repository();
    let local_player = LocalPlayerRepository::new(context.local_player_path.clone())?;

    load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    setup_logger(&context.current_dir);
    setup_panic_hook();

    tauri::Builder::default()
        .manage(context)
        .manage(loader)
        .manage(repository)
        .manage(settings_manager)
        .manage(auto_launch_manager)
        .manage(local_player)
        .setup(setup::setup)
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .on_window_event(on_window_event)
        .system_tray(create_system_tray_menu())
        .on_system_tray_event(on_system_tray_event)
        .invoke_handler(generate_handlers())
        .run(tauri_context)
        .expect("error while running application");

    Ok(())
}
