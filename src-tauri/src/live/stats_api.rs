use crate::live::debug_print;
use crate::live::utils::{boss_to_raid_map, is_valid_player};
use crate::models::*;
use hashbrown::HashMap;
use log::warn;
use reqwest::Client;
use serde_json::json;
use std::time::Duration;
use tauri::{AppHandle, Manager};
use tokio::time::sleep;

// pub const API_URL: &str = "http://localhost:5180";
pub const API_URL: &str = "https://api.snow.xyz";

pub const RETRIES: u8 = 3;
pub const RETRY_DELAY_MS: u64 = 250;

#[derive(Clone)]
pub struct StatsApi {
    pub client_id: String,
    client: Client,
    app: AppHandle,
}

impl StatsApi {
    pub fn new(app: AppHandle) -> Self {
        Self {
            client_id: String::new(),
            app,
            client: Client::new(),
        }
    }

    pub async fn get_character_info(
        &self,
        encounter: &Encounter,
    ) -> Option<HashMap<String, InspectInfo>> {
        if encounter.region.is_none() {
            warn!("region is not set");
            return None;
        }

        let raid_name = encounter
            .entities
            .get(&encounter.current_boss_name)
            .and_then(|boss| boss_to_raid_map(&encounter.current_boss_name, boss.max_hp));

        let players: Vec<String> = encounter
            .entities
            .iter()
            .filter_map(|(_, e)| {
                if is_valid_player(e) {
                    Some(e.name.clone())
                } else {
                    None
                }
            })
            .collect();

        if players.len() > 16 {
            return None;
        }

        let request_body = json!({
            "clientId": self.client_id,
            "version": self.app.package_info().version.to_string(),
            "region": encounter.region.as_ref().unwrap(),
            "raidName": raid_name.unwrap_or_default(),
            "boss": encounter.current_boss_name,
            "characters": players,
            "difficulty": encounter.difficulty,
            "cleared": encounter.cleared,
        });

        for attempt in 1..=RETRIES {
            let response = self
                .client
                .post(format!("{API_URL}/inspect"))
                .json(&request_body)
                .send()
                .await;

            match response {
                Ok(res) => match res.json::<HashMap<String, InspectInfo>>().await {
                    Ok(data) => {
                        debug_print(format_args!("received player stats"));
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
