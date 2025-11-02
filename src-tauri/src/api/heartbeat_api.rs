use std::time::{Duration, Instant};
use log::*;
use reqwest::Client;

use crate::api::SendHeartbeatArgs;

pub struct HeartBeatApi {
    base_url: String,
    client_id: String,
    client: Client,
    version: String,
    last_heartbeat: Instant,
    heartbeat_duration: Duration
}

impl HeartBeatApi {
    pub fn new(base_url: String, client_id: String, version: String) -> Self {
        Self {
            base_url,
            client_id,
            version,
            client: Client::new(),
            last_heartbeat: Instant::now(),
            heartbeat_duration: Duration::from_secs(60 * 15)
        }
    }

    pub fn heartbeat(&mut self, region: &str) {
        if self.last_heartbeat.elapsed() >= self.heartbeat_duration {

            let url = format!("{}/analytics/heartbeat", self.base_url);
            let client = self.client.clone();

            let args = SendHeartbeatArgs {
                id: &self.client_id,
                version: &self.version,
                region
            };
            let body = serde_json::to_value(args).unwrap();

            tokio::task::spawn(async move {
                

                match client
                    .post(url)
                    .json(&body)
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
}