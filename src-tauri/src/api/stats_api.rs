use crate::{api::{GetCharacterInfoArgs, SendRaidAnalyticsArgs}, models::*};
use hashbrown::HashMap;
use log::*;
use reqwest::Client;
use std::time::Duration;
use tokio::time::sleep;

pub const RETRIES: u8 = 3;
pub const RETRY_DELAY_MS: u64 = 250;

#[derive(Clone)]
pub struct StatsApi {
    base_url: String,
    client_id: String,
    client: Client,
    version: String,
}

impl StatsApi {
    pub fn new(base_url: String, client_id: String, version: String) -> Self {
        Self {
            base_url,
            client_id,
            version,
            client: Client::new(),
        }
    }

    pub async fn send_raid_analytics<'a>(&self, args: SendRaidAnalyticsArgs<'a>)  {
        let url = "https://recap.ags.lol/api/report";

        let _ = self
            .client
            .post(url)
            .json(&args)
            .send()
            .await;
    }

    pub async fn get_character_info<'a>(&self, mut args: GetCharacterInfoArgs<'a>) -> Option<HashMap<String, InspectInfo>> {
        
        args.client_id = &self.client_id;
        args.version = &self.version;
        let url = format!("{}/inspect", self.base_url);
        let body = serde_json::to_value(&args).unwrap();

        for attempt in 1..=RETRIES {
            let response = self
                .client
                .post(&url)
                .json(&body)
                .send()
                .await;

            match response {
                Ok(res) => match res.json::<HashMap<String, InspectInfo>>().await {
                    Ok(data) => {
                        info!("received player stats");
                        return Some(data);
                    }
                    Err(e) => {
                        warn!(
                            "failed to parse player stats (attempt {}/{}): {:?}",
                            attempt, RETRIES, e
                        );
                    }
                },
                Err(e) => {
                    warn!(
                        "failed to get inspect data (attempt {}/{}): {:?}",
                        attempt, RETRIES, e
                    );
                }
            }

            if attempt < RETRIES {
                let backoff = RETRY_DELAY_MS.saturating_mul(1 << (attempt - 1));
                sleep(Duration::from_millis(backoff)).await;
            }
        }

        None
    }
}
