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

use anyhow::Result;
use flate2::read::GzDecoder;
use hashbrown::HashMap;
use log::{error, info, warn};
use models::*;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::{fs, io::Read, path::PathBuf, str::FromStr};

use app::autostart::{AutoLaunch, AutoLaunchManager};
use rusqlite::{params, params_from_iter, Connection, Transaction};
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::{
    menu::MenuBuilder,
    tray::{MouseButton, MouseButtonState, TrayIconEvent},
    Emitter, LogicalPosition, LogicalSize, Manager, Position, Size, State, WindowEvent,
};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_updater::UpdaterExt;
use tauri_plugin_window_state::{AppHandleExt, WindowExt};
use window_vibrancy::{apply_blur, clear_blur};

use crate::constants::*;
use crate::context::AppContext;
use crate::data::AssetPreloader;
use crate::misc::load_windivert;
use crate::settings::{Settings, SettingsManager};

#[tokio::main]
async fn main() -> Result<()> {
    let _ = app::logger::init()?;
    let tauri_context = tauri::generate_context!();
    let package_info = tauri_context.package_info();
    let context = AppContext::new(package_info.version.to_string())?;
    let settings_manager = SettingsManager::new(context.settings_path).expect("could not create settings");
    load_windivert(&context.current_dir).expect("could not load windivert dependencies");
    // load meter-data
    AssetPreloader::new()?;

    tauri::Builder::default()
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
        .setup(|app| {
            let app_handle = app.handle();
            info!("starting app v{}", app.package_info().version);
            app::panic::set_hook(app.handle());

            if let Err(e) = setup_db(app.handle()) {
                warn!("error setting up database: {e}");
            }

            setup_tray(app_handle)?;

            let app_path = std::env::current_exe()?.display().to_string();
            app.manage(AutoLaunchManager::new(&app_path));

            let update_checked = Arc::new(AtomicBool::new(false));
            let checked_clone = update_checked.clone();
            let cloned_app_handle = app_handle.clone();
            tauri::async_runtime::spawn(async move {
                match cloned_app_handle.updater().unwrap().check().await {
                    #[cfg(not(debug_assertions))]
                    Ok(Some(update)) => {
                        info!("update available, downloading update: v{}", update.version);

                        unload_driver();
                        remove_driver();

                        if let Err(e) = update.download_and_install(|_, _| {}, || {}).await {
                            error!("failed to download update: {}", e);
                        }
                    }
                    Err(e) => {
                        warn!("failed to get update: {e}");
                        checked_clone.store(true, Ordering::Relaxed);
                    }
                    _ => {
                        info!("no update available");
                        checked_clone.store(true, Ordering::Relaxed);
                    }
                }
            });

            let settings_manager = app_handle.state::<SettingsManager>();
            let settings = settings_manager.read()?;

            let meter_window = app.get_webview_window(METER_WINDOW_LABEL).unwrap();
            meter_window
                .restore_state(WINDOW_STATE_FLAGS)
                .expect("failed to restore window state");

            let mini_window = app.get_webview_window(METER_MINI_WINDOW_LABEL).unwrap();
            meter_window
                .restore_state(WINDOW_STATE_FLAGS)
                .expect("failed to restore window state");
            // #[cfg(debug_assertions)]
            // {
            //     meter_window.open_devtools();
            // }

            let logs_window = app.get_webview_window(LOGS_WINDOW_LABEL).unwrap();
            logs_window
                .restore_state(WINDOW_STATE_FLAGS)
                .expect("failed to restore window state");

            let mut port = 6040;

            if let Some(settings) = settings.clone() {
                info!("settings loaded");
                if settings.general.mini {
                    mini_window.show().unwrap();
                } else if !settings.general.hide_meter_on_start && !settings.general.mini {
                    meter_window.show().unwrap();
                }
                if !settings.general.hide_logs_on_start {
                    logs_window.show().unwrap();
                }
                if !settings.general.always_on_top {
                    meter_window.set_always_on_top(false).unwrap();
                    mini_window.set_always_on_top(false).unwrap();
                } else {
                    meter_window.set_always_on_top(true).unwrap();
                    mini_window.set_always_on_top(true).unwrap();
                }

                if settings.general.auto_iface && settings.general.port > 0 {
                    port = settings.general.port;
                }

                if settings.general.start_loa_on_start {
                    info!("auto launch game enabled");
                    start_loa_process(app.handle().clone());
                }
            } else {
                meter_window.show().unwrap();
                logs_window.show().unwrap();
            }

            remove_driver();

            // only start listening if we have live meter
            #[cfg(feature = "meter-core")]
            {
                let app = app.app_handle().clone();
                tokio::task::spawn_blocking(move || {
                    // only start listening when there's no update, otherwise unable to remove driver
                    while !update_checked.load(Ordering::Relaxed) {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }
                    info!("listening on port: {port}");
                    live::start(app, port, settings).map_err(|e| {
                        error!("unexpected error occurred in parser: {e}");
                    })
                });
            }

            // #[cfg(debug_assertions)]
            // {
            //     _logs_window.open_devtools();
            // }

            Ok(())
        })
        .on_window_event(|window, event| match event {
            WindowEvent::CloseRequested { api, .. } => {
                api.prevent_close();

                if window.label() == METER_WINDOW_LABEL || window.label() == METER_MINI_WINDOW_LABEL
                {
                    let app_handle = window.app_handle();
                    let meter_window = app_handle.get_webview_window(METER_WINDOW_LABEL).unwrap();
                    let logs_window = app_handle.get_webview_window(LOGS_WINDOW_LABEL).unwrap();

                    if logs_window.is_minimized().unwrap() {
                        logs_window.unminimize().unwrap();
                    }

                    if meter_window.is_minimized().unwrap() {
                        meter_window.unminimize().unwrap();
                    }

                    app_handle
                        .save_window_state(WINDOW_STATE_FLAGS)
                        .expect("failed to save window state");
                    unload_driver();
                    app_handle.exit(0);
                } else if window.label() == LOGS_WINDOW_LABEL {
                    window.hide().unwrap();
                }
            }
            WindowEvent::Focused(focused) if !focused => {
                window
                    .app_handle()
                    .save_window_state(WINDOW_STATE_FLAGS)
                    .expect("failed to save window state");
            }
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
            open_db_path,
            delete_encounters_below_min_duration,
            get_db_info,
            disable_blur,
            enable_blur,
            write_log,
            toggle_encounter_favorite,
            delete_all_encounters,
            delete_all_uncleared_encounters,
            enable_aot,
            disable_aot,
            set_clickthrough,
            optimize_database,
            check_start_on_boot,
            set_start_on_boot,
            check_loa_running,
            start_loa_process,
            get_sync_candidates,
            sync,
            remove_driver,
            unload_driver,
        ])
        .run(tauri_context)
        .expect("error while running application");

    Ok(())
}

