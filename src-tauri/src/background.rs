#![allow(dead_code)]

use anyhow::*;
use log::*;
use std::sync::{Arc, Mutex, atomic::AtomicBool};
use tauri::AppHandle;
use tokio::{sync::watch, task::JoinHandle};

use crate::settings::Settings;

pub struct BackgroundWorkerArgs {
    pub update_checked: Arc<AtomicBool>,
    pub port: u16,
    pub settings: Option<Settings>,
    pub version: String,
}

pub struct BackgroundWorker {
    app_handle: AppHandle,
    shutdown_tx: watch::Sender<bool>,
    shutdown_rx: watch::Receiver<bool>,
    handle: Mutex<Option<JoinHandle<()>>>,
}

impl BackgroundWorker {
    pub fn new(app_handle: AppHandle) -> Self {
        let (shutdown_tx, shutdown_rx) = watch::channel(false);

        Self {
            app_handle,
            handle: Mutex::new(None),
            shutdown_tx,
            shutdown_rx,
        }
    }

    pub fn start(&mut self, args: BackgroundWorkerArgs) -> Result<()> {
        let app_handle = self.app_handle.clone();
        let shutdown_rx = self.shutdown_rx.clone();
        let handle =
            tokio::task::spawn_blocking(move || Self::inner(app_handle, args, shutdown_rx));

        *self.handle.lock().unwrap() = Some(handle);

        Ok(())
    }

    fn inner(
        app_handle: AppHandle,
        args: BackgroundWorkerArgs,
        shutdown_rx: watch::Receiver<bool>,
    ) {
        let BackgroundWorkerArgs {
            update_checked,
            port,
            settings,
            version,
        } = args;

        #[cfg(feature = "meter-core")]
        {
            use std::sync::atomic::Ordering;

            use tauri::Manager;

            use crate::{
                api::*,
                context::AppContext,
                live::{self, broadcast::DefaultBroadcastManager, capture::SnowPacketCapture, region::DefaultRegionAccessor, StartArgs},
                local::LocalPlayerRepository,
            };

            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            info!("listening on port: {port}");

            let context = app_handle.state::<AppContext>();

            // usage pwsh: $env:STATS_API = "http://localhost:5180"; cargo tauri dev
            let base_url = option_env!("STATS_API")
                .unwrap_or("https://api.snow.xyz")
                .to_owned();
            let local_player_path = context.local_player_path.clone();
            let local_player_repository = LocalPlayerRepository::new(local_player_path)
                .expect("could not read local players");
            let local_info = local_player_repository
                .read()
                .expect("could not read local players");
            let heartbeat_api = HeartBeatApi::new(
                base_url.clone(),
                local_info.client_id.clone(),
                version.clone(),
            );
            app_handle.manage(StatsApi::new(
                base_url,
                local_info.client_id.clone(),
                version.clone(),
            ));
            let region_accessor = DefaultRegionAccessor::new(context.region_file_path.clone());
            let region_file_path = context.region_file_path.display().to_string();
            let capture = SnowPacketCapture::new(region_file_path.clone());
            let broadcast = DefaultBroadcastManager::new(version.clone());

            let args = StartArgs {
                capture,
                broadcast,
                region_accessor,
                app: app_handle,
                port,
                settings,
                shutdown_rx,
                local_info,
                local_player_repository,
                heartbeat_api
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
