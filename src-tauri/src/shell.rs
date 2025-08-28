use log::*;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::api::process::Command;
use tauri::ShellScope;

use crate::{constants::{GAME_EXE_NAME, STEAM_GAME_URL}, context::AppContext};

pub struct ShellManager(ShellScope, AppContext);

impl ShellManager {
    pub fn new(scope: ShellScope, context: AppContext) -> Self {
        Self(scope, context)
    }

    pub fn open_db_path(&self) {
        
        let path = &self.1.database_path;
        info!("open_db_path: {}", path.display());

        if let Err(e) = self.0.open(path.to_str().unwrap(), None) {
            error!("Failed to open database path: {}", e);
        }
    }

    pub fn start_loa_process(&self) {
        if self.check_loa_running() {
            return info!("lost ark already running");
        }

        info!("starting lost ark process...");

        if let Err(e) = self.0.open(STEAM_GAME_URL, None) {
            error!("could not open lost ark: {}", e);
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

    pub fn remove_driver(&self) {
        #[cfg(target_os = "windows")]
        {
            let command = Command::new("sc").args(["delete", "windivert"]);

            command.output().expect("unable to delete driver");
        }
    }

    pub fn unload_driver(&self) {
        #[cfg(target_os = "windows")]
        {  
            let command = Command::new("sc").args(["stop", "windivert"]);

            if command.output().is_ok_and(|output| output.status.success()) {
                info!("stopped driver");
            } else {
                warn!("could not execute command to stop driver");
            }
        }
    }
}