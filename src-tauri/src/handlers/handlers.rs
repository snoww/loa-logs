use log::*;
use tauri::ipc::Invoke;
use tauri::{command, generate_handler, AppHandle, Emitter, Manager, State};
use window_vibrancy::{apply_blur, clear_blur};

use crate::app::autostart::{AutoLaunch, AutoLaunchManager};
use crate::constants::*;
use crate::database::models::{GetEncounterPreviewArgs, InsertSyncLogsArgs};
use crate::database::{Database, Repository};
use crate::ui::AppHandleExtensions;
use crate::handlers::error::AppError;
use crate::models::*;
use crate::settings::{Settings, SettingsManager};
use crate::shell::ShellManager;

pub fn generate_handlers() -> Box<dyn Fn(Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
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
}

#[command]
pub fn load_encounters_preview(
    repository: State<Repository>,
    page: i32,
    page_size: i32,
    search: String,
    filter: SearchFilter,
) -> EncountersOverview {

    let args = GetEncounterPreviewArgs {
        page,
        page_size,
        search,
        filter
    };
    
    let encounter = repository.get_encounter_preview(args).expect("could not get encounter preview");

    encounter
}

#[command(async)]
pub fn load_encounter(repository: State<Repository>, id: String) -> Encounter {

    let encounter = repository.get_encounter(id).expect("could not get encounter");

    encounter
}

#[command]
pub fn get_sync_candidates(repository: State<Repository>, force_resync: bool) -> Vec<i32> {

    let ids = repository.get_sync_candidates(force_resync).expect("could not get sync candidates");

    ids
}

#[command]
pub fn get_encounter_count(repository: State<Repository>) -> i32 {

    let count = repository.get_encounter_count().expect("could not get encounter count");

    count
}

#[command]
pub fn open_most_recent_encounter(app_handle: AppHandle, repository: State<Repository>) {
    
    let id = repository.get_last_encounter_id().expect("could not get last encounter");

    if let Some(logs) = app_handle.get_logs_window() {
        match id {
            Some(id) => {
                logs.emit("show-latest-encounter", id.to_string()).unwrap();
            }
            None => {
                logs.emit("redirect-url", "logs").unwrap();
            }
        }
    }
}

#[command]
pub fn toggle_encounter_favorite(repository: State<Repository>, id: i32) {

    repository.toggle_encounter_favorite(id).expect("could not update encounter");
}

#[command]
pub fn delete_encounter(repository: State<Repository>, id: String) {

    repository.delete_encounter(id).expect("could not delete encounter");
}

#[command]
pub fn delete_encounters(repository: State<Repository>, ids: Vec<i32>) {

    repository.delete_encounters(ids).expect("");
}

