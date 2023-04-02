#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
use std::time::{Duration, Instant};

use parser::models::*;

use tauri::{Manager, Window, api::process::{Command, CommandEvent}, LogicalSize, Size};

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let window = app.get_window("main").unwrap();
            window.set_always_on_top(true)
                .expect("failed to set windows always on top");
            #[cfg(debug_assertions)] // only include this code on debug builds
            {
              window.open_devtools();
            }
            window.set_size(Size::Logical(LogicalSize { width: 500.0, height: 300.0 })).unwrap();


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
                                if clone.current_boss.is_some() || clone.entities.len() > 0 {
                                    window.emit("encounter-update", Some(clone))
                                        .expect("failed to emit encounter-update");
                                }
                                last_time = Instant::now();
                            });
                        }
                    }
                }
            });

            Ok(())
        })
        // .invoke_handler(tauri::generate_handler![init_process])
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
