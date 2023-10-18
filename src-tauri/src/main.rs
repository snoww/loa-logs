#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod parser;
mod resources;
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Result;
use flexi_logger::{
    Cleanup, Criterion, DeferredNow, Duplicate, FileSpec, Logger, Naming, WriteMode,
};
use hashbrown::HashMap;
use log::{info, warn, Record};
use parser::models::*;

use rusqlite::{params, params_from_iter, Connection};
use tauri::{
    api::process::Command, CustomMenuItem, LogicalPosition, LogicalSize, Manager, Position, Size,
    SystemTray, SystemTrayEvent, SystemTrayMenu, SystemTrayMenuItem, WindowBuilder,
};
use tauri_plugin_window_state::{AppHandleExt, StateFlags, WindowExt};
use window_vibrancy::{apply_blur, clear_blur};

#[tokio::main]
async fn main() -> Result<()> {
    let mut logger = Logger::try_with_str("info")?
        .log_to_file(
            FileSpec::default()
                .suppress_timestamp()
                .basename("loa_logs"),
        )
        .use_utc()
        .write_mode(WriteMode::BufferAndFlush)
        .append()
        .format(default_format_with_time)
        .rotate(
            Criterion::Size(5_000_000),
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
    let load_saved_pos = CustomMenuItem::new("load".to_string(), "Load Saved");
    let save_current_pos = CustomMenuItem::new("save".to_string(), "Save Position");
    let reset = CustomMenuItem::new("reset".to_string(), "Reset Window");
    let tray_menu = SystemTrayMenu::new()
        .add_item(show_logs)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(show_meter)
        .add_item(hide_meter)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(save_current_pos)
        .add_item(load_saved_pos)
        .add_item(reset)
        .add_native_item(SystemTrayMenuItem::Separator)
        .add_item(quit);

    let system_tray = SystemTray::new().with_menu(tray_menu);

    tauri::Builder::default()
        .setup(|app| {
            info!("starting app v{}", app.package_info().version.to_string());

            let resource_path = app
                .path_resolver()
                .resource_dir()
                .expect("could not get resource dir");

            #[cfg(not(debug_assertions))]
            {
                resources::Resources::new(resource_path.clone()).extract()?;
            }

            let settings = read_settings(&resource_path).ok();

            let meter_window = app.get_window("main").unwrap();
            meter_window
                .set_always_on_top(true)
                .expect("failed to set windows always on top");
            meter_window
                .restore_state(StateFlags::all())
                .expect("failed to restore window state");
            // #[cfg(debug_assertions)]
            // {
            //     meter_window.open_devtools();
            // }

            let mut raw_socket = false;
            let mut ip: String;
            let mut port = 6040;

            if let Some(settings) = settings {
                info!("settings loaded");
                if settings.general.auto_iface {
                    ip = meter_core::get_most_common_ip().unwrap();
                    info!("auto_iface enabled, using ip: {}", ip);
                } else {
                    ip = settings.general.ip;
                    let interface = settings.general.if_desc;
                    info!(
                        "manual interface set, ip: {} and interface: {}",
                        ip, interface
                    );
                    let os_interfaces = get_network_interfaces();
                    let right_name: Vec<&(String, String)> = os_interfaces
                        .iter()
                        .filter(|iface| iface.0 == interface)
                        .collect();
                    if !right_name.is_empty() {
                        let perfect_match =
                            right_name.clone().into_iter().find(|iface| iface.1 == ip);
                        if perfect_match.is_none() {
                            //in case of multiple interfaces with same name, try the first one
                            ip = right_name[0].1.clone(); //get the up to date ip
                            warn!("ip for manual interface was wrong, using ip: {}", ip);
                        }
                    } else {
                        ip = meter_core::get_most_common_ip().unwrap();
                        warn!("manually set interface not found, using default ip: {}", ip);
                    }
                    if settings.general.port > 0 {
                        port = settings.general.port;
                        info!("using port: {}", port);
                    }
                    raw_socket = settings.general.raw_socket;
                    if raw_socket {
                        info!("using raw socket");
                    } else {
                        info!("using npcap");
                    }
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

            let _logs_window =
                WindowBuilder::new(app, "logs", tauri::WindowUrl::App("/logs".into()))
                    .title("LOA Logs")
                    .min_inner_size(650.0, 300.0)
                    .inner_size(800.0, 500.0)
                    .build()
                    .expect("failed to create log window");

            tokio::task::spawn_blocking(move || {
                parser::start(meter_window, ip, port, raw_socket).map_err(|e| {
                    warn!("unexpected error occurred in parser: {}", e);
                })
            });

            // #[cfg(debug_assertions)]
            // {
            //     _logs_window.open_devtools();
            // }

            Ok(())
        })
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .plugin(tauri_plugin_single_instance::init(|_app, _argv, _cwd| {}))
        .on_window_event(|event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event.event() {
                api.prevent_close();

                if event.window().label() == "main" {
                    event
                        .window()
                        .app_handle()
                        .save_window_state(StateFlags::all())
                        .expect("failed to save window state");
                    event.window().app_handle().exit(0);
                } else if event.window().label() == "logs" {
                    event.window().hide().unwrap();
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
                    meter.set_ignore_cursor_events(false).unwrap()
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
                        meter.unminimize().unwrap();
                        meter.set_ignore_cursor_events(false).unwrap()
                    }
                }
                "load" => {
                    if let Some(meter) = app.get_window("main") {
                        meter.restore_state(StateFlags::all()).unwrap();
                    }
                }
                "save" => {
                    if let Some(meter) = app.get_window("main") {
                        meter
                            .app_handle()
                            .save_window_state(StateFlags::all())
                            .unwrap();
                    }
                }
                "reset" => {
                    if let Some(meter) = app.get_window("main") {
                        meter
                            .set_size(Size::Logical(LogicalSize {
                                width: 500.0,
                                height: 350.0,
                            }))
                            .unwrap();
                        meter
                            .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                            .unwrap();
                        meter.show().unwrap();
                        meter.unminimize().unwrap();
                        meter.set_focus().unwrap();
                        meter.set_ignore_cursor_events(false).unwrap();
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
            delete_encounters,
            toggle_meter_window,
            toggle_logs_window,
            open_url,
            save_settings,
            get_settings,
            open_folder,
            open_db_path,
            delete_encounters_below_min_duration,
            get_db_info,
            disable_blur,
            enable_blur,
            get_network_interfaces,
            write_log,
            toggle_encounter_favorite
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
            misc TEXT,
            difficulty TEXT
            favorite BOOLEAN NOT NULL DEFAULT 0,
            cleared BOOLEAN,
            version INTEGER NOT NULL DEFAULT 1
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

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('encounter') WHERE name='difficulty'")
        .unwrap();
    let column_count: u32 = stmt.query_row([], |row| row.get(0)).unwrap();
    if column_count == 0 {
        conn.execute("ALTER TABLE encounter ADD COLUMN difficulty TEXT", [])
            .expect("failed to add column");
    }

    let mut stmt = conn
        .prepare("SELECT COUNT(*) FROM pragma_table_info('encounter') WHERE name='favorite'")
        .unwrap();
    let column_count: u32 = stmt.query_row([], |row| row.get(0)).unwrap();
    if column_count == 0 {
        conn.execute(
            "ALTER TABLE encounter ADD COLUMN favorite BOOLEAN DEFAULT 0",
            [],
        )
        .expect("failed to add columns");
        conn.execute(
            "ALTER TABLE encounter ADD COLUMN version INTEGER DEFAULT 1",
            [],
        )
        .expect("failed to add columns");
        conn.execute("ALTER TABLE encounter ADD COLUMN cleared BOOLEAN", [])
            .expect("failed to add columns");
        conn.execute(
            "CREATE INDEX IF NOT EXISTS encounter_favorite_index ON encounter (favorite);",
            [],
        )
        .expect("failed to add index");
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
    search: String,
    filter: SearchFilter,
) -> EncountersOverview {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");

    let min_duration = filter.min_duration * 1000;

    let mut params = vec![
        min_duration.to_string(),
        search.clone(),
        search.clone(),
        search,
    ];

    let boss_filter = if !filter.bosses.is_empty() {
        let placeholders: Vec<String> = filter.bosses.iter().map(|_| "?".to_string()).collect();
        filter.bosses.into_iter().for_each(|boss| params.push(boss));
        format!("AND (current_boss IN ({}))", placeholders.join(","))
    } else {
        "".to_string()
    };

    let class_filter = if !filter.classes.is_empty() {
        let placeholders: Vec<String> = filter.classes.iter().map(|_| "?".to_string()).collect();
        filter
            .classes
            .into_iter()
            .for_each(|class| params.push(class));
        format!("AND (class IN ({}))", placeholders.join(","))
    } else {
        "".to_string()
    };

    let raid_clear_filter = if filter.cleared {
        "AND json_extract(misc, '$.raidClear') IS NOT NULL".to_string()
    } else {
        "".to_string()
    };

    let favorite_filter = if filter.favorite {
        "AND favorite = 1".to_string()
    } else {
        "".to_string()
    };

    let difficulty_filter = if !filter.difficulty.is_empty() {
        format!("AND difficulty = '{}'", filter.difficulty)
    } else {
        "".to_string()
    };

    let count_params = params.clone();

    let query = format!("SELECT
    e.id,
    e.fight_start,
    e.current_boss,
    e.duration,
    e.difficulty,
    e.favorite,
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
        {} {} {} {} {}
    GROUP BY encounter_id
    ORDER BY e.fight_start DESC
    LIMIT ?
    OFFSET ?", boss_filter, class_filter, raid_clear_filter, favorite_filter, difficulty_filter);

    let mut stmt = conn.prepare_cached(&query).unwrap();

    let offset = (page - 1) * page_size;

    params.push(page_size.to_string());
    params.push(offset.to_string());

    let encounter_iter = stmt
        .query_map(params_from_iter(params), |row| {
            let classes = row.get(6).unwrap_or_else(|_| "".to_string());

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
                difficulty: row.get(4)?,
                favorite: row.get(5)?,
            })
        })
        .expect("could not query encounters");

    let mut encounters: Vec<EncounterPreview> = Vec::new();
    for encounter in encounter_iter {
        encounters.push(encounter.unwrap());
    }

    let query = format!("
    SElECT COUNT(*)
    FROM (SELECT encounter_id
        FROM encounter e
        JOIN entity ent ON e.id = ent.encounter_id
        WHERE duration > ? AND ((current_boss LIKE '%' || ? || '%') OR (ent.class LIKE '%' || ? || '%') OR (ent.name LIKE '%' || ? || '%'))
            {} {} {} {} {}
        GROUP BY encounter_id)
        ", boss_filter, class_filter, raid_clear_filter, favorite_filter, difficulty_filter);

    let count: i32 = conn
        .query_row_and_then(&query, params_from_iter(count_params), |row| row.get(0))
        .expect("could not get encounter count");

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
       misc,
       difficulty,
       favorite
    FROM encounter
    WHERE id = ?
    ;",
        )
        .unwrap();

    let mut encounter = encounter_stmt
        .query_row(params![id], |row| {
            let buff_str = row.get(10).unwrap_or_else(|_| "".to_string());
            let buffs = serde_json::from_str::<HashMap<i32, StatusEffect>>(buff_str.as_str())
                .unwrap_or_else(|_| HashMap::new());

            let debuff_str = row.get(11).unwrap_or_else(|_| "".to_string());
            let debuffs = serde_json::from_str::<HashMap<i32, StatusEffect>>(debuff_str.as_str())
                .unwrap_or_else(|_| HashMap::new());

            let misc_str = row.get(12).unwrap_or_else(|_| "".to_string());
            let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
                .map(Some)
                .unwrap_or_else(|_| None);

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
                difficulty: row.get(13)?,
                favorite: row.get(14)?,
                ..Default::default()
            })
        })
        .unwrap_or_else(|_| Encounter::default());

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
            let skill_str = row.get(7).unwrap_or_else(|_| "".to_string());
            let skills = serde_json::from_str::<HashMap<i32, Skill>>(skill_str.as_str())
                .unwrap_or_else(|_| HashMap::new());

            let damage_stats_str = row.get(8).unwrap_or_else(|_| "".to_string());

            let damage_stats = serde_json::from_str::<DamageStats>(damage_stats_str.as_str())
                .unwrap_or_else(|_| DamageStats::default());

            let skill_stats_str = row.get(9).unwrap_or_else(|_| "".to_string());
            let skill_stats = serde_json::from_str::<SkillStats>(skill_stats_str.as_str())
                .unwrap_or_else(|_| SkillStats::default());

            let entity_type = row.get(11).unwrap_or_else(|_| "".to_string());

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
                entity_type: EntityType::from_str(entity_type.as_str())
                    .unwrap_or(EntityType::UNKNOWN),
                npc_id: row.get(12)?,
                ..Default::default()
            })
        })
        .unwrap();

    let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
    for entity in entity_iter.flatten() {
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
fn toggle_encounter_favorite(window: tauri::Window, id: i32) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");

    let conn = get_db_connection(&path).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    UPDATE encounter
    SET favorite = NOT favorite
    WHERE id = ?;
    ",
        )
        .unwrap();

    stmt.execute(params![id]).unwrap();
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

    info!("deleting encounter: {}", id);

    stmt.execute(params![id]).unwrap();
}