fn setup_tray(app: &tauri::AppHandle) -> tauri::Result<()> {
    let menu = MenuBuilder::new(app)
        .text("show-logs", "Show Logs")
        .separator()
        .text("show-meter", "Show Meter")
        .text("hide", "Hide Meter")
        .separator()
        .text("start-loa", "Start Lost Ark")
        .separator()
        .text("reset", "Reset Window")
        .separator()
        .text("quit", "Quit")
        .build()?;

    fn show_window(window: &tauri::WebviewWindow) {
        window.show().unwrap();
        window.unminimize().unwrap();
        window.set_focus().unwrap();
        if window.label() == METER_WINDOW_LABEL {
            window.set_ignore_cursor_events(false).unwrap();
        }
    }

    fn get_meter_window(
        app: &tauri::AppHandle,
        settings: &Settings,
    ) -> Option<tauri::WebviewWindow> {
        let label = if settings.general.mini {
            METER_MINI_WINDOW_LABEL
        } else {
            METER_WINDOW_LABEL
        };
        app.get_webview_window(label)
    }

    let tray = app.tray_by_id("main").expect("should be in tauri config");
    tray.set_menu(Some(menu))?;
    tray.on_menu_event(move |app, event| match event.id().as_ref() {
        "quit" => {
            app.save_window_state(WINDOW_STATE_FLAGS)
                .expect("failed to save window state");
            unload_driver();
            app.exit(0);
        }
        "hide" => {
            if let Some(meter) = app.get_webview_window(METER_WINDOW_LABEL) {
                meter.hide().unwrap();
            }
            if let Some(mini) = app.get_webview_window(METER_MINI_WINDOW_LABEL) {
                mini.hide().unwrap();
            }
        }
        "show-meter" => {
            let settings_manager = app.state::<SettingsManager>();
            let settings = settings_manager.read().unwrap().unwrap_or_default();
            if let Some(meter) = get_meter_window(app, &settings) {
                show_window(&meter);
            }
        }
        "reset" => {
            let settings_manager = app.state::<SettingsManager>();
            let settings = settings_manager.read().unwrap().unwrap_or_default();

            if settings.general.mini {
                if let Some(mini) = app.get_webview_window(METER_MINI_WINDOW_LABEL) {
                    mini.set_size(Size::Logical(LogicalSize {
                        width: 1280.0,
                        height: 200.0,
                    }))
                    .unwrap();
                    mini.set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                        .unwrap();
                    show_window(&mini);
                }
            } else if let Some(meter) = app.get_webview_window(METER_WINDOW_LABEL) {
                meter
                    .set_size(Size::Logical(LogicalSize {
                        width: 500.0,
                        height: 350.0,
                    }))
                    .unwrap();
                meter
                    .set_position(Position::Logical(LogicalPosition { x: 100.0, y: 100.0 }))
                    .unwrap();
                show_window(&meter);
            }
        }
        "show-logs" => {
            if let Some(logs) = app.get_webview_window(LOGS_WINDOW_LABEL) {
                logs.show().unwrap();
                logs.unminimize().unwrap();
            }
        }
        "start-loa" => {
            start_loa_process(app.clone());
        }
        _ => {}
    });
    tray.on_tray_icon_event(|tray, event| {
        if let TrayIconEvent::Click {
            button: MouseButton::Left,
            button_state: MouseButtonState::Up,
            ..
        } = event
        {
            let app = tray.app_handle();
            let settings_manager = app.state::<SettingsManager>();
            let settings = settings_manager.read().unwrap().unwrap_or_default();

            if let Some(meter) = get_meter_window(app, &settings) {
                show_window(&meter);
            }
        }
    });

    Ok(())
}

fn get_db_path(app: &tauri::AppHandle) -> PathBuf {
    app::path::data_dir(app).join("encounters.db")
}

fn get_db_connection(app: &tauri::AppHandle) -> Result<Connection, rusqlite::Error> {
    let path = get_db_path(app);
    if !path.exists() {
        setup_db(app)?;
    }
    Connection::open(path)
}

