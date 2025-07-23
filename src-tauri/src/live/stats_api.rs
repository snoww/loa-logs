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

        match self
            .client
            .post(format!("{API_URL}/inspect"))
            .json(&request_body)
            .send()
            .await
        {
            Ok(res) => match res.json::<HashMap<String, InspectInfo>>().await {
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

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct InspectInfo {
    pub combat_power: Option<CombatPower>,
    pub ark_passive_enabled: bool,
    pub ark_passive_data: Option<ArkPassiveData>,
    pub engravings: Option<Vec<u32>>,
    pub gems: Option<Vec<GemData>>,
    pub loadout_snapshot: Option<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct CombatPower {
    // 1 for dps, 2 for support
    pub id: u32,
    pub score: f32,
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
