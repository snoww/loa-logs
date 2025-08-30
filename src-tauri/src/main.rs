#![allow(unused_imports)]
#![allow(unused_variables)]

#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod live;
mod abstractions;
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

use anyhow::Result;
use log::LevelFilter;

use crate::autostart::AutoLaunchManager;
use crate::constants::*;
use crate::context::AppContext;
use crate::database::Database;
use crate::handlers::generate_handlers;
use crate::live::local::LocalPlayerRepository;
use crate::logger::setup_panic_hook;
use crate::live::data::AssetPreloader;
use crate::settings::SettingsManager;
use crate::ui_events::*;

#[tokio::main]
async fn main() -> Result<()> {
    let tauri_context = tauri::generate_context!();
    let context = AppContext::new()?;
    let auto_launch_manager = AutoLaunchManager::new(
        &tauri_context.package_info().name,
        &context.app_path.display().to_string());
    let settings_manager = SettingsManager::new(context.settings_path.clone())?;
    let loader = AssetPreloader::new();
    let database = Database::new(context.database_path.clone()).expect("error setting up database: {}");
    let repository = database.create_repository();
    let local_player = LocalPlayerRepository::new(context.local_player_path.clone())?;

    #[cfg(feature = "meter-core")]
    {
        use crate::abstractions::load_windivert;
        load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    }
    setup_panic_hook();

    let log_builder = tauri_plugin_log::Builder::new()
        .level(log::LevelFilter::Info)
        .level_for("tao::platform_impl::platform::event_loop::runner", LevelFilter::Error)
        .max_file_size(5_000_000)
        .rotation_strategy(tauri_plugin_log::RotationStrategy::KeepAll)
        .target(tauri_plugin_log::Target::new(
            tauri_plugin_log::TargetKind::LogDir {
                file_name: Some("loa_logs".to_string()),
            },
        ));

    tauri::Builder::default()
        .manage(context)
        .manage(loader)
        .manage(repository)
        .manage(settings_manager)
        .manage(auto_launch_manager)
        .manage(local_player)
        .plugin(log_builder.build())
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .with_state_flags(WINDOW_STATE_FLAGS)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|app, args, cwd| {}))
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .setup(setup::setup)
        .on_window_event(on_window_event)
        .invoke_handler(generate_handlers())
        .run(tauri_context)
        .expect("error while running application");

    Ok(())
}
