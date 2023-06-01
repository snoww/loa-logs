#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
use std::{
    fs::File,
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use flexi_logger::{
    Cleanup, Criterion, Duplicate, FileSpec, Logger, Naming, WriteMode, DeferredNow,
};
use hashbrown::HashMap;
use log::{info, warn, Record};
use parser::models::*;

use rusqlite::{params, Connection};
use tauri::{
    api::process::Command, CustomMenuItem, Manager, SystemTray, SystemTrayEvent, SystemTrayMenu,
    SystemTrayMenuItem, WindowBuilder,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use window_vibrancy::{apply_blur, clear_blur};

#[tokio::main]
async fn main() -> Result<()> {
    let mut logger = Logger::try_with_str("info, tao=off")?
        .log_to_file(FileSpec::default().suppress_timestamp().basename("loa_logs"))
        .use_utc()
        .write_mode(WriteMode::BufferAndFlush)
        .append()
        .format(default_format_with_time)
        .rotate(
            Criterion::Size(1_000_000),
            Naming::Timestamps,
            Cleanup::KeepLogFiles(2),
        );

    #[cfg(debug_assertions)]
    {
        logger = logger.duplicate_to_stdout(Duplicate::All);
    }

    logger.start()?;

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
            let resource_path = app
                .path_resolver()
                .resource_dir()
                .expect("could not get resource dir");
            let settings = read_settings(&resource_path).ok();

            let meter_window = app.get_window("main").unwrap();
            meter_window
                .set_always_on_top(true)
                .expect("failed to set windows always on top");
            meter_window
                .restore_state(StateFlags::all())
                .expect("failed to restore window state");
            #[cfg(debug_assertions)]
            {
                meter_window.open_devtools();
            }

            let ip: String;
            let mut port = 6040;

            if let Some(settings) = settings {
                if settings.general.auto_iface {
                    ip = meter_core::get_most_common_ip().unwrap();
                    info!("auto_iface enabled, using ip: {}", ip);
                } else {
                    ip = settings.general.ip;
                    if settings.general.port > 0 {
                        port = settings.general.port;
                    }
                    info!(
                        "manual interface, using ip: {}, port: {}",
                        ip, port
                    )
                }
            } else {
                ip = meter_core::get_most_common_ip().unwrap();
                info!("settings not found, auto_iface enabled, using ip: {}", ip);
            }

            match setup_db(resource_path) {
                Ok(_) => (),
                Err(e) => {
                    warn!("error setting up database: {}", e);
                }
            }
            tokio::task::spawn(async move {
                parser::start(meter_window, ip, port).expect("failed to start parser");
            });

            let _logs_window =
                WindowBuilder::new(app, "logs", tauri::WindowUrl::App("/logs".into()))
                    .title("LOA Logs")
                    .min_inner_size(650.0, 300.0)
                    .inner_size(800.0, 500.0)
                    .build()
                    .expect("failed to create log window");
            #[cfg(debug_assertions)]
            {
                _logs_window.open_devtools();
            }

            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                if event.window().label() == "logs" {
                    event.window().hide().unwrap();
                    api.prevent_close();
                }
                if event.window().label() == "main" {
                    event
                        .window()
                        .app_handle()
                        .save_window_state(StateFlags::all())
                        .expect("failed to save window state");
                    event.window().app_handle().exit(0);
                }
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
            SystemTrayEvent::MenuItemClick { id, .. } => match id.as_str() {
                "quit" => {
                    app.save_window_state(StateFlags::all())
                        .expect("failed to save window state");
                    app.exit(0);
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
            },
            _ => {}
        })
        .invoke_handler(tauri::generate_handler![
            load_encounters_preview,
            load_encounter,
            get_encounter_count,
            open_most_recent_encounter,
            delete_encounter,
            toggle_meter_window,
            open_url,
            save_settings,
            get_settings,
            check_old_db_location_exists,
            copy_db,
            open_folder,
            disable_blur,
            enable_blur,
            get_network_interfaces
        ])
        .run(tauri::generate_context!())
        .expect("error while running application");

    Ok(())
}

fn get_db_connection(resource_path: &Path) -> Result<Connection, String> {
    let mut path = resource_path.to_path_buf();
    path.push("encounters.db");
    if !path.exists() {
        setup_db(path.clone())?;
    }
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(e) => {
            return Err(e.to_string());
        }
    };
    Ok(conn)
}

