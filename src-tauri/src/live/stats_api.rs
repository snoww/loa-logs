use crate::live::debug_print;
use crate::live::utils::{boss_to_raid_map, is_valid_player};
use crate::models::*;
use hashbrown::HashMap;
use log::warn;
use reqwest::Client;
use serde_json::json;
use sha2::{Digest, Sha256};
use std::time::Duration;
use serde::Serialize;
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

    pub async fn send_raid_analytics(
        &self,
        encounter: &Encounter,
        battle_items: HashMap<u32, u32>,
        cc_tracker: HashMap<u32, u32>,
    ) {
        if encounter.region.is_none() {
            return;
        }

        if let Some(local_player) = encounter.entities.get(&encounter.local_player) {
            if !is_valid_player(local_player) {
                return;
            }
        } else {
            return;
        }

        let mut player_names: Vec<String> = Vec::new();
        let mut participants: Vec<ParticipantInfo> = Vec::new();
        for player in encounter
            .entities
            .values()
            .filter(|e| e.entity_type == EntityType::Player)
        {
            player_names.push(player.name.clone());
            participants.push(ParticipantInfo {
                class: player.class.clone(),
                damage_done: player.damage_stats.damage_dealt,
                damage_taken: player.damage_stats.damage_taken,
                counters: player.skill_stats.counters,
                died_at: if player.damage_stats.deaths > 0 { Some(player.damage_stats.death_time) } else { None },
                boss_hp_at_death: player.damage_stats.boss_hp_at_death,
            })
        }

        player_names.sort();

        let hash = Sha256::digest(player_names.join("").as_bytes());
        let duration_s = (encounter.last_combat_packet - encounter.fight_start) / 1000;
        let boss_hp = encounter
            .entities
            .get(&encounter.current_boss_name)
            .map(|boss| boss.current_hp)
            .unwrap_or_default();
        let mut esther_skills: HashMap<u32, u32> = HashMap::new();
        encounter
            .entities
            .values()
            .filter(|e| e.entity_type == EntityType::Esther)
            .flat_map(|e| {
                e.skills
                    .iter()
                    .map(|(&skill_id, skill)| (skill_id, skill.casts as u32))
            })
            .for_each(|(skill_id, casts)| {
                *esther_skills.entry(skill_id).or_insert(0) += casts;
            });

        let request_body = json!({
            "participantsHash": format!("{:x}", hash),
            "boss": encounter.current_boss_name,
            "difficulty": encounter.difficulty,
            "startTime": encounter.fight_start,
            "duration": duration_s,
            "clear": encounter.cleared,
            "finalBossHP": boss_hp,
            "battleItemsUsed": battle_items,
            "crowdControlDebuffs": cc_tracker,
            "estherCasts": esther_skills,
            "participants": participants,
        });

        let _ = self
            .client
            .post(format!("{API_URL}/analytics/raid"))
            .json(&request_body)
            .send()
            .await;
    }
}

#[derive(Debug, Serialize, Clone)]
struct ParticipantInfo {
    pub class: String,
    pub damage_done: i64,
    pub damage_taken: i64,
    pub counters: i64,
    pub died_at: Option<i64>,
    pub boss_hp_at_death: Option<i64>,
}
