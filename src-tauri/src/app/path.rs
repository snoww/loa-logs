use std::path::PathBuf;
use tauri::Manager;

/// Returns the path where we store application data
///
/// - **Linux:** Resolves to `$XDG_DATA_HOME` with fallback to `$HOME/.local/share`
/// - **Windows:** Resolves to $RESOURCE directory - near executable
pub fn data_dir(app: &tauri::AppHandle) -> PathBuf {
    #[cfg(target_os = "linux")]
    let path = app.path().app_data_dir();
    #[cfg(target_os = "windows")]
    let path = app.path().resource_dir();

    path.expect("could not get app data dir")
}

/// Returns the path where we store application logs
/// FIXME: this code duplication needs to go, but without `tauri::AppHandle`
/// there's no way to get `tauri::PathResolver` in Tauri v2
pub fn log_dir() -> PathBuf {
    #[cfg(target_os = "linux")]
    let path = {
        let context: tauri::Context<tauri::Wry> = tauri::generate_context!();
        let fallback = std::env::var_os("HOME")
            .map(PathBuf::from)
            .map(|h| h.join(".local/share"));
        let xdg_data_home = std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .or(fallback);
        xdg_data_home.map(|p| p.join(&context.config().identifier))
    };
    #[cfg(target_os = "windows")]
    let path = std::env::current_exe().unwrap().parent().map(Into::into);

    path.expect("could not get app data dir")
}