fn setup_db(resource_path: PathBuf) -> Result<(), String> {
    let mut path = resource_path;
    path.push("encounters.db");
    let conn = match Connection::open(path) {
        Ok(conn) => conn,
        Err(e) => {
            return Err(e.to_string());
        }
    };

    match conn.execute_batch(
        "
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
            debuffs TEXT,
            misc TEXT
        );
        CREATE INDEX IF NOT EXISTS encounter_fight_start_index
        ON encounter (fight_start desc);
        CREATE INDEX IF NOT EXISTS encounter_current_boss_index
        ON encounter (current_boss);
        ",
    ) {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('encounter') WHERE name='misc'")
        .unwrap();
    let column_count: u32 = stmt.query_row([], |row| row.get(0)).unwrap();
    if column_count == 0 {
        conn.execute("ALTER TABLE encounter ADD COLUMN misc TEXT", [])
            .expect("failed to add column");
    }

    match conn.execute_batch(
        "
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
        ",
    ) {
        Ok(_) => (),
        Err(e) => {
            return Err(e.to_string());
        }
    }

    Ok(())
}

#[tauri::command]
fn load_encounters_preview(
    window: tauri::Window,
    page: i32,
    page_size: i32,
    min_duration: i32,
    search: String,
) -> EncountersOverview {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");

    let mut stmt = conn.prepare_cached("
    SELECT
        e.id,
        e.fight_start,
        e.current_boss,
        e.duration,
        (
            SELECT GROUP_CONCAT(ordered_classes.class_info, ',')
            FROM (
                SELECT en.class_id || ':' || en.name AS class_info
                FROM entity en
                WHERE en.encounter_id = e.id AND en.entity_type = 'PLAYER'
                ORDER BY json_extract(en.damage_stats, '$.dps') DESC
            ) AS ordered_classes
        ) AS classes
    FROM encounter e
    JOIN entity ent ON e.id = ent.encounter_id
    WHERE e.duration > ? AND ((current_boss LIKE '%' || ? || '%') OR (ent.class LIKE '%' || ? || '%') OR (ent.name LIKE '%' || ? || '%'))
    GROUP BY encounter_id
    ORDER BY e.fight_start DESC
    LIMIT ?
    OFFSET ?
    ")
    .unwrap();

    let offset = (page - 1) * page_size;
    let min_duration = min_duration * 1000;

    let encounter_iter = stmt
        .query_map(
            [
                min_duration.to_string(),
                search.to_string(),
                search.to_string(),
                search.to_string(),
                page_size.to_string(),
                offset.to_string(),
            ],
            |row| {
                let classes = match row.get(4) {
                    Ok(classes) => classes,
                    Err(_) => "".to_string(),
                };

                let (classes, names) = classes
                    .split(',')
                    .map(|s| {
                        let info: Vec<&str> = s.split(':').collect();
                        (info[0].parse::<i32>().unwrap_or(101), info[1].to_string())
                    })
                    .unzip();

                Ok(EncounterPreview {
                    id: row.get(0)?,
                    fight_start: row.get(1)?,
                    boss_name: row.get(2)?,
                    duration: row.get(3)?,
                    classes,
                    names,
                })
            },
        )
        .expect("could not query encounters");

    let mut encounters: Vec<EncounterPreview> = Vec::new();
    for encounter in encounter_iter {
        encounters.push(encounter.unwrap());
    }

    let count: i32 = conn.query_row_and_then("
    SElECT COUNT(*)
    FROM (SELECT encounter_id
        FROM encounter e
        JOIN entity ent ON e.id = ent.encounter_id
        WHERE duration > ? AND ((current_boss LIKE '%' || ? || '%') OR (ent.class LIKE '%' || ? || '%') OR (ent.name LIKE '%' || ? || '%'))
        GROUP BY encounter_id)
    ", [min_duration.to_string(), search.to_string(), search.to_string(), search], |row| {
        row.get(0)
    }).expect("could not get encounter count");

    EncountersOverview {
        encounters,
        total_encounters: count,
    }
}