fn setup_db(app: &tauri::AppHandle) -> Result<(), rusqlite::Error> {
    info!("setting up database");
    let mut conn = Connection::open(get_db_path(app))?;
    let tx = conn.transaction()?;

    // FIXME: replace me with idempotent migrations

    let mut stmt = tx.prepare("SELECT 1 FROM sqlite_master WHERE type=? AND name=?")?;
    if !stmt.exists(["table", "encounter"])? {
        info!("creating tables");
        migration_legacy_encounter(&tx)?;
        migration_legacy_entity(&tx)?;
    }

    // NOTE: for databases, where the bad migration code already ran
    migration_legacy_entity(&tx)?;

    if !stmt.exists(["table", "encounter_preview"])? {
        info!("optimizing searches");
        migration_legacy_encounter(&tx)?;
        migration_legacy_entity(&tx)?;
        migration_full_text_search(&tx)?;
    }

    if !stmt.exists(["table", "sync_logs"])? {
        info!("adding sync table");
        migration_sync(&tx)?;
    }

    migration_specs(&tx)?;

    migration_combat_power(&tx)?;

    migration_buff_summary(&tx)?;

    stmt.finalize()?;
    info!("finished setting up database");
    tx.commit()
}

fn migration_legacy_encounter(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(&format!(
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
        total_shielding INTEGER DEFAULT 0,
        total_effective_shielding INTEGER DEFAULT 0,
        applied_shield_buffs TEXT,
        misc TEXT,
        difficulty TEXT,
        favorite BOOLEAN NOT NULL DEFAULT 0,
        cleared BOOLEAN,
        version INTEGER NOT NULL DEFAULT {},
        boss_only_damage BOOLEAN NOT NULL DEFAULT 0
    );
    CREATE INDEX IF NOT EXISTS encounter_fight_start_index
    ON encounter (fight_start desc);
    CREATE INDEX IF NOT EXISTS encounter_current_boss_index
    ON encounter (current_boss);
    ",
        DB_VERSION
    ))?;

    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["encounter", "misc"])? {
        tx.execute("ALTER TABLE encounter ADD COLUMN misc TEXT", [])?;
    }
    if !stmt.exists(["encounter", "difficulty"])? {
        tx.execute("ALTER TABLE encounter ADD COLUMN difficulty TEXT", [])?;
    }
    if !stmt.exists(["encounter", "favorite"])? {
        tx.execute_batch(&format!(
            "
            ALTER TABLE encounter ADD COLUMN favorite BOOLEAN DEFAULT 0;
            ALTER TABLE encounter ADD COLUMN version INTEGER DEFAULT {};
            ALTER TABLE encounter ADD COLUMN cleared BOOLEAN;
            ",
            DB_VERSION,
        ))?;
    }
    if !stmt.exists(["encounter", "boss_only_damage"])? {
        tx.execute(
            "ALTER TABLE encounter ADD COLUMN boss_only_damage BOOLEAN NOT NULL DEFAULT 0",
            [],
        )?;
    }
    if !stmt.exists(["encounter", "total_shielding"])? {
        tx.execute_batch(
            "
                ALTER TABLE encounter ADD COLUMN total_shielding INTEGER DEFAULT 0;
                ALTER TABLE encounter ADD COLUMN total_effective_shielding INTEGER DEFAULT 0;
                ALTER TABLE encounter ADD COLUMN applied_shield_buffs TEXT;
                ",
        )?;
    }
    tx.execute("UPDATE encounter SET cleared = coalesce(json_extract(misc, '$.raidClear'), 0) WHERE cleared IS NULL;", [])?;
    stmt.finalize()
}

