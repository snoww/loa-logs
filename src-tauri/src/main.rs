#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
use std::{time::{Duration, Instant}, path::PathBuf};

use hashbrown::HashMap;
use parser::{models::*, Parser};

use rusqlite::{Connection, params};
use tauri::{Manager, api::process::{Command, CommandEvent }, SystemTray, CustomMenuItem, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder, SystemTrayEvent};
use tauri_plugin_window_state::{AppHandleExt, WindowExt, StateFlags};
use window_vibrancy::apply_blur;

fn main() {
    let quit = CustomMenuItem::new("quit".to_string(), "Quit");
    let show_logs = CustomMenuItem::new("show-logs".to_string(), "Show Logs");
    let show_meter = CustomMenuItem::new("show-meter".to_string(), "Show Meter");
    let hide_meter = CustomMenuItem::new("hide".to_string(), "Hide Meter");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show_logs)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show_meter)
        .add_item(hide_meter)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .setup(|app| {
            let meter_window = app.get_window("main").unwrap();
            meter_window.set_always_on_top(true)
                .expect("failed to set windows always on top");
            meter_window.restore_state(StateFlags::all())
                .expect("failed to restore window state");
            #[cfg(debug_assertions)]
            {
                meter_window.open_devtools();
            }
            
            #[cfg(target_os = "windows")]
            {
                apply_blur(&meter_window, Some((10, 10, 10, 50))).expect("Unsupported platform! 'apply_blur' is only supported on Windows");
            }
            let mut resource_path = app.path_resolver().resource_dir().expect("could not get resource dir");
            match setup_db(&mut resource_path) {
                Ok(_) => (),
                Err(e) => {
                    println!("error setting up database: {}", e);
                }
            }

            tauri::async_runtime::spawn(async move {
                let (mut rx, _child) = Command::new_sidecar("meter-core")
                    .expect("failed to start `meter-core` ")
                    .spawn()
                    .expect("Failed to spawn sidecar");
                let mut parser = Parser::new(&meter_window);
                let mut last_time = Instant::now();
                let duration = Duration::from_millis(100);
                while let Some(event) = rx.recv().await {
                    if let CommandEvent::Stdout(line) = event {
                        parser.parse_line(line);
                        // if raid end, we send regardless of window
                        if last_time.elapsed() >= duration || parser.raid_end {
                            let mut clone = parser.encounter.clone();
                            let window = meter_window.clone();
                            tauri::async_runtime::spawn(async move {
                                if !clone.current_boss_name.is_empty() {
                                    clone.current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                                    if clone.current_boss.is_none() {
                                        clone.current_boss_name = String::new();
                                    }
                                }
                                clone.entities.retain(|_, v| v.entity_type == EntityType::PLAYER && v.skill_stats.hits > 0 && v.max_hp > 0);
                                if !clone.entities.is_empty() {
                                    // don't need to send these to the live meter
                                    clone.entities.values_mut()
                                        .for_each(|e| {
                                            e.damage_stats.dps_average = Vec::new();
                                            e.damage_stats.dps_rolling_10s_avg = Vec::new();
                                            e.skills.values_mut()
                                                .for_each(|s| {
                                                    s.cast_log = Vec::new();
                                                });
                                        });
                                    window.emit("encounter-update", Some(clone))
                                        .expect("failed to emit encounter-update");
                                }
                            });
                        }
                        last_time = Instant::now();
                    }
                }
            });

            let logs_window = WindowBuilder::new(app, "logs", tauri::WindowUrl::App("/logs".into()))
                .title("LOA Logs")
                .min_inner_size(650.0, 300.0)
                .inner_size(800.0, 500.0)
                .build()
                .expect("failed to create log window");
            #[cfg(debug_assertions)]
            {
                logs_window.open_devtools();
            }

            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .on_window_event(|event| if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
            if event.window().label() == "logs" {
                event.window().hide().unwrap();
                api.prevent_close();
            }
            if event.window().label() == "main" {
                event.window().app_handle().save_window_state(StateFlags::all()).expect("failed to save window state");
                std::process::exit(0);
            }
        })
        .system_tray(system_tray)
        .on_system_tray_event(|app, event| match event {
            SystemTrayEvent::LeftClick {
                position: _,
                size: _,
                ..
            } => {
                if let Some(meter) = app.get_window("main") {
                    meter.show().unwrap();
                    meter.unminimize().unwrap();
                }
            }
            SystemTrayEvent::MenuItemClick { id, .. } => {
                match id.as_str() {
                    "quit" => {
                        app.save_window_state(StateFlags::all()).expect("failed to save window state");
                        std::process::exit(0);
                    }
                    "hide" => {
                        if let Some(meter) = app.get_window("main") {
                            meter.hide().unwrap();
                        }
                    }
                    "show-meter" => {
                        if let Some(meter) = app.get_window("main") {
                            meter.show().unwrap();
                        }
                    }
                    "show-logs" => {
                        if let Some(logs) = app.get_window("logs") {
                            logs.show().unwrap();
                            logs.unminimize().unwrap();
                        } else {
                            WindowBuilder::new(app, "logs", tauri::WindowUrl::App("/logs".into()))
                                .title("LOA Logs")
                                .min_inner_size(500.0, 300.0)
                                .build()
                                .expect("failed to create log window");
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![load_encounters_preview, load_encounter, open_most_recent_encounter, delete_encounter, toggle_meter_window, open_url])
        .run(tauri::generate_context!())
        .expect("error while running application");
}

fn get_db_connection(resource_path: &mut PathBuf) -> Result<Connection, String> {
    resource_path.push("encounters.db");
    let conn = match Connection::open(resource_path) {
        Ok(conn) => conn,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    Ok(conn)
}


fn setup_db(resource_path: &mut PathBuf) -> Result<(), String> {
    let conn = get_db_connection(resource_path)?;

    match conn.execute_batch("
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
            buffs TEXT,
            debuffs TEXT
        );
        CREATE INDEX IF NOT EXISTS encounter_fight_start_index
        ON encounter (fight_start desc);
        CREATE INDEX IF NOT EXISTS encounter_current_boss_index
        ON encounter (current_boss);
        ") {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    let mut stmt = conn.prepare("SELECT COUNT(*) FROM pragma_table_info('encounter') WHERE name='misc'").unwrap();
    let column_count: u32 = stmt.query_row([], |row| row.get(0)).unwrap();
    if column_count == 0 {
        conn.execute("ALTER TABLE encounter ADD COLUMN misc TEXT", []).expect("failed to add column");
    }

    match conn.execute_batch("
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
            last_update INTEGER,
            PRIMARY KEY (name, encounter_id),
            FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
        );
        CREATE INDEX IF NOT EXISTS entity_encounter_id_index
        ON entity (encounter_id desc);
        CREATE INDEX IF NOT EXISTS entity_name_index
        ON entity (name);
        CREATE INDEX IF NOT EXISTS entity_class_index
        ON entity (class);
        ") {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    Ok(())
}

#[tauri::command]
fn load_encounters_preview(window: tauri::Window, page: i32, page_size: i32, min_duration: i32, search: String) -> EncountersOverview {
    let mut path = window.app_handle().path_resolver().resource_dir().expect("could not get resource dir");
    let conn = get_db_connection(&mut path).expect("could not get db connection");

    let mut stmt = conn.prepare_cached("
    SELECT
        e.id,
        e.fight_start,
        e.current_boss,
        e.duration,
        (
            SELECT GROUP_CONCAT(ordered_classes.class_id, ',')
            FROM (
                SELECT en.class_id
                FROM entity en
                WHERE en.encounter_id = e.id
                ORDER BY json_extract(en.damage_stats, '$.dps') DESC
            ) AS ordered_classes
        ) AS classes
    FROM
        encounter e
    WHERE e.duration > ? AND current_boss LIKE '%' || ? || '%'
    ORDER BY
        e.fight_start DESC
    LIMIT ?
    OFFSET ?
    ")
    .unwrap();

    let offset = (page - 1) * page_size;
    let min_duration = min_duration * 1000;

    let encounter_iter = stmt.query_map([min_duration.to_string(), search.to_string(), page_size.to_string(), offset.to_string()], |row| {
        let classes = match row.get(4) {
            Ok(classes) => classes,
            Err(_) => "101".to_string()
        };

        Ok(EncounterPreview {
            id: row.get(0)?,
            fight_start: row.get(1)?,
            boss_name: row.get(2)?,
            duration: row.get(3)?,
            classes: classes.split(',').map(|s| s.parse::<i32>().unwrap()).collect()
        })
    }).expect("could not query encounters");

    let mut encounters: Vec<EncounterPreview> = Vec::new();
    for encounter in encounter_iter {
        encounters.push(encounter.unwrap());
    }

    let count: i32 = conn.query_row_and_then("
    SELECT COUNT(*) 
    FROM encounter 
    WHERE duration > ? AND current_boss LIKE '%' || ? || '%'
    ", [min_duration.to_string(), search], |row| {
        row.get(0)
    }).expect("could not get encounter count");

    EncountersOverview {
        encounters,
        total_encounters: count
    }
}

#[tauri::command]
fn load_encounter(window: tauri::Window, id: String) -> Encounter {
    let mut path = window.app_handle().path_resolver().resource_dir().expect("could not get resource dir");
    let conn = get_db_connection(&mut path).expect("could not get db connection");
    let mut encounter_stmt = conn.prepare_cached("
    SELECT last_combat_packet,
       fight_start,
       local_player,
       current_boss,
       duration,
       total_damage_dealt,
       top_damage_dealt,
       total_damage_taken,
       top_damage_taken,
       dps,
       buffs,
       debuffs,
       misc
    FROM encounter
    WHERE id = ?
    ;").unwrap();

    let mut encounter = match encounter_stmt.query_row(params![id], |row| {
        let buff_str = match row.get(10) {
            Ok(buff_str) => buff_str,
            Err(_) => "".to_string()
        };

        let buffs = match serde_json::from_str::<HashMap<i32, StatusEffect>>(buff_str.as_str()) {
            Ok(v) => v,
            Err(_) => HashMap::new()
        };

        let debuff_str = match row.get(11) {
            Ok(debuff_str) => debuff_str,
            Err(_) => "".to_string()
        };
        let debuffs = match serde_json::from_str::<HashMap<i32, StatusEffect>>(debuff_str.as_str()) {
            Ok(v) => v,
            Err(_) => HashMap::new()
        };

        let misc_str = match row.get(12) {
            Ok(misc_str) => misc_str,
            Err(_) => "".to_string()
        };
        let misc = match serde_json::from_str::<EncounterMisc>(misc_str.as_str()) {
            Ok(v) => Some(v),
            Err(_) => None
        };

        Ok(Encounter {
            last_combat_packet: row.get(0)?,
            fight_start: row.get(1)?,
            local_player: row.get(2)?,
            current_boss_name: row.get(3)?,
            duration: row.get(4)?,
            encounter_damage_stats: EncounterDamageStats {
                total_damage_dealt: row.get(5)?,
                top_damage_dealt: row.get(6)?,
                total_damage_taken: row.get(7)?,
                top_damage_taken: row.get(8)?,
                dps: row.get(9)?,
                buffs,
                debuffs,
                misc,
                ..Default::default()
            },
            ..Default::default()
        })
    }) {
        Ok(v) => v,
        Err(_) => return Encounter::default()
    };

    let mut entity_stmt = conn.prepare_cached("
    SELECT name,
        class_id,
        class,
        gear_score,
        current_hp,
        max_hp,
        is_dead,
        skills,
        damage_stats,
        skill_stats,
        last_update
    FROM entity
    WHERE encounter_id = ?;
    ").unwrap();

    let entity_iter = entity_stmt.query_map(params![id], |row| {
        let skill_str = match row.get(7) {
            Ok(skill_str) => skill_str,
            Err(_) => "".to_string()
        };
        let skills = match serde_json::from_str::<HashMap<i32, Skill>>(skill_str.as_str()) {
            Ok(v) => v,
            Err(_) => HashMap::new()
        };

        let damage_stats_str = match row.get(8) {
            Ok(damage_stats_str) => damage_stats_str,
            Err(_) => "".to_string()
        };

        let damage_stats = match serde_json::from_str::<DamageStats>(damage_stats_str.as_str()) {
            Ok(v) => v,
            Err(_) => DamageStats::default()
        };

        let skill_stats_str = match row.get(9) {
            Ok(skill_stats_str) => skill_stats_str,
            Err(_) => "".to_string()
        };
        let skill_stats = match serde_json::from_str::<SkillStats>(skill_stats_str.as_str()) {
            Ok(v) => v,
            Err(_) => SkillStats::default()
        };

        Ok(Entity {
            name: row.get(0)?,
            class_id: row.get(1)?,
            class: row.get(2)?,
            gear_score: row.get(3)?,
            current_hp: row.get(4)?,
            max_hp: row.get(5)?,
            is_dead: row.get(6)?,
            skills,
            damage_stats,
            skill_stats,
            last_update: row.get(10)?,
            ..Default::default()
        })
    }).unwrap();

    let mut entities: HashMap<String, Entity> = HashMap::new();
    for entity in entity_iter {
        let entity = entity.unwrap();
        entities.insert(entity.name.to_string(), entity);
    }

    encounter.entities = entities;

    encounter
}

#[tauri::command]
fn open_most_recent_encounter(window: tauri::Window) {
    let mut path = window.app_handle().path_resolver().resource_dir().expect("could not get resource dir");
    let conn = get_db_connection(&mut path).expect("could not get db connection");
    let mut stmt = conn.prepare_cached("
    SELECT id
    FROM encounter
    ORDER BY fight_start DESC
    LIMIT 1;
    ").unwrap();

    let id_result: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| {
        row.get(0)
    });

    if let Some(logs) = window.app_handle().get_window("logs") {
        match id_result {
            Ok(id) => {
                logs.emit("show-latest-encounter", id.to_string()).unwrap();
            },
            Err(_) => {
                logs.emit("redirect-url", "logs").unwrap();
            },
        }
    }
}

#[tauri::command]
fn delete_encounter(window: tauri::Window, id: String) {
    let mut path = window.app_handle().path_resolver().resource_dir().expect("could not get resource dir");
    let conn = get_db_connection(&mut path).expect("could not get db connection");
    conn.execute("PRAGMA foreign_keys = ON;", params![]).unwrap();
    let mut stmt = conn.prepare_cached("
        DELETE FROM encounter
        WHERE id = ?;
    ").unwrap();

    stmt.execute(params![id]).unwrap();
}

#[tauri::command]
fn toggle_meter_window(window: tauri::Window) {
    if let Some(meter) = window.app_handle().get_window("main") {
        if meter.is_visible().unwrap() {
            meter.hide().unwrap();
        } else {
            meter.show().unwrap();
        }
    }
}

#[tauri::command]
fn open_url(window: tauri::Window, url: String) {
    if let Some(logs) = window.app_handle().get_window("logs") {
        logs.emit("redirect-url", url).unwrap();
    }
}