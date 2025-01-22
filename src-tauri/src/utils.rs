use std::process::Command;

use log::{error, info, warn};
use sysinfo::System;

use crate::app;

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

pub fn remove_driver() {
    Command::new("sc").args(["delete", "windivert"]).output().expect("unable to delete driver");
}

pub fn start_loa_process() {
    if !check_loa_running() {
        info!("starting lost ark process...");
        Command::new("cmd")
            .args(["/C", "start", "steam://rungameid/1599340"])
            .spawn()
            .map_err(|e| error!("could not open lost ark: {}", e))
            .ok();
    } else {
        info!("lost ark already running")
    }
}

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

pub fn set_panic_hook() {
    std::panic::set_hook(Box::new(|info| {
        error!("Panicked: {:?}", info);

        app::get_logger().unwrap().flush();
    }));
}