#[tauri::command]
fn delete_encounters(window: tauri::Window, ids: Vec<i32>) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    conn.execute("PRAGMA foreign_keys = ON;", params![])
        .unwrap();

    let placeholders: Vec<String> = ids.iter().map(|_| "?".to_string()).collect();
    let placeholders_str = placeholders.join(",");

    let sql = format!("DELETE FROM encounter WHERE id IN ({})", placeholders_str);
    let mut stmt = conn.prepare_cached(&sql).unwrap();

    info!("deleting encounters: {:?}", ids);

    stmt.execute(params_from_iter(ids)).unwrap();
}

#[tauri::command]
fn toggle_meter_window(window: tauri::Window) {
    if let Some(meter) = window.app_handle().get_window("main") {
        if meter.is_visible().unwrap() {
            meter.hide().unwrap();
        } else {
            meter.show().unwrap();
            meter.set_ignore_cursor_events(false).unwrap();
        }
    }
}

#[tauri::command]
fn toggle_logs_window(window: tauri::Window) {
    if let Some(logs) = window.app_handle().get_window("logs") {
        if logs.is_visible().unwrap() {
            logs.hide().unwrap();
        } else {
            logs.emit("redirect-url", "logs").unwrap();
            logs.show().unwrap();
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
fn open_db_path(window: tauri::Window) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    info!("open_db_path: {}", path.display());
    Command::new("explorer")
        .args([path.to_str().unwrap()])
        .spawn()
        .ok();
}

#[tauri::command]
fn delete_encounters_below_min_duration(window: tauri::Window, min_duration: i64) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    conn.execute(
        "
        DELETE FROM encounter
        WHERE duration < ?;
    ",
        params![min_duration * 1000],
    )
    .unwrap();
    conn.execute("VACUUM;", params![]).unwrap();
}

#[tauri::command]
fn get_db_info(window: tauri::Window, min_duration: i64) -> EncounterDbInfo {
    let mut path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
    let encounter_count = conn
        .query_row("SELECT COUNT(*) FROM encounter;", [], |row| row.get(0))
        .unwrap();
    let encounter_filtered_count = conn
        .query_row(
            "SELECT COUNT(*) FROM encounter WHERE duration >= ?;",
            params![min_duration * 1000],
            |row| row.get(0),
        )
        .unwrap();

    path.push("encounters.db");
    let metadata = fs::metadata(path).expect("could not get db metadata");

    let size_in_bytes = metadata.len();
    let size_in_kb = size_in_bytes as f64 / 1024.0;
    let size_in_mb = size_in_kb / 1024.0;

    let size_str = if size_in_mb < 1.0 {
        format!("{:.2} KB", size_in_kb)
    } else {
        format!("{:.2} MB", size_in_mb)
    };

    EncounterDbInfo {
        size: size_str,
        total_encounters: encounter_count,
        total_encounters_filtered: encounter_filtered_count,
    }
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

#[tauri::command]
fn write_log(message: String) {
    info!("{}", message);
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
