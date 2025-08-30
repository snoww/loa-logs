use std::time::{Duration, Instant};

use log::*;
use reqwest::Client;
use serde_json::json;
pub const API_URL: &str = "https://api.snow.xyz";

pub struct HeartbeatSendArgs {
    pub client_id: String,
    pub version: String,
    pub region: Option<String>
}

pub trait HeartbeatApi {
    fn can_send(&self) -> bool;
    fn send(&mut self, args: HeartbeatSendArgs);
}

pub struct SnowHeartbeatApi {
    client: Client,
    last_heartbeat: Instant,
    heartbeat_duration: Duration
}

impl HeartbeatApi for SnowHeartbeatApi {
    fn can_send(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.heartbeat_duration
    }

    fn send(&mut self, args: HeartbeatSendArgs) {

        let HeartbeatSendArgs {
            client_id,
            region,
            version
        } = args;

        let client = self.client.clone();
        let region = match region {
            Some(ref region) => region.clone(),
            None => return,
        };

        tokio::task::spawn(async move {
            let request_body = json!({
                "id": client_id,
                "version": version,
                "region": region,
            });

            match client
                .post(format!("{API_URL}/analytics/heartbeat"))
                .json(&request_body)
                .send()
                .await
            {
                Ok(_) => {
                    info!("sent heartbeat");
                }
                Err(e) => {
                    warn!("failed to send heartbeat: {:?}", e);
                }
            }
        });
    
        self.last_heartbeat = Instant::now();
    }
}

impl SnowHeartbeatApi {
    pub fn new() -> Self {
        let client = Client::new();
        let last_heartbeat = Instant::now();
        let heartbeat_duration = Duration::from_secs(60 * 15);
        Self {
            client,
            last_heartbeat,
            heartbeat_duration
        }
    }
}

pub struct FakeHeartbeatApi {
    last_heartbeat: Instant,
    heartbeat_duration: Duration
}

impl HeartbeatApi for FakeHeartbeatApi {
    fn can_send(&self) -> bool {
        self.last_heartbeat.elapsed() >= self.heartbeat_duration
    }

    fn send(&mut self, args: HeartbeatSendArgs) {
        info!("heartbeat client_id: {} region: {:?} version: {}", args.client_id, args.region, args.version);
        self.last_heartbeat = Instant::now();
    }
}

impl FakeHeartbeatApi {
    pub fn new() -> Self {
        let last_heartbeat = Instant::now();
        let heartbeat_duration = Duration::from_secs(60 * 15);

        Self {
            last_heartbeat,
            heartbeat_duration
        }
    }
}
