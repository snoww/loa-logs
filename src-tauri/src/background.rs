#![allow(dead_code)]

use anyhow::*;
use log::*;
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc},
};
use tauri::AppHandle;
use tokio::task::JoinHandle;

use crate::settings::Settings;

pub struct BackgroundWorkerArgs {
    pub app_handle: AppHandle,
    pub update_checked: Arc<AtomicBool>,
    pub port: u16,
    pub region_file_path: PathBuf,
    pub settings: Option<Settings>,
    pub version: String,
}

pub struct BackgroundWorker(Option<JoinHandle<()>>);

impl BackgroundWorker {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn start(&mut self, args: BackgroundWorkerArgs) -> Result<()> {
        let handle = tokio::task::spawn_blocking(move || Self::inner(args));

        self.0 = Some(handle);

        Ok(())
    }

    fn inner(args: BackgroundWorkerArgs) {
        let BackgroundWorkerArgs {
            app_handle,
            update_checked,
            port,
            region_file_path,
            settings,
            version,
        } = args;

        #[cfg(feature = "meter-core")]
        {
            use std::sync::atomic::Ordering;

            use crate::live;

            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            info!("listening on port: {port}");

            live::start(app_handle, port, settings).expect("unexpected error occurred in parser");
        }
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}
