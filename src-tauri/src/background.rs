use std::{path::PathBuf, sync::{atomic::{AtomicBool, Ordering}, Arc}, thread::{self, JoinHandle}};
use log::*;
use anyhow::Result;
use tauri::AppHandle;
use tokio::runtime::Runtime;

use crate::live::models::Settings;

pub struct BackgroundWorkerArgs {
    pub app: AppHandle,
    pub port: u16,
    pub update_checked: Arc<AtomicBool>,
    pub region_file_path: PathBuf,
    pub settings: Settings,
    pub version: String
}

pub struct BackgroundWorker(Option<JoinHandle<()>>);

impl BackgroundWorker {
    pub fn new() -> Self {
        Self(None)
    }

    pub fn start(&mut self, args: BackgroundWorkerArgs) -> Result<()> {
        let BackgroundWorkerArgs {
            app,
            port,
            update_checked,
            region_file_path,
            settings,
            version
        } = args;

        let builder = thread::Builder::new().name("background-worker".to_string());

        let handle = builder.spawn(move || {
            // only start listening when there's no update, otherwise unable to remove driver
            while !update_checked.load(Ordering::Relaxed) {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            let rt = Runtime::new().expect("Failed to create Tokio runtime");
            
            rt.block_on(async {

                info!("listening on port: {}", port);
                
                #[cfg(feature = "meter-core")]
                {
                    use crate::{abstractions::{DefaultRegionAccessor, SnowHeartbeatApi}, live};

                    let heartbeat_api = SnowHeartbeatApi::new();
                    let region_acessor = DefaultRegionAccessor::new(region_file_path.clone().into());
                    let packet_source = WindivertPacketCapture::new(region_file_path.display().to_string());
                    let damage_handler = DefaultDamageEncryptionHandler::new();
                    live::start(
                        heartbeat_api,
                        region_acessor,
                        packet_source,
                        damage_handler,
                        app,
                        port,
                        version,
                        settings).unwrap();
                }

                #[cfg(feature = "meter-core-fake")]
                {
                    use crate::{abstractions::{DefaultDamageEncryptionHandler, FakeHeartbeatApi, FakePacketSource, FakeRegionAccessor}, live};

                    let heartbeat_api = FakeHeartbeatApi::new();
                    let region_acessor = FakeRegionAccessor::new("EUC".into());
                    let packet_source = FakePacketSource::new();
                    let damage_handler = DefaultDamageEncryptionHandler::new();
                    live::start(
                        heartbeat_api,
                        region_acessor,
                        packet_source,
                        damage_handler,
                        app,
                        port,
                        version,
                        settings).unwrap();
                }
            });
        })?;

        self.0 = Some(handle);

        Ok(())
    }

    pub fn is_running(&self) -> bool {
        self.0.as_ref().is_some_and(|handle| !handle.is_finished())
    }
}