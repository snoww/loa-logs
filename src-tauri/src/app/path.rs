use std::path::PathBuf;
#[cfg(target_os = "linux")]
use tauri::api::path::app_data_dir;
#[cfg(target_os = "windows")]
use tauri::{api::path::resource_dir, Env, Manager};

/// Returns the path where we store application data
///
/// - **Linux:** Resolves to `$XDG_DATA_HOME` with fallback to `$HOME/.local/share`
/// - **Windows:** Resolves to $RESOURCE directory - near executable
pub fn data_dir(app_handle: &tauri::AppHandle) -> PathBuf {
    #[cfg(target_os = "linux")]
    let path = app_data_dir(&app_handle.config());
    #[cfg(target_os = "windows")]
    let path = resource_dir(app_handle.package_info(), &app_handle.env());

    path.expect("could not get app data dir")
}

/// Returns the path where we store application logs
pub fn log_dir() -> PathBuf {
    let context = tauri::generate_context!();

    #[cfg(target_os = "linux")]
    let path = app_data_dir(context.config());
    #[cfg(target_os = "windows")]
    let path = resource_dir(context.package_info(), &Env::default());

    path.expect("could not get app data dir")
}
