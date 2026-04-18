use log::*;
use sysinfo::{Pid, Process, ProcessRefreshKind, RefreshKind, System};
use tauri::{AppHandle, Manager};
use tauri_plugin_opener::OpenerExt;
use tauri_plugin_shell::ShellExt;

use std::path::{Path, PathBuf};

use crate::constants::{GAME_EXE_NAME, NINEVEH_COMPAT_EXE_NAME, NINEVEH_EXE_NAME, STEAM_GAME_URL};
use crate::context::AppContext;

/// Returns true if the process's executable lives directly inside `app_dir`.
fn process_in_dir(process: &Process, app_dir: &Path) -> bool {
    process
        .exe()
        .and_then(|exe| exe.parent())
        .is_some_and(|parent| parent == app_dir)
}

/// Returns true if the process name matches one of the names we use for the nineveh binary.
fn has_nineveh_name(process: &Process) -> bool {
    let name = process.name();
    name.eq_ignore_ascii_case(NINEVEH_EXE_NAME)
        || name.eq_ignore_ascii_case(NINEVEH_COMPAT_EXE_NAME)
}

/// Snapshot of the running processes, refreshed only for names/paths.
fn process_snapshot() -> System {
    System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::nothing().without_tasks()),
    )
}

/// Find a running nineveh process belonging to this installation — i.e. a process named
/// `nineveh.exe` or `LOSTARK.exe` whose executable lives inside `app_dir` (and isn't this
/// meter process itself). Returns `None` if no such process is found.
pub fn find_nineveh_pid(app_dir: &Path) -> Option<Pid> {
    let system = process_snapshot();
    let self_pid = std::process::id();
    system
        .processes()
        .iter()
        .find(|(pid, p)| {
            pid.as_u32() != self_pid && has_nineveh_name(p) && process_in_dir(p, app_dir)
        })
        .map(|(pid, _)| *pid)
}

#[derive(Debug)]
pub struct ShellManager(AppHandle, PathBuf);

impl ShellManager {
    pub fn new(shell: AppHandle, database_path: PathBuf) -> Self {
        Self(shell, database_path)
    }

    fn app_dir(&self) -> PathBuf {
        self.0.state::<AppContext>().current_dir.clone()
    }

    pub fn open_db_path(&self) {
        let path = &self.1.parent().unwrap();
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
        // The game is any LOSTARK.exe whose executable is NOT inside meter install dir
        // the lostark.exe inside install dir is renamed nineveh binary when exitlag_compat is enabled
        let app_dir = self.app_dir();
        let system = process_snapshot();
        system
            .processes()
            .values()
            .any(|p| p.name().eq_ignore_ascii_case(GAME_EXE_NAME) && !process_in_dir(p, &app_dir))
    }

    pub fn check_nineveh_running(&self) -> bool {
        find_nineveh_pid(&self.app_dir()).is_some()
    }

    pub fn kill_nineveh_process(&self) {
        let app_dir = self.app_dir();
        let system = process_snapshot();
        let self_pid = std::process::id();
        for (pid, process) in system.processes() {
            if pid.as_u32() == self_pid {
                continue;
            }
            if has_nineveh_name(process) && process_in_dir(process, &app_dir) {
                let name = process.name().to_string_lossy();
                if process.kill() {
                    info!("stopped {name} (pid: {pid})");
                } else {
                    warn!("failed to stop {name} (pid: {pid})");
                }
            }
        }
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
                        if matches!(code, 1060 | 1062) {
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
