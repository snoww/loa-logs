#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
use std::{time::{Duration, Instant}, path::PathBuf};

use parser::models::*;

use rusqlite::Connection;
use tauri::{Manager, api::process::{Command, CommandEvent}, LogicalSize, Size, SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem};
use window_vibrancy::apply_blur;

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let hide = CustomMenuItem::new("hide".to_string(), "Hide");
    let tray_menu = SystemTrayMenu::new()
        .add_item(quit)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(hide);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            window.set_always_on_top(true)
                .expect("failed to set windows always on top");
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
              window.open_devtools();
            }

            window.set_size(Size::Logical(LogicalSize { width: 500.0, height: 350.0 })).unwrap();

            #[cfg(target_os = "windows")]
            apply_blur(&window, Some((10, 10, 10, 50))).expect("Unsupported platform! 'apply_blur' is only supported on Windows");
            let mut resource_path = app.path_resolver().resource_dir().expect("could not get resource dir");
            match setup_db(&mut resource_path) {
                Ok(_) => (),
                Err(e) => {
                    println!("error setting up database: {}", e);
                    window.emit("error", Some(e))
                        .expect("failed to emit encounter-update");
                }
            }

            tauri::async_runtime::spawn(async move {
                let (mut rx, _child) = Command::new_sidecar("meter-core")
                    .expect("failed to start `meter-core` ")
                    .spawn()
                    .expect("Failed to spawn sidecar");
                // let (mut rx, _child) = Command::new_sidecar("loa-fake-log")
                //     .expect("failed to start `meter-core` ")
                //     .spawn()
                //     .expect("Failed to spawn sidecar");

                let mut encounter: Encounter = Default::default();
                let mut none: Option<Vec<Encounter>> = None;
                let mut last_time = Instant::now();
                let duration = Duration::from_millis(100);
                let mut reset = false;
                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        parser::parse_line(Some(&window), &mut none, &mut reset, &mut encounter, line);
                        let elapsed = last_time.elapsed();
                        if elapsed >= duration {
                        // if true {
                            let mut clone = encounter.clone();
                            let window = window.clone();
                            tauri::async_runtime::spawn(async move {
                                if !clone.current_boss_name.is_empty() {
                                    clone.current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                                    if clone.current_boss.is_none() {
                                        clone.current_boss_name = String::new();
                                    }
                                }
                                clone.entities.retain(|_, v| v.entity_type == EntityType::PLAYER && v.skill_stats.hits > 0);
                                if clone.entities.len() > 0 {
                                    window.emit("encounter-update", Some(clone))
                                        .expect("failed to emit encounter-update");
                                }
                            });
                        }
                        last_time = Instant::now();
                    }
                }
            });

            Ok(())
        })
        // .invoke_handler(tauri::generate_handler![init_process])
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .system_tray(system_tray)
        .run(tauri::generate_context!())
        .expect("error while running application");
}


fn setup_db(resource_path: &mut PathBuf) -> Result<(), String> {
    resource_path.push("encounters.db");
    let conn = match Connection::open(resource_path) {
        Ok(conn) => conn,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    match conn.execute("
        CREATE TABLE IF NOT EXISTS entity (
            name TEXT,
            encounter_id INTEGER NOT NULL,
            npc_id INTEGER,
            entity_type TEXT,
            class_id INTEGER,
            class TEXT,
            gear_score REAL,
            current_hp INTEGER,
            max_hp INTEGER,
            is_dead INTEGER,
            skills TEXT,
            damage_stats TEXT,
            skill_stats TEXT,
            PRIMARY KEY (name, encounter_id),
            FOREIGN KEY (encounter_id) REFERENCES encounter (id)
        );", ()) {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    match conn.execute("
        CREATE TABLE IF NOT EXISTS encounter (
            id INTEGER PRIMARY KEY,
            last_combat_packet INTEGER,
            fight_start INTEGER,
            local_player TEXT,
            current_boss TEXT,
            duration INTEGER,
            total_damage_dealt INTEGER,
            top_damage_dealt INTEGER,
            total_damage_taken INTEGER,
            top_damage_taken INTEGER,
            dps INTEGER,
            dps_intervals TEXT,
            buffs TEXT,
            debuffs TEXT
        );", ()) {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    Ok(())
}
