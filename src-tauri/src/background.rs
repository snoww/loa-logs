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

            use tauri::Manager;
            use tokio::sync::watch;

            use crate::{app, context::AppContext, emitter::TauriAppEmitter, live::{self, *}, local::LocalPlayerRepository};

            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            info!("listening on port: {port}");

            let (_shutdown_tx, shutdown_rx) = watch::channel(false);
            let context = app_handle.state::<AppContext>();
            let capture = SnowPacketCapture::new(port, region_file_path);
            let encounter_service = DefaultEncounterService::new(app_handle.clone(), version);
            let emitter = TauriAppEmitter::new(app_handle.clone());
            let manager = TauriEventManager::new(app_handle.clone());
            let local_player_path = context.local_player_path.clone();
            let local_player_repository = LocalPlayerRepository::new(local_player_path).expect("could not read local players");
            let local_info = local_player_repository.read().expect("could not read local players");
            let sntp_client = SntpTimeSyncClient::new("time.cloudflare.com");

            let args = StartArgs {
                capture,
                settings,
                shutdown_rx,
                local_info,
                local_player_repository,
                // heartbeat_api: todo!(),
                encounter_service,
                emitter,
                manager,
                sntp_client
            };

            live::start(args).expect("unexpected error occurred in parser");
        }
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}
