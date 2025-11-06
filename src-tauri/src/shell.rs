use log::*;
use sysinfo::{ProcessRefreshKind, RefreshKind, System};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::ShellExt;

use crate::{
    constants::{GAME_EXE_NAME, STEAM_GAME_URL},
    context::AppContext,
};

#[derive(Debug)]
pub struct ShellManager(AppHandle, AppContext);

impl ShellManager {
    pub fn new(shell: AppHandle, context: AppContext) -> Self {
        Self(shell, context)
    }

    pub fn open_db_path(&self) {
        let path = &self.1.current_dir;
        info!("open_db_path: {}", path.display());

        if let Err(err) = self
            .0
            .opener()
            .open_path(path.to_str().unwrap(), None::<String>)
        {
            error!("Failed to open database path: {err}");
        }
    }

    pub fn start_loa_process(&self) {
        if self.check_loa_running() {
            return info!("lost ark already running");
        }

        info!("starting lost ark process...");

        if let Err(err) = self.0.opener().open_path(STEAM_GAME_URL, None::<String>) {
            error!("could not open lost ark: {err}");
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

            match command.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if output.status.success() {
                        info!("Driver removed successfully");
                    } else {
                        warn!(
                            "Failed to remove driver. Exit code {} - stdout: {}",
                            output.status.code().unwrap_or(-1),
                            stdout
                        );
                    }
                }
                Err(err) => {
                    warn!("Failed to execute remove driver command: {err}");
                }
            }
        }
    }

    pub async fn unload_driver(&self) {
        #[cfg(target_os = "windows")]
        {
            let command = self.0.shell().command("sc").args(["stop", "windivert"]);

            match command.output().await {
                Ok(output) => {
                    let stdout = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .map(|line| line.trim())
                        .filter(|line| !line.is_empty())
                        .collect::<Vec<_>>()
                        .join(" ");

                    if output.status.success() {
                        info!("Driver stopped successfully");
                    } else {
                        let code = output.status.code().unwrap_or(-1);
                        // ignore error if driver is not running
                        if code == 1062 {
                            return;
                        }
                        warn!("Failed to stop driver. Exit code: {code} - stdout: {stdout}");
                    }
                }
                Err(err) => {
                    warn!("Failed to execute driver stop command: {err}");
                }
            }
        }
    }
}