#[tauri::command]
fn load_encounter(window: tauri::Window, id: String) -> Encounter {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut encounter_stmt = conn
        .prepare_cached(
            "
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
    ;",
        )
        .unwrap();

    let mut encounter = match encounter_stmt.query_row(params![id], |row| {
        let buff_str = match row.get(10) {
            Ok(buff_str) => buff_str,
            Err(_) => "".to_string(),
        };

        let buffs = match serde_json::from_str::<HashMap<i32, StatusEffect>>(buff_str.as_str()) {
            Ok(v) => v,
            Err(_) => HashMap::new(),
        };

        let debuff_str = match row.get(11) {
            Ok(debuff_str) => debuff_str,
            Err(_) => "".to_string(),
        };
        let debuffs = match serde_json::from_str::<HashMap<i32, StatusEffect>>(debuff_str.as_str())
        {
            Ok(v) => v,
            Err(_) => HashMap::new(),
        };

        let misc_str = match row.get(12) {
            Ok(misc_str) => misc_str,
            Err(_) => "".to_string(),
        };
        let misc = match serde_json::from_str::<EncounterMisc>(misc_str.as_str()) {
            Ok(v) => Some(v),
            Err(_) => None,
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
        Err(_) => return Encounter::default(),
    };

    let mut entity_stmt = conn
        .prepare_cached(
            "
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
        last_update,
        entity_type,
        npc_id
    FROM entity
    WHERE encounter_id = ?;
    ",
        )
        .unwrap();

    let entity_iter = entity_stmt
        .query_map(params![id], |row| {
            let skill_str = match row.get(7) {
                Ok(skill_str) => skill_str,
                Err(_) => "".to_string(),
            };
            let skills = match serde_json::from_str::<HashMap<i32, Skill>>(skill_str.as_str()) {
                Ok(v) => v,
                Err(_) => HashMap::new(),
            };

            let damage_stats_str = match row.get(8) {
                Ok(damage_stats_str) => damage_stats_str,
                Err(_) => "".to_string(),
            };

            let damage_stats = match serde_json::from_str::<DamageStats>(damage_stats_str.as_str())
            {
                Ok(v) => v,
                Err(_) => DamageStats::default(),
            };

            let skill_stats_str = match row.get(9) {
                Ok(skill_stats_str) => skill_stats_str,
                Err(_) => "".to_string(),
            };
            let skill_stats = match serde_json::from_str::<SkillStats>(skill_stats_str.as_str()) {
                Ok(v) => v,
                Err(_) => SkillStats::default(),
            };

            let entity_type = match row.get(11) {
                Ok(entity_type) => entity_type,
                Err(_) => "".to_string(),
            };

            Ok(EncounterEntity {
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
                entity_type: EntityType::from_str(entity_type.as_str()).unwrap(),
                npc_id: row.get(12)?,
                ..Default::default()
            })
        })
        .unwrap();

    let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
    for entity in entity_iter {
        let entity = entity.unwrap();
        entities.insert(entity.name.to_string(), entity);
    }

    encounter.entities = entities;

    encounter
}

#[tauri::command]
fn get_encounter_count(window: tauri::Window) -> i32 {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached("SELECT COUNT(*) FROM encounter;")
        .unwrap();

    let count: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    count.unwrap_or(0)
}

#[tauri::command]
fn open_most_recent_encounter(window: tauri::Window) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    SELECT id
    FROM encounter
    ORDER BY fight_start DESC
    LIMIT 1;
    ",
        )
        .unwrap();

    let id_result: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    if let Some(logs) = window.app_handle().get_window("logs") {
        match id_result {
            Ok(id) => {
                logs.emit("show-latest-encounter", id.to_string()).unwrap();
            }
            Err(_) => {
                logs.emit("redirect-url", "logs").unwrap();
            }
        }
    }
}

