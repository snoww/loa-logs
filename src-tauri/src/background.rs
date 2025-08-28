use std::{path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self, JoinHandle}};
use crate::live::{self, heartbeat::DefaultHeartbeatApi, region::DefaultRegionAccessor, DefaultDamageEncryptionHandler, WindivertPacketCapture};
use log::*;
use tauri::AppHandle;

use crate::parser::models::Settings;

pub struct BackgroundWorkerArgs {
    pub app: AppHandle,
    pub port: u16,
    pub update_checked: Arc<AtomicBool>,
    pub region_file_path: PathBuf,
    pub settings: Settings,
    pub version: String
}

pub struct BackgroundWorker(Option<JoinHandle<Result<(), ()>>>);

impl BackgroundWorker {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn start(&mut self, args: BackgroundWorkerArgs) {
        let BackgroundWorkerArgs {
            app,
            port,
            update_checked,
            region_file_path,
            settings,
            version
        } = args;

        let handle: thread::JoinHandle<Result<(), ()>> = thread::spawn(move || {
            
            // only start listening when there's no update, otherwise unable to remove driver
            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            info!("listening on port: {}", port);
            
            let heartbeat_api = DefaultHeartbeatApi::new();
            let region_acessor = DefaultRegionAccessor::new(region_file_path.clone().into());
            let packet_capture = WindivertPacketCapture::new(region_file_path.display().to_string());
            let damage_handler = DefaultDamageEncryptionHandler::new();
            live::start(
                heartbeat_api,
                region_acessor,
                packet_capture,
                damage_handler,
                app,
                port,
                version,
                settings).map_err(|e| {
                error!("unexpected error occurred in parser: {}", e);
            })
        });

        self.0 = Some(handle);
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}