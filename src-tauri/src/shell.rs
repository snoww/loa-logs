use log::*;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::ShellExt;

use crate::{constants::{GAME_EXE_NAME, STEAM_GAME_URL}, context::AppContext};

pub struct ShellManager(AppHandle, AppContext);

impl ShellManager {
    pub fn new(shell: AppHandle, context: AppContext) -> Self {
        Self(shell, context)
    }

    pub fn open_db_path(&self) {
        
        let path = &self.1.database_path;
        info!("open_db_path: {}", path.display());

        if let Err(err) = self.0.opener().open_path(path.to_str().unwrap(), None::<String>) {
            error!("Failed to open database path: {}", err);
        }
    }

    pub fn start_loa_process(&self) {
        if self.check_loa_running() {
            return info!("lost ark already running");
        }

        info!("starting lost ark process...");

        if let Err(err) = self.0.opener().open_path(STEAM_GAME_URL, None::<String>) {
            error!("could not open lost ark: {}", err);
        }
    }

    pub fn check_loa_running(&self) -> bool {
        let system = System::new_with_specifics(
            RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().without_tasks()),
        );
        
        let process_name = GAME_EXE_NAME;

        system
            .processes()
            .values()
            .any(|p| p.name().eq_ignore_ascii_case(process_name))
    }

    pub async fn remove_driver(&self) {
        #[cfg(target_os = "windows")]
        {
            use tauri_plugin_shell::ShellExt;

            let command = self.0.shell().command("sc").args(["delete", "windivert"]);

            command.output().await.expect("unable to delete driver");
        }
    }

    pub async fn unload_driver(&self) {
        #[cfg(target_os = "windows")]
        {  
            let command = self.0.shell().command("sc").args(["stop", "windivert"]);
            let result = command.output().await;

            if result.is_ok_and(|output| output.status.success()) {
                info!("stopped driver");
            } else {
                warn!("could not execute command to stop driver");
            }
        }
    }
}