#[tauri::command]
fn delete_encounter(window: tauri::Window, id: String) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    conn.execute("PRAGMA foreign_keys = ON;", params![])
        .unwrap();
    let mut stmt = conn
        .prepare_cached(
            "
        DELETE FROM encounter
        WHERE id = ?;
    ",
        )
        .unwrap();

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

#[tauri::command]
fn save_settings(window: tauri::Window, settings: Settings) {
    let mut path: PathBuf = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    path.push("settings.json");
    let mut file = File::create(path).expect("could not create settings file");
    file.write_all(serde_json::to_string_pretty(&settings).unwrap().as_bytes())
        .expect("could not write to settings file");
}

fn read_settings(resource_path: &Path) -> Result<Settings, Box<dyn std::error::Error>> {
    let mut path = resource_path.to_path_buf();
    path.push("settings.json");
    let mut file = File::open(path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    let settings = serde_json::from_str(&contents)?;
    Ok(settings)
}

#[tauri::command]
fn get_settings(window: tauri::Window) -> Option<Settings> {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    read_settings(&path).ok()
}

#[tauri::command]
fn get_network_interfaces() -> Vec<(String, String)> {
    let interfaces = meter_core::get_network_interfaces();
    info!("get_network_interfaces: {:?}", interfaces);
    interfaces
}

#[tauri::command]
fn check_old_db_location_exists() -> bool {
    let user_dir = std::env::var("USERPROFILE");
    match user_dir {
        Ok(user_dir) => {
            let old_path = PathBuf::from(format!(
                "{}/AppData/Local/Programs/LOA Logs/encounters.db",
                user_dir
            ));
            old_path.exists()
        }
        Err(_) => false,
    }
}

#[tauri::command]
fn copy_db(window: tauri::Window) -> Result<(), String> {
    let user_dir = std::env::var("USERPROFILE");
    match user_dir {
        Ok(user_dir) => {
            let old_path = PathBuf::from(format!(
                "{}/AppData/Local/Programs/LOA Logs/encounters.db",
                user_dir
            ));
            let mut new_path = window
                .app_handle()
                .path_resolver()
                .resource_dir()
                .expect("could not get resource dir");
            new_path.push("encounters.db");
            match std::fs::copy(old_path, new_path) {
                Ok(_) => Ok(()),
                Err(e) => {
                    warn!("copy_db: Error copying db: {}", e);
                    Err(e.to_string())
                }
            }
        }
        Err(_) => Err("Could not get user dir".to_string()),
    }
}

#[tauri::command]
fn open_folder(path: String) {
    let mut path = path;
    if path.contains("USERPROFILE") {
        if let Ok(user_dir) = std::env::var("USERPROFILE") {
            path = path.replace("USERPROFILE", user_dir.as_str());
        }
    }
    info!("open_folder: {}", path);
    Command::new("explorer").args([path.as_str()]).spawn().ok();
}

#[tauri::command]
fn disable_blur(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window("main") {
        clear_blur(&meter_window).ok();
    }
}

#[tauri::command]
fn enable_blur(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window("main") {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

fn default_format_with_time(
    w: &mut dyn std::io::Write,
    now: &mut DeferredNow,
    record: &Record,
) -> Result<(), std::io::Error> {
    write!(
        w,
        "[{}] {} [{}] {}",
        now.format("%Y-%m-%dT%H:%M:%S%.6fZ"),
        record.level(),
        record.module_path().unwrap_or("<unnamed>"),
        record.args()
    )
}