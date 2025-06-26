use crate::live::debug_print;
use crate::live::encounter_state::EncounterState;
use crate::live::utils::{boss_to_raid_map, is_valid_player};
use crate::parser::models::{ArkPassiveData, Encounter, EntityType};
use hashbrown::HashMap;
use log::warn;
use moka::sync::Cache;
use reqwest::Client;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::fmt;
use std::sync::Arc;
use tauri::{AppHandle, Manager, Window, Wry};

// pub const API_URL: &str = "http://localhost:5180";
pub const API_URL: &str = "https://api.snow.xyz";

#[derive(Clone)]
pub struct StatsApi {
    pub client_id: String,
    client: Client,
    app: AppHandle,
    pub valid_zone: bool,
    stats_cache: Cache<String, PlayerStats>,
}

impl StatsApi {
    pub fn new(app: AppHandle) -> Self {
        Self {
            client_id: String::new(),
            app,
            client: Client::new(),
            valid_zone: false,
            stats_cache: Cache::builder().max_capacity(64).build(),
        }
    }

    pub fn get_stats(&mut self, state: &EncounterState) -> Option<Cache<String, PlayerStats>> {
        if !self.valid_difficulty(&state.raid_difficulty) {
            return None;
        }

        Some(self.stats_cache.clone())
    }

    fn valid_difficulty(&self, difficulty: &str) -> bool {
        self.valid_zone
            && (difficulty == "Normal"
                || difficulty == "Hard"
                || difficulty == "The First"
                || difficulty == "Trial")
    }

    pub async fn get_character_info(
        &self,
        encounter: &Encounter,
    ) -> Option<HashMap<String, PlayerStats>> {
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

        match self
            .client
            .post(format!("{API_URL}/inspect"))
            .json(&request_body)
            .send()
            .await
        {
            Ok(res) => match res.json::<HashMap<String, PlayerStats>>().await {
                Ok(data) => {
                    debug_print(format_args!("received player stats"));
                    Some(data)
                }
                Err(e) => {
                    warn!("failed to parse player stats: {:?}", e);
                    None
                }
            },
            Err(e) => {
                warn!("failed to get inspect data: {:?}", e);
                None
            }
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub crit: u32,
    pub spec: u32,
    pub swift: u32,
    pub exp: u32,
    pub atk_power: u32,
    pub add_dmg: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerStats {
    pub ark_passive_enabled: bool,
    pub ark_passive_data: Option<ArkPassiveData>,
    pub engravings: Option<Vec<u32>>,
    pub gems: Option<Vec<GemData>>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ElixirData {
    pub slot: u8,
    pub entries: Vec<ElixirEntry>,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct ElixirEntry {
    pub id: u32,
    pub level: u8,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GemData {
    pub tier: u8,
    pub skill_id: u32,
    pub gem_type: u8,
    pub value: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Engraving {
    pub id: u32,
    pub level: u8,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerHash {
    pub name: String,
    pub hash: String,
    pub id: u64,
}

struct StatsVisitor;

impl<'de> Visitor<'de> for StatsVisitor {
    type Value = Stats;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with integer keys")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut stats = Stats::default();
        while let Some((key, value)) = map.next_entry::<usize, u32>()? {
            if key == 0 {
                stats.crit = value;
            } else if key == 1 {
                stats.spec = value;
            } else if key == 2 {
                stats.swift = value;
            } else if key == 3 {
                stats.exp = value;
            } else if key == 4 {
                stats.atk_power = value;
            } else if key == 5 {
                stats.add_dmg = value;
            }
        }
        Ok(stats)
    }
}

impl<'de> Deserialize<'de> for Stats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(StatsVisitor)
    }
}