fn migration_legacy_entity(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS entity (
            name TEXT,
            character_id INTEGER,
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
            dps INTEGER,
            skill_stats TEXT,
            last_update INTEGER,
            engravings TEXT,
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
    )?;

    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "dps"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN dps INTEGER", [])?;
    }
    if !stmt.exists(["entity", "character_id"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN character_id INTEGER", [])?;
    }
    if !stmt.exists(["entity", "engravings"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN engravings TEXT", [])?;
    }
    if !stmt.exists(["entity", "gear_hash"])? {
        tx.execute("ALTER TABLE entity ADD COLUMN gear_hash TEXT", [])?;
    }
    tx.execute("UPDATE entity SET dps = coalesce(json_extract(damage_stats, '$.dps'), 0) WHERE dps IS NULL;", [])?;
    stmt.finalize()
}

fn migration_full_text_search(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE encounter_preview (
            id INTEGER PRIMARY KEY,
            fight_start INTEGER,
            current_boss TEXT,
            duration INTEGER,
            players TEXT,
            difficulty TEXT,
            local_player TEXT,
            my_dps INTEGER,
            favorite BOOLEAN NOT NULL DEFAULT 0,
            cleared BOOLEAN,
            boss_only_damage BOOLEAN NOT NULL DEFAULT 0,
            FOREIGN KEY (id) REFERENCES encounter(id) ON DELETE CASCADE
        );

        INSERT INTO encounter_preview SELECT
            id, fight_start, current_boss, duration, 
            (
                SELECT GROUP_CONCAT(class_id || ':' || name ORDER BY dps DESC)
                FROM entity
                WHERE encounter_id = encounter.id AND entity_type = 'PLAYER'
            ) AS players,
            difficulty, local_player,
            (
                SELECT dps
                FROM entity
                WHERE encounter_id = encounter.id AND name = encounter.local_player
            ) AS my_dps,
            favorite, cleared, boss_only_damage
        FROM encounter;

        DROP INDEX IF EXISTS encounter_fight_start_index;
        DROP INDEX IF EXISTS encounter_current_boss_index;
        DROP INDEX IF EXISTS encounter_favorite_index;
        DROP INDEX IF EXISTS entity_name_index;
        DROP INDEX IF EXISTS entity_class_index;

        ALTER TABLE encounter DROP COLUMN fight_start;
        ALTER TABLE encounter DROP COLUMN current_boss;
        ALTER TABLE encounter DROP COLUMN duration;
        ALTER TABLE encounter DROP COLUMN difficulty;
        ALTER TABLE encounter DROP COLUMN local_player;
        ALTER TABLE encounter DROP COLUMN favorite;
        ALTER TABLE encounter DROP COLUMN cleared;
        ALTER TABLE encounter DROP COLUMN boss_only_damage;

        ALTER TABLE encounter ADD COLUMN boss_hp_log BLOB;
        ALTER TABLE encounter ADD COLUMN stagger_log TEXT;

        CREATE INDEX encounter_preview_favorite_index ON encounter_preview(favorite);
        CREATE INDEX encounter_preview_fight_start_index ON encounter_preview(fight_start);
        CREATE INDEX encounter_preview_my_dps_index ON encounter_preview(my_dps);
        CREATE INDEX encounter_preview_duration_index ON encounter_preview(duration);

        CREATE VIRTUAL TABLE encounter_search USING fts5(
            current_boss, players, columnsize=0, detail=full,
            tokenize='trigram remove_diacritics 1',
            content=encounter_preview, content_rowid=id
        );
        INSERT INTO encounter_search(encounter_search) VALUES('rebuild');
        CREATE TRIGGER encounter_preview_ai AFTER INSERT ON encounter_preview BEGIN
            INSERT INTO encounter_search(rowid, current_boss, players)
            VALUES (new.id, new.current_boss, new.players);
        END;
        CREATE TRIGGER encounter_preview_ad AFTER DELETE ON encounter_preview BEGIN
            INSERT INTO encounter_search(encounter_search, rowid, current_boss, players)
            VALUES('delete', old.id, old.current_boss, old.players);
        END;
        CREATE TRIGGER encounter_preview_au AFTER UPDATE OF current_boss, players ON encounter_preview BEGIN
            INSERT INTO encounter_search(encounter_search, rowid, current_boss, players)
            VALUES('delete', old.id, old.current_boss, old.players);
            INSERT INTO encounter_search(rowid, current_boss, players)
            VALUES (new.id, new.current_boss, new.players);
        END;
        ",
    )
}

fn migration_sync(tx: &Transaction) -> Result<(), rusqlite::Error> {
    tx.execute_batch(
        "
        CREATE TABLE IF NOT EXISTS sync_logs (
        encounter_id INTEGER PRIMARY KEY,
        upstream_id TEXT,
        failed BOOLEAN NOT NULL DEFAULT 0,
        FOREIGN KEY (encounter_id) REFERENCES encounter (id) ON DELETE CASCADE
    );",
    )
}

fn migration_specs(tx: &Transaction) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "spec"])? {
        info!("adding spec info columns");
        tx.execute_batch(
            "
                ALTER TABLE entity ADD COLUMN spec TEXT;
                ALTER TABLE entity ADD COLUMN ark_passive_active BOOLEAN;
                ALTER TABLE entity ADD COLUMN ark_passive_data TEXT;
                ",
        )?;
    }

    stmt.finalize()
}

fn migration_combat_power(tx: &Transaction) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "combat_power"])? {
        info!("adding loadout hash and combat score columns");
        tx.execute("ALTER TABLE entity ADD COLUMN loadout_hash TEXT", [])?;
        tx.execute("ALTER TABLE entity ADD COLUMN combat_power REAL", [])?;
    }

    stmt.finalize()
}

fn migration_buff_summary(tx: &Transaction) -> Result<(), rusqlite::Error> {
    let mut stmt = tx.prepare("SELECT 1 FROM pragma_table_info(?) WHERE name=?")?;
    if !stmt.exists(["entity", "support_ap"])? {
        info!("adding support buff summary columns");
        tx.execute("ALTER TABLE entity ADD COLUMN support_ap REAL", [])?;
        tx.execute("ALTER TABLE entity ADD COLUMN support_brand REAL", [])?;
        tx.execute("ALTER TABLE entity ADD COLUMN support_identity REAL", [])?;
        tx.execute("ALTER TABLE entity ADD COLUMN support_hyper REAL", [])?;
    }

    stmt.finalize()
}

