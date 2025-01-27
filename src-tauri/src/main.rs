#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod parser;
mod commands;
mod constants;
mod system_tray;
mod database;
mod settings;
mod setup;
mod window_event;
mod utils;

use anyhow::Result;
use commands::generate_handlers;
use constants::{LOGS_WINDOW_LABEL, METER_WINDOW_LABEL};
use parser::models::*;

use system_tray::{create_system_tray, on_system_tray_event};
use utils::set_panic_hook;
use window_event::on_window_event;

#[tokio::main]
async fn main() -> Result<()> {
    app::init();

    set_panic_hook();

    let system_tray = create_system_tray();
    
    tauri::Builder::default()
        .setup(setup::setup)
        .plugin(
            tauri_plugin_window_state::Builder::new()
                .skip_initial_state(METER_WINDOW_LABEL)
                .skip_initial_state(LOGS_WINDOW_LABEL)
                .build(),
        )
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .on_window_event(on_window_event)
        .system_tray(system_tray)
        .on_system_tray_event(on_system_tray_event)
        .invoke_handler(generate_handlers())
        .run(tauri::generate_context!())
        .expect("error while running application");

    Ok(())
}
