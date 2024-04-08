use crate::parser::debug_print;
use crate::parser::encounter_state::EncounterState;
use crate::parser::entity_tracker::{Entity, EntityTracker};
use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use log::{info, warn};
use md5::compute;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

const API_URL: &str = "https://inspect.fau.dev/query";

pub struct StatsApi {
    cache: Arc<Mutex<HashMap<String, PlayerStats>>>,
    stats_cache: Arc<Mutex<HashMap<String, Stats>>>,
    cache_status: Arc<AtomicBool>,
    client: Arc<Client>,
    hash: HashMap<String, String>,
}

impl StatsApi {
    pub fn new() -> Self {
        Self {
            cache: Arc::new(Mutex::new(HashMap::new())),
            stats_cache: Arc::new(Mutex::new(HashMap::new())),
            cache_status: Arc::new(AtomicBool::new(false)),
            client: Arc::new(Client::new()),
            hash: HashMap::new(),
        }
    }

    pub fn sync(
        &mut self,
        party: Vec<Vec<String>>,
        state: &EncounterState,
        entity_tracker: &EntityTracker,
        cached: &HashMap<u64, String>,
    ) {
        let region = match state.region.as_ref() {
            Some(region) => region.clone(),
            None => cached.get(&0).cloned().unwrap_or_default(),
        };
        if party.is_empty() || region.is_empty() {
            debug_print(format_args!("party info is empty or region is not set"));
            return;
        }

        let player_names = party.iter().flatten().cloned().collect::<Vec<String>>();
        let mut player_hashes: Vec<PlayerHash> = Vec::new();
        for player in player_names.iter() {
            let entity_id = match state.encounter.entities.get(player) {
                Some(entity) => entity.id,
                None => continue,
            };
            if let Some(entity) = entity_tracker.entities.get(&entity_id) {
                if let Some(hash) = self.get_hash(entity) {
                    if self
                        .hash
                        .get(player)
                        .map_or(false, |cached_hash| cached_hash == &hash)
                    {
                        continue;
                    } else {
                        self.hash.insert(player.clone(), hash.clone());
                        player_hashes.push(PlayerHash {
                            name: player.clone(),
                            hash,
                            expiry: Utc::now() + Duration::minutes(5),
                        });
                    }
                }
            }
        }
        self.request(region, player_hashes);
    }

    fn request(&self, region: String, players: Vec<PlayerHash>) {
        let client_clone = Arc::clone(&self.client);
        let cache_lock = Arc::clone(&self.cache);
        let cache_status = Arc::clone(&self.cache_status);
        let stats_cache_lock = Arc::clone(&self.stats_cache);
        tokio::task::spawn(async move {
            let request_body = json!({
                "region": region,
                "characters": players,
            });
            debug_print(format_args!("requesting player stats"));
            match client_clone.get(API_URL).json(&request_body).send().await {
                Ok(response) => match response.json::<HashMap<String, PlayerStats>>().await {
                    Ok(data) => {
                        debug_print(format_args!("received player stats"));
                        let mut cache_clone = cache_lock.lock().unwrap();
                        let mut stats_cache_clone = stats_cache_lock.lock().unwrap();
                        for (name, stats) in data {
                            stats_cache_clone.insert(
                                name.clone(),
                                Stats {
                                    crit: stats.stats.get(&0).cloned().unwrap_or_default(),
                                    spec: stats.stats.get(&1).cloned().unwrap_or_default(),
                                    atk_power: stats.stats.get(&4).cloned().unwrap_or_default(),
                                    add_dmg: stats.stats.get(&5).cloned().unwrap_or_default(),
                                },
                            );
                            cache_clone.insert(name, stats);
                        }
                        cache_status.store(true, Ordering::Relaxed);
                    }
                    Err(e) => {
                        warn!("failed to parse player stats: {:?}", e);
                    }
                },
                Err(e) => {
                    warn!("failed to fetch player stats: {:?}", e);
                }
            }
        });
    }

    pub fn get_hash(&self, player: &Entity) -> Option<String> {
        if player.items.equip_list.is_none()
            || player.character_id == 0
            || player.class_id == 0
            || player.name == "You"
            || !player
                .name
                .chars()
                .next()
                .unwrap_or_default()
                .is_uppercase()
        {
            return None;
        }

        let mut equip_data: [u32; 32] = [0; 32];
        for item in player.items.equip_list.as_ref().unwrap().iter() {
            equip_data[item.slot as usize] = item.id;
        }

        let data = format!(
            "{}{}{}{}",
            player.name,
            player.class_id,
            player.character_id,
            equip_data.iter().map(|x| x.to_string()).collect::<String>()
        );

        Some(format!("{:x}", compute(data)))
    }

    pub fn get_all_stats(&self) -> Option<HashMap<String, PlayerStats>> {
        if self.cache_status.load(Ordering::Relaxed) {
            if let Ok(cache) = self.cache.lock() {
                Some(cache.clone())
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_stats(&self, difficulty: &str) -> Option<HashMap<String, Stats>> {
        if difficulty == "Inferno" || difficulty == "Challenge" {
            return None;
        }
        if self.cache_status.load(Ordering::Relaxed) {
            if let Ok(cache) = self.stats_cache.lock() {
                Some(cache.clone())
            } else {
                None
            }
        } else {
            None
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct Stats {
    pub crit: u32,
    pub spec: u32,
    pub atk_power: u32,
    pub add_dmg: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerStats {
    pub name: String,
    pub stats: HashMap<u8, u32>,
    pub elixirs: Option<Vec<ElixirData>>,
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
    pub id: u32,
    pub skill_id: u32,
    #[serde(alias = "type")]
    pub gem_type: u8,
    pub value: u32,
}

#[derive(Debug, Default, Clone, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct EngravingData {
    pub name: String,
    pub level: u8,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase", default)]
pub struct PlayerHash {
    pub name: String,
    pub hash: String,
    #[serde(skip)]
    pub expiry: DateTime<Utc>,
}