#[tauri::command]
fn load_encounters_preview(
    app: tauri::AppHandle,
    page: i32,
    page_size: i32,
    search: String,
    filter: SearchFilter,
) -> EncountersOverview {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let mut params = vec![];

    let join_clause = if search.len() > 2 {
        let escaped_search = search
            .split_whitespace()
            .map(|word| format!("\"{}\"", word.replace("\"", "")))
            .collect::<Vec<_>>()
            .join(" ");
        params.push(escaped_search);
        "JOIN encounter_search(?) ON encounter_search.rowid = e.id"
    } else {
        ""
    };

    params.push((filter.min_duration * 1000).to_string());

    let boss_filter = if !filter.bosses.is_empty() {
        let mut placeholders = "?,".repeat(filter.bosses.len());
        placeholders.pop(); // remove trailing comma
        params.extend(filter.bosses);
        format!("AND e.current_boss IN ({})", placeholders)
    } else {
        "".to_string()
    };

    let raid_clear_filter = if filter.cleared {
        "AND cleared = 1"
    } else {
        ""
    };

    let favorite_filter = if filter.favorite {
        "AND favorite = 1"
    } else {
        ""
    };

    let boss_only_damage_filter = if filter.boss_only_damage {
        "AND boss_only_damage = 1"
    } else {
        ""
    };

    let difficulty_filter = if !filter.difficulty.is_empty() {
        params.push(filter.difficulty);
        "AND difficulty = ?"
    } else {
        ""
    };

    let raids_only_filter = if filter.raids_only {
        "AND difficulty IS NOT NULL and difficulty != ''"
    } else {
        ""
    };

    let sort = format!("e.{}", filter.sort);

    let count_params = params.clone();

    let query = format!(
        "SELECT
    e.id,           -- 0
    e.fight_start,  -- 1
    e.current_boss, -- 2
    e.duration,     -- 3
    e.difficulty,   -- 4
    e.favorite,     -- 5
    e.cleared,      -- 6
    e.local_player, -- 7
    e.my_dps,       -- 8
    e.players,      -- 9
    le.spec,            -- 10
    le.support_ap,      -- 11
    le.support_brand,   -- 12
    le.support_identity,-- 13
    le.support_hyper    -- 14
    FROM encounter_preview e
    LEFT JOIN entity le ON le.encounter_id = e.id AND le.name = e.local_player
    {}
    WHERE e.duration > ? {}
    {} {} {} {} {}
    ORDER BY {} {}
    LIMIT ?
    OFFSET ?",
        join_clause,
        boss_filter,
        raid_clear_filter,
        favorite_filter,
        difficulty_filter,
        raids_only_filter,
        boss_only_damage_filter,
        sort,
        filter.order
    );

    let mut stmt = match conn.prepare_cached(&query) {
        Ok(stmt) => stmt,
        Err(e) => {
            info!("database not setup yet");
            return EncountersOverview::default();
        }
    };

    let offset = (page - 1) * page_size;

    params.push(page_size.to_string());
    params.push(offset.to_string());

    let encounter_iter = stmt
        .query_map(params_from_iter(params), |row| {
            let classes: String = row.get(9).unwrap_or_default();

            let (classes, names) = classes
                .split(',')
                .map(|s| {
                    let info: Vec<&str> = s.split(':').collect();
                    if info.len() != 2 {
                        return (101, "Unknown".to_string());
                    }
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
                cleared: row.get(6)?,
                local_player: row.get(7)?,
                my_dps: row.get(8).unwrap_or(0),
                spec: row.get(10).unwrap_or_default(),
                support_ap: row.get(11).unwrap_or_default(),
                support_brand: row.get(12).unwrap_or_default(),
                support_identity: row.get(13).unwrap_or_default(),
                support_hyper: row.get(14).unwrap_or_default(),
            })
        })
        .expect("could not query encounters");

    let encounters: Vec<EncounterPreview> = encounter_iter.collect::<Result<_, _>>().unwrap();

    let query = format!(
        "SELECT COUNT(*)
        FROM encounter_preview e {join_clause}
        WHERE duration > ? {boss_filter}
        {raid_clear_filter} {favorite_filter} {difficulty_filter} {boss_only_damage_filter}"
    );

    let count: i32 = conn
        .query_row_and_then(&query, params_from_iter(count_params), |row| row.get(0))
        .expect("could not get encounter count");

    EncountersOverview {
        encounters,
        total_encounters: count,
    }
}

