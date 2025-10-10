#![allow(dead_code)]

use anyhow::*;
use log::*;
use tokio::{sync::watch, task::JoinHandle};
use std::{
    path::PathBuf,
    sync::{atomic::AtomicBool, Arc, Mutex}};
use tauri::AppHandle;

use crate::settings::Settings;

pub struct BackgroundWorkerArgs {
    pub app_handle: AppHandle,
    pub update_checked: Arc<AtomicBool>,
    pub port: u16,
    pub region_file_path: PathBuf,
    pub settings: Option<Settings>,
    pub version: String,
}

pub struct BackgroundWorker {
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    handle: Mutex<Option<JoinHandle<()>>>
}

impl BackgroundWorker {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            handle: Mutex::new(None),
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub fn start(&self, args: BackgroundWorkerArgs) -> Result<()> {
        let shutdown_rx = self.shutdown_rx.clone();
        let handle = tokio::task::spawn_blocking(move || Self::inner(args, shutdown_rx));

        *self.handle.lock().unwrap() = Some(handle);

        Ok(())
    }

    fn inner(args: BackgroundWorkerArgs, shutdown_rx: watch::Receiver<bool>) {
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

            use crate::live::{self, StartArgs};

            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            info!("listening on port: {port}");

            let args = StartArgs {
                app: app_handle,
                port,
                settings,
                shutdown_rx
            };

            live::start(args).expect("unexpected error occurred in parser");
        }
    }

    pub async fn stop(&self) -> Result<()> {
        self.shutdown_tx.send(true)?;

        let mut guard = self.handle.lock().unwrap();
        
        if let Some(handle) = guard.take().filter(|pr| !pr.is_finished()) {
            // TO-DO
            // Send signal to meter-core
            // handle.await?;
        }

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        let guard = self.handle.lock().unwrap();

        guard.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}