#[command]
pub fn toggle_meter_window(app: AppHandle, settings_manager: State<SettingsManager>) {
    let settings = settings_manager.read().ok().flatten().expect("could not read settings");

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

#[command]
pub fn toggle_logs_window(app_handle: AppHandle) {
    if let Some(logs) = app_handle.get_logs_window() {
        if logs.is_visible().unwrap() {
            logs.hide().unwrap();
        } else {
            logs.emit("redirect-url", "logs").unwrap();
            logs.show().unwrap();
        }
    }
}

#[command]
pub fn open_url(app_handle: AppHandle, url: String) {
    if let Some(logs) = app_handle.get_logs_window() {
        logs.emit("redirect-url", url).unwrap();
    }
}

#[command]
pub fn save_settings(settings_manager: State<SettingsManager>, settings: Settings) {
    settings_manager.save(&settings).expect("could not write to settings file");
}

#[command]
pub fn get_settings(settings_manager: State<SettingsManager>) -> Settings {
    settings_manager.read().ok().flatten().expect("could not read settings file")
}

#[command]
pub fn open_db_path(shell_manager: State<ShellManager>) {
    shell_manager.open_db_path();
}

#[command]
pub fn delete_encounters_below_min_duration(
    repository: State<Repository>,
    min_duration: i64,
    keep_favorites: bool,
) {

    repository.delete_encounters_below_min_duration(min_duration, keep_favorites).expect("could not delete encounters");
}

#[command]
pub fn sync(repository: State<Repository>, encounter: i32, upstream: String, failed: bool) {
    
    let args = InsertSyncLogsArgs {
        encounter,
        upstream,
        failed
    };

    repository.insert_sync_logs(args).expect("could not insert sync logs");
}

#[command]
pub fn delete_all_uncleared_encounters(repository: State<Repository>, keep_favorites: bool) {
    
    repository.delete_all_uncleared_encounters(keep_favorites).expect("could not delete encounters");
}

#[command]
pub fn delete_all_encounters(repository: State<Repository>, keep_favorites: bool) {

    repository.delete_all_encounters(keep_favorites).expect("could not delete encounters");
}

#[command]
pub fn get_db_info(
    database: State<Database>,
    repository: State<Repository>,
    min_duration: i64) -> EncounterDbInfo {
    let (total_encounters, total_encounters_filtered) = repository.get_db_stats(min_duration)
        .expect("could not get db stats");

    let size = database.get_metadata().expect("could not get db metadata");

    EncounterDbInfo {
        size,
        total_encounters,
        total_encounters_filtered,
    }
}

#[command]
pub fn optimize_database(repository: State<Repository>) {

    repository.optimize().expect("could not optimize database");
    info!("optimized database");
}

#[command]
pub fn disable_blur(app_handle: AppHandle) -> Result<(), AppError> {
    if let Some(meter_window) = app_handle.get_meter_window() {
        clear_blur(&*meter_window).map_err(|err| AppError::Vibrancy(err))?;
    }

    Ok(())
}

#[command]
pub fn enable_blur(app_handle: AppHandle) -> Result<(), AppError> {
    if let Some(meter_window) = app_handle.get_meter_window() {
        apply_blur(&*meter_window, Some(DEFAULT_BLUR)).map_err(|err| AppError::Vibrancy(err))?;
    }

    Ok(())
}

#[command]
pub fn enable_aot(app_handle: AppHandle) -> Result<(), AppError> {
    
    if let Some(meter_window) = app_handle.get_meter_window() {
        meter_window.set_always_on_top(true).map_err(|err| AppError::Ui(err))?;
    }

    if let Some(mini_window) = app_handle.get_mini_window() {
        mini_window.set_always_on_top(true).map_err(|err| AppError::Ui(err))?;
    }

    Ok(())
}

#[command]
pub fn disable_aot(app_handle: AppHandle) -> Result<(), AppError> {
    if let Some(meter_window) = app_handle.get_meter_window() {
        meter_window.set_always_on_top(false).map_err(|err| AppError::Ui(err))?;
    }

    if let Some(mini_window) = app_handle.get_mini_window() {
        mini_window.set_always_on_top(false).map_err(|err| AppError::Ui(err))?;
    }

    Ok(())
}

#[command]
pub fn set_clickthrough(app_handle: AppHandle, set: bool) -> Result<(), AppError> {
    if let Some(meter_window) = app_handle.get_meter_window() {
        meter_window.set_ignore_cursor_events(set).map_err(|err| AppError::Ui(err))?;
    }

    Ok(())
}

#[command]
pub async fn remove_driver(shell_manager: State<'_, ShellManager>) -> Result<(), AppError> {
    shell_manager.remove_driver().await;
    Ok(())
}

#[command]
pub async fn unload_driver(shell_manager: State<'_, ShellManager>)  -> Result<(), AppError> {
    shell_manager.unload_driver().await;
    Ok(())
}

#[command]
pub fn check_start_on_boot(auto: State<AutoLaunchManager>) -> bool {
    auto.is_enabled().unwrap_or(false)
}

#[command]
pub fn set_start_on_boot(auto: State<AutoLaunchManager>, set: bool) {
    let _ = match set {
        true => auto.enable(),
        false => auto.disable(),
    };
}

#[command]
pub fn check_loa_running(shell_manager: State<ShellManager>) -> bool {
    shell_manager.check_loa_running()
}

#[command]
pub fn start_loa_process(shell_manager: State<ShellManager>) {
    shell_manager.start_loa_process();
}

#[command]
pub fn write_log(message: String) {
    info!("{}", message);
}