#[tauri::command(async)]
fn load_encounter(app: tauri::AppHandle, id: String) -> Encounter {
    let conn = get_db_connection(&app).expect("could not get db connection");
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
       favorite,
       cleared,
       boss_only_damage,
       total_shielding,
       total_effective_shielding,
       applied_shield_buffs,
       boss_hp_log
    FROM encounter JOIN encounter_preview USING (id)
    WHERE id = ?
    ",
        )
        .unwrap();

    let mut compressed = false;
    let mut encounter = encounter_stmt
        .query_row(params![id], |row| {
            let misc_str: String = row.get(12).unwrap_or_default();
            let misc = serde_json::from_str::<EncounterMisc>(misc_str.as_str())
                .map(Some)
                .unwrap_or_default();

            let mut boss_hp_log: HashMap<String, Vec<BossHpLog>> = HashMap::new();

            if let Some(misc) = misc.as_ref() {
                let version = misc
                    .version
                    .clone()
                    .unwrap_or_default()
                    .split('.')
                    .map(|x| x.parse::<i32>().unwrap_or_default())
                    .collect::<Vec<_>>();

                if version[0] > 1
                    || (version[0] == 1 && version[1] >= 14)
                    || (version[0] == 1 && version[1] == 13 && version[2] >= 5)
                {
                    compressed = true;
                }

                if !compressed {
                    boss_hp_log = misc.boss_hp_log.clone().unwrap_or_default();
                }
            }

            let buffs: HashMap<u32, StatusEffect>;
            let debuffs: HashMap<u32, StatusEffect>;
            let applied_shield_buffs: HashMap<u32, StatusEffect>;
            if compressed {
                let raw_bytes: Vec<u8> = row.get(10).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut buff_string = String::new();
                decompress
                    .read_to_string(&mut buff_string)
                    .expect("could not decompress buffs");
                buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_string.as_str())
                    .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(11).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut debuff_string = String::new();
                decompress
                    .read_to_string(&mut debuff_string)
                    .expect("could not decompress debuffs");
                debuffs =
                    serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_string.as_str())
                        .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(19).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut applied_shield_buff_string = String::new();
                decompress
                    .read_to_string(&mut applied_shield_buff_string)
                    .expect("could not decompress applied_shield_buffs");
                applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
                    applied_shield_buff_string.as_str(),
                )
                .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(20).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut boss_string = String::new();
                decompress
                    .read_to_string(&mut boss_string)
                    .expect("could not decompress boss_hp_log");
                boss_hp_log =
                    serde_json::from_str::<HashMap<String, Vec<BossHpLog>>>(boss_string.as_str())
                        .unwrap_or_default();
            } else {
                let buff_str: String = row.get(10).unwrap_or_default();
                buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(buff_str.as_str())
                    .unwrap_or_default();
                let debuff_str: String = row.get(11).unwrap_or_default();
                debuffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(debuff_str.as_str())
                    .unwrap_or_default();
                let applied_shield_buff_str: String = row.get(19).unwrap_or_default();
                applied_shield_buffs = serde_json::from_str::<HashMap<u32, StatusEffect>>(
                    applied_shield_buff_str.as_str(),
                )
                .unwrap_or_default();
            }

            let total_shielding = row.get(17).unwrap_or_default();
            let total_effective_shielding = row.get(18).unwrap_or_default();

            Ok(Encounter {
                last_combat_packet: row.get(0)?,
                fight_start: row.get(1)?,
                local_player: row.get(2).unwrap_or("You".to_string()),
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
                    total_shielding,
                    total_effective_shielding,
                    applied_shield_buffs,
                    boss_hp_log,
                    ..Default::default()
                },
                difficulty: row.get(13)?,
                favorite: row.get(14)?,
                cleared: row.get(15)?,
                boss_only_damage: row.get(16)?,
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
        npc_id,
        character_id,
        engravings,
        spec,
        ark_passive_active,
        ark_passive_data,
        loadout_hash,
        combat_power
    FROM entity
    WHERE encounter_id = ?;
    ",
        )
        .unwrap();

    let entity_iter = entity_stmt
        .query_map(params![id], |row| {
            let skills: HashMap<u32, Skill>;
            let damage_stats: DamageStats;

            if compressed {
                let raw_bytes: Vec<u8> = row.get(7).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut skill_string = String::new();
                decompress
                    .read_to_string(&mut skill_string)
                    .expect("could not decompress skills");
                skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_string.as_str())
                    .unwrap_or_default();

                let raw_bytes: Vec<u8> = row.get(8).unwrap_or_default();
                let mut decompress = GzDecoder::new(raw_bytes.as_slice());
                let mut damage_stats_string = String::new();
                decompress
                    .read_to_string(&mut damage_stats_string)
                    .expect("could not decompress damage stats");
                damage_stats = serde_json::from_str::<DamageStats>(damage_stats_string.as_str())
                    .unwrap_or_default();
            } else {
                let skill_str: String = row.get(7).unwrap_or_default();
                skills = serde_json::from_str::<HashMap<u32, Skill>>(skill_str.as_str())
                    .unwrap_or_default();

                let damage_stats_str: String = row.get(8).unwrap_or_default();
                damage_stats = serde_json::from_str::<DamageStats>(damage_stats_str.as_str())
                    .unwrap_or_default();
            }

            let skill_stats_str: String = row.get(9).unwrap_or_default();
            let skill_stats =
                serde_json::from_str::<SkillStats>(skill_stats_str.as_str()).unwrap_or_default();

            let entity_type: String = row.get(11).unwrap_or_default();

            let engravings_str: String = row.get(14).unwrap_or_default();
            let engravings = serde_json::from_str::<Option<Vec<String>>>(engravings_str.as_str())
                .unwrap_or_default();

            let spec: Option<String> = row.get(15).unwrap_or_default();
            let ark_passive_active: Option<bool> = row.get(16).unwrap_or_default();

            let ark_passive_data_str: String = row.get(17).unwrap_or_default();
            let ark_passive_data =
                serde_json::from_str::<Option<ArkPassiveData>>(ark_passive_data_str.as_str())
                    .unwrap_or_default();

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
                    .unwrap_or(EntityType::Unknown),
                npc_id: row.get(12)?,
                character_id: row.get(13).unwrap_or_default(),
                engraving_data: engravings,
                spec,
                ark_passive_active,
                ark_passive_data,
                loadout_hash: row.get(18).unwrap_or_default(),
                combat_power: row.get(19).unwrap_or_default(),
                ..Default::default()
            })
        })
        .unwrap();

    let mut entities: HashMap<String, EncounterEntity> = HashMap::new();
    for entity in entity_iter.flatten() {
        entities.insert(entity.name.to_string(), entity);
    }

    let mut sync_stmt = conn
        .prepare_cached(
            "
    SELECT upstream_id
    FROM sync_logs
    WHERE encounter_id = ? AND failed = false;
            ",
        )
        .unwrap();

    let sync: Result<String, rusqlite::Error> = sync_stmt.query_row(params![id], |row| row.get(0));
    encounter.sync = sync.ok();

    encounter.entities = entities;

    encounter
}

