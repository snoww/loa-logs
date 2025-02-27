use std::{fs::{self, File}, io::Write, path::PathBuf, process::Command};

use log::{info, warn};
use rusqlite::params;
use sysinfo::System;
use tauri::Manager;
use window_vibrancy::{apply_blur, clear_blur};

use crate::{constants::{DATABASE_FILE_NAME, LOGS_WINDOW_LABEL, METER_WINDOW_LABEL, SETTINGS_FILE_NAME}, database::get_db_connection, settings::read_settings, utils, EncounterDbInfo, Settings};


#[tauri::command]
pub fn get_db_info(window: tauri::Window, min_duration: i64) -> EncounterDbInfo {
    let mut path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
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

    path.push(DATABASE_FILE_NAME);
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
pub fn optimize_database(window: tauri::Window) {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    let conn = get_db_connection(&path).expect("could not get db connection");
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
pub fn disable_blur(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        clear_blur(&meter_window).ok();
    }
}

#[tauri::command]
pub fn enable_blur(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        apply_blur(&meter_window, Some((10, 10, 10, 50))).ok();
    }
}

#[tauri::command]
pub fn enable_aot(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        meter_window.set_always_on_top(true).ok();
    }
}

#[tauri::command]
pub fn disable_aot(window: tauri::Window) {
    if let Some(meter_window) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        meter_window.set_always_on_top(false).ok();
    }
}

#[tauri::command]
pub fn set_clickthrough(window: tauri::Window, set: bool) {
    if let Some(meter_window) = window.app_handle().get_window(METER_WINDOW_LABEL) {
        meter_window.set_ignore_cursor_events(set).unwrap();
    }
}

#[tauri::command]
pub fn remove_driver() {
    Command::new("sc").args(["delete", "windivert"]).output().expect("unable to delete driver");
}

#[tauri::command]
pub fn unload_driver() {
    let output = Command::new("sc").args(["stop", "windivert"]).output();

    match output {
        Ok(output) => {
            if output.status.success() {
                info!("stopped driver");
            }
        }
        Err(_) => {
            warn!("could not execute command to stop driver");
        }
    }
}

#[tauri::command]
pub fn check_start_on_boot() -> bool {
    // Run the `schtasks` command to query the task
    let output = Command::new("schtasks")
        .args(["/query", "/tn", "LOA_Logs_Auto_Start"])
        .output();

    match output {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

#[tauri::command]
pub fn set_start_on_boot(set: bool) {
    let app_path = match std::env::current_exe() {
        Ok(path) => path.to_string_lossy().to_string(),
        Err(e) => {
            warn!("could not get current exe path: {}", e);
            return;
        }
    };

    let task_name = "LOA_Logs_Auto_Start";

    if set {
        Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .output().ok();
        
        let output = Command::new("schtasks")
            .args([
                "/create", "/tn", task_name, "/tr", &format!("\"{}\"", &app_path), "/sc", "onlogon", "/rl", "highest",
            ])
            .output();

        match output {
            Ok(_) => {
                info!("enabled start on boot");
            }
            Err(e) => {
                warn!("error enabling start on boot: {}", e);
            }
        }
    } else {
        let output = Command::new("schtasks")
            .args(["/delete", "/tn", task_name, "/f"])
            .output();

        match output {
            Ok(_) => {
                info!("disabled start on boot");
            }
            Err(e) => {
                warn!("error disabling start on boot: {}", e);
            }
        }
    }
}

#[tauri::command]
pub fn check_loa_running() -> bool {
    let system = System::new_all();
    let process_name = "lostark.exe";

    // Iterate through all running processes
    for process in system.processes().values() {
        if process.name().to_string_lossy().to_ascii_lowercase() == process_name {
            return true;
        }
    }
    false
}

#[tauri::command]
pub fn start_loa_process() {
   utils::start_loa_process();
}

#[tauri::command]
pub fn write_log(message: String) {
    info!("{}", message);
}

#[tauri::command]
pub fn get_settings(window: tauri::Window) -> Option<Settings> {
    let path = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    read_settings(&path).ok()
}

#[tauri::command]
pub fn open_folder(path: String) {
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
pub fn open_db_path(window: tauri::Window) {
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
pub fn open_url(window: tauri::Window, url: String) {
    if let Some(logs) = window.app_handle().get_window(LOGS_WINDOW_LABEL) {
        logs.emit("redirect-url", url).unwrap();
    }
}

#[tauri::command]
pub fn save_settings(window: tauri::Window, settings: Settings) {
    let mut path: PathBuf = window
        .app_handle()
        .path_resolver()
        .resource_dir()
        .expect("could not get resource dir");
    path.push(SETTINGS_FILE_NAME);
    let mut file = File::create(path).expect("could not create settings file");
    file.write_all(serde_json::to_string_pretty(&settings).unwrap().as_bytes())
        .expect("could not write to settings file");
}