#[tauri::command]
fn get_sync_candidates(app: tauri::AppHandle, force_resync: bool) -> Vec<i32> {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let query = if force_resync { "= '0'" } else { "IS NULL" };
    let mut stmt = conn
        .prepare_cached(&format!(
            "
    SELECT id
    FROM encounter_preview
    LEFT JOIN sync_logs ON encounter_id = id
    WHERE cleared = true AND boss_only_damage = 1 AND upstream_id {}
    ORDER BY fight_start;
            ",
            query
        ))
        .unwrap();
    let rows = stmt.query_map([], |row| row.get(0)).unwrap();

    let mut ids = Vec::new();
    for id_result in rows {
        ids.push(id_result.unwrap_or(0));
    }
    ids
}

#[tauri::command]
fn get_encounter_count(app: tauri::AppHandle) -> i32 {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached("SELECT COUNT(*) FROM encounter_preview")
        .unwrap();

    let count: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    count.unwrap_or(0)
}

#[tauri::command]
fn open_most_recent_encounter(app: tauri::AppHandle) {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    SELECT id
    FROM encounter_preview
    ORDER BY fight_start DESC
    LIMIT 1;
    ",
        )
        .unwrap();

    let id_result: Result<i32, rusqlite::Error> = stmt.query_row(params![], |row| row.get(0));

    if let Some(logs) = app.get_webview_window(LOGS_WINDOW_LABEL) {
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
fn toggle_encounter_favorite(app: tauri::AppHandle, id: i32) {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let mut stmt = conn
        .prepare_cached(
            "
    UPDATE encounter_preview
    SET favorite = NOT favorite
    WHERE id = ?;
    ",
        )
        .unwrap();

    stmt.execute(params![id]).unwrap();
}

#[tauri::command]
fn delete_encounter(app: tauri::AppHandle, id: String) {
    let conn = get_db_connection(&app).expect("could not get db connection");
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
fn delete_encounters(app: tauri::AppHandle, ids: Vec<i32>) {
    let conn = get_db_connection(&app).expect("could not get db connection");
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
fn toggle_meter_window(app: tauri::AppHandle) {
    let settings_manager = app.state::<SettingsManager>();
    let settings = settings_manager.read().unwrap();

    if let Some(settings) = settings {
        let label = if settings.general.mini {
            METER_MINI_WINDOW_LABEL
        } else {
            METER_WINDOW_LABEL
        };
        if let Some(meter) = app.get_webview_window(label) {
            if meter.is_visible().unwrap() {
                // workaround for tauri not handling minimized state for windows without decorations
                if meter.is_minimized().unwrap() {
                    meter.unminimize().unwrap();
                }
                meter.hide().unwrap();
            } else {
                meter.show().unwrap();
            }
        }
    }
}

#[tauri::command]
fn toggle_logs_window(app: tauri::AppHandle) {
    if let Some(logs) = app.get_webview_window(LOGS_WINDOW_LABEL) {
        if logs.is_visible().unwrap() {
            logs.hide().unwrap();
        } else {
            logs.emit("redirect-url", "logs").unwrap();
            logs.show().unwrap();
        }
    }
}

#[tauri::command]
fn open_url(app: tauri::AppHandle, url: String) {
    if let Some(logs) = app.get_webview_window(LOGS_WINDOW_LABEL) {
        logs.emit("redirect-url", url).unwrap();
    }
}

#[tauri::command]
fn save_settings(app: tauri::AppHandle, settings: Settings) {
    let path = app::path::data_dir(&app).join("settings.json");
    let contents = serde_json::to_string_pretty(&settings).unwrap();
    fs::write(path, contents).expect("could not write to settings file");
}

#[tauri::command]
fn get_settings(app: tauri::AppHandle) -> Option<Settings> {
    read_settings(&app).ok()
}

fn read_settings(app: &tauri::AppHandle) -> Result<Settings, Box<dyn std::error::Error>> {
    let path = app::path::data_dir(app).join("settings.json");
    let contents = fs::read_to_string(path)?;
    let settings = serde_json::from_str(&contents)?;
    Ok(settings)
}

#[tauri::command]
fn open_db_path(app: tauri::AppHandle) {
    let path = app::path::data_dir(&app).display().to_string();
    info!("open_db_path: {}", &path);

    if let Err(e) = app.opener().open_path(&path, None::<&str>) {
        error!("Failed to open database path: {}", e);
    }
}

#[tauri::command]
fn delete_encounters_below_min_duration(
    app: tauri::AppHandle,
    min_duration: i64,
    keep_favorites: bool,
) {
    let conn = get_db_connection(&app).expect("could not get db connection");
    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE duration < ? AND favorite = 0
            )",
            params![min_duration * 1000],
        )
        .unwrap();
    } else {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE duration < ?
            )",
            params![min_duration * 1000],
        )
        .unwrap();
    }
    conn.execute("VACUUM", params![]).unwrap();
}

#[tauri::command]
fn sync(app: tauri::AppHandle, encounter: i32, upstream: String, failed: bool) {
    let conn = get_db_connection(&app).expect("could not get db connection");

    conn.execute(
        "
        INSERT OR REPLACE INTO sync_logs (encounter_id, upstream_id, failed)
        VALUES(?, ?, ?);
        ",
        params![encounter, upstream, failed],
    )
    .unwrap();
}

#[tauri::command]
fn delete_all_uncleared_encounters(app: tauri::AppHandle, keep_favorites: bool) {
    let conn = get_db_connection(&app).expect("could not get db connection");
    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE cleared = 0 AND favorite = 0
            )",
            [],
        )
        .unwrap();
    } else {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE cleared = 0
            )",
            [],
        )
        .unwrap();
    }
    conn.execute("VACUUM", params![]).unwrap();
}

#[tauri::command]
fn delete_all_encounters(app: tauri::AppHandle, keep_favorites: bool) {
    let conn = get_db_connection(&app).expect("could not get db connection");

    if keep_favorites {
        conn.execute(
            "DELETE FROM encounter
            WHERE id IN (
                SELECT id
                FROM encounter_preview
                WHERE favorite = 0
            )",
            [],
        )
        .unwrap();
    } else {
        conn.execute("DELETE FROM encounter", []).unwrap();
    }
    conn.execute("VACUUM", []).unwrap();
}

#[tauri::command]
fn get_db_info(app: tauri::AppHandle, min_duration: i64) -> EncounterDbInfo {
    let conn = get_db_connection(&app).expect("could not get db connection");
    let encounter_count = conn
        .query_row("SELECT COUNT(*) FROM encounter_preview", [], |row| {
            row.get(0)
        })
        .unwrap();
    let encounter_filtered_count = conn
        .query_row(
            "SELECT COUNT(*) FROM encounter_preview WHERE duration >= ?",
            params![min_duration * 1000],
            |row| row.get(0),
        )
        .unwrap();

    let path = conn.path().expect("could not get db path");
    let metadata = fs::metadata(path).expect("could not get db metadata");

    let size_in_bytes = metadata.len();
    let size_in_kb = size_in_bytes as f64 / 1024.0;
    let size_in_mb = size_in_kb / 1024.0;
    let size_in_gb = size_in_mb / 1024.0;

    let size_str = if size_in_gb >= 1.0 {
        format!("{:.2} GB", size_in_gb)
    } else if size_in_mb >= 1.0 {
        format!("{:.2} MB", size_in_mb)
    } else {
        format!("{:.2} KB", size_in_kb)
    };

    EncounterDbInfo {
        size: size_str,
        total_encounters: encounter_count,
        total_encounters_filtered: encounter_filtered_count,
    }
}

#[tauri::command]
fn optimize_database(app: tauri::AppHandle) {
    let conn = get_db_connection(&app).expect("could not get db connection");
    conn.execute_batch(
        "
        INSERT INTO encounter_search(encounter_search) VALUES('optimize');
        VACUUM;
        ",
    )
    .unwrap();
    info!("optimized database");
}

#[tauri::command]
fn disable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(METER_WINDOW_LABEL) {
        clear_blur(&meter_window).ok();
    }
}

#[tauri::command]
fn enable_blur(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(METER_WINDOW_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

#[tauri::command]
fn enable_aot(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(METER_WINDOW_LABEL) {
        meter_window.set_always_on_top(true).ok();
    }
    if let Some(mini_window) = app.get_webview_window(METER_MINI_WINDOW_LABEL) {
        mini_window.set_always_on_top(true).ok();
    }
}

#[tauri::command]
fn disable_aot(app: tauri::AppHandle) {
    if let Some(meter_window) = app.get_webview_window(METER_WINDOW_LABEL) {
        meter_window.set_always_on_top(false).ok();
    }
    if let Some(mini_window) = app.get_webview_window(METER_MINI_WINDOW_LABEL) {
        mini_window.set_always_on_top(false).ok();
    }
}

#[tauri::command]
fn set_clickthrough(app: tauri::AppHandle, set: bool) {
    if let Some(meter_window) = app.get_webview_window(METER_WINDOW_LABEL) {
        meter_window.set_ignore_cursor_events(set).unwrap();
    }
}

#[tauri::command]
fn remove_driver() {
    #[cfg(target_os = "windows")]
    {
        use app::compat::Command;
        let status = Command::new("sc").args(["delete", "windivert"]).status();

        status.expect("unable to delete driver");
    }
}

#[tauri::command]
fn unload_driver() {
    #[cfg(target_os = "windows")]
    {
        use app::compat::Command;
        let status = Command::new("sc").args(["stop", "windivert"]).status();

        if status.is_ok_and(|status| status.success()) {
            info!("stopped driver");
        } else {
            warn!("could not execute command to stop driver");
        }
    }
}

#[tauri::command]
fn check_start_on_boot(auto: State<AutoLaunchManager>) -> bool {
    auto.is_enabled().unwrap_or(false)
}

#[tauri::command]
fn set_start_on_boot(auto: State<AutoLaunchManager>, set: bool) {
    let _ = match set {
        true => auto.enable(),
        false => auto.disable(),
    };
}

#[tauri::command]
fn check_loa_running() -> bool {
    let system = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().without_tasks()),
    );
    let process_name = "LOSTARK.exe";
    system
        .processes()
        .values()
        .any(|p| p.name().eq_ignore_ascii_case(process_name))
}

#[tauri::command]
fn start_loa_process(app: tauri::AppHandle) {
    if check_loa_running() {
        return info!("lost ark already running");
    }
    info!("starting lost ark process...");

    if let Err(e) = app
        .opener()
        .open_url("steam://rungameid/1599340", None::<&str>)
    {
        error!("could not open lost ark: {}", e);
    }
}

#[tauri::command]
fn write_log(message: String) {
    info!("{}", message);
}
