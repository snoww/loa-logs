use crate::parser::debug_print;
use crate::parser::encounter_state::EncounterState;
use crate::parser::entity_tracker::Entity;
use crate::parser::models::EntityType;
use async_recursion::async_recursion;
use hashbrown::HashMap;
use log::{info, warn};
use md5::compute;
use moka::sync::Cache;
use reqwest::{Client, StatusCode};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, Window, Wry};

pub const API_URL: &str = "https://inspect.fau.dev";

pub struct StatsApi {
    pub client_id: String,
    client: Client,
    window: Arc<Window<Wry>>,
    pub valid_zone: bool,
    stats_cache: Cache<String, PlayerStats>,
    request_cache: Cache<String, PlayerStats>,
    inflight_cache: Cache<String, u8>,
    cancel_queue: Cache<String, String>,

    region_file_path: String,

    pub region: String,
}

impl StatsApi {
    pub fn new(window: Window<Wry>, region_file_path: String) -> Self {
        Self {
            client_id: String::new(),
            window: Arc::new(window),
            client: Client::new(),
            valid_zone: false,
            stats_cache: Cache::builder().max_capacity(64).build(),
            request_cache: Cache::builder().max_capacity(64).build(),
            inflight_cache: Cache::builder().max_capacity(32).build(),
            cancel_queue: Cache::builder()
                .max_capacity(16)
                .time_to_live(Duration::from_secs(15))
                .build(),
            region_file_path,

            region: "".to_string(),
        }
    }

    pub fn sync(&mut self, player: &Entity, state: &EncounterState) {
        if state.encounter.fight_start > 0
            && state.encounter.last_combat_packet - state.encounter.fight_start > 1_000
        {
            debug_print(format_args!("fight in progress, ignoring sync"));
            return;
        }

        if !self.valid_difficulty(&state.raid_difficulty) {
            self.broadcast("invalid_zone");
            return;
        }

        let region = match state.region.as_ref() {
            Some(region) => region.clone(),
            None => std::fs::read_to_string(&self.region_file_path).unwrap_or_else(|e| {
                warn!("failed to read region file. {}", e);
                "".to_string()
            }),
        };

        if region.is_empty() {
            warn!("region is not set");
            self.broadcast("missing_info");
            return;
        }

        self.region.clone_from(&region);

        if player.entity_type != EntityType::PLAYER {
            warn!("invalid entity type: {:?}", player);
            return;
        }

        let player_hash = if let Some(hash) = self.get_hash(player) {
            if let Some(cached) = self.request_cache.get(&hash) {
                info!("using cached stats for {:?}", player.name);
                self.stats_cache.insert(player.name.clone(), cached.clone());
                return;
            } else if !self.inflight_cache.contains_key(&hash) {
                self.inflight_cache.insert(hash.clone(), 0);
                self.stats_cache.invalidate(&player.name);
                self.cancel_queue.insert(player.name.clone(), hash.clone());
                PlayerHash {
                    name: player.name.clone(),
                    id: player.character_id,
                    hash,
                }
            } else {
                return;
            }
        } else {
            warn!("missing info for {:?}, could not generate hash", player);
            self.broadcast("missing_info");
            return;
        };

        self.request(region, player_hash);
    }

    fn request(&mut self, region: String, player: PlayerHash) {
        let client_clone = self.client.clone();
        let client_id_clone = self.client_id.clone();

        let stats_cache = self.stats_cache.clone();
        let request_cache = self.request_cache.clone();
        let inflight_cache = self.inflight_cache.clone();
        let cancel_queue = self.cancel_queue.clone();

        let window_clone = Arc::clone(&self.window);

        self.broadcast("requesting_stats");
        tokio::task::spawn(async move {
            make_request(
                &client_id_clone,
                &client_clone,
                &window_clone,
                &region,
                stats_cache,
                request_cache,
                inflight_cache,
                cancel_queue,
                player,
                0,
                false,
            )
            .await;
        });
    }

    pub fn get_hash(&self, player: &Entity) -> Option<String> {
        if player.gear_level < 0.0
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
        if let Some(equip_list) = player.items.equip_list.as_ref() {
            for item in equip_list.iter() {
                if item.slot >= 32 {
                    continue;
                }
                equip_data[item.slot as usize] = item.id;
            }
        }

        if equip_data[..26].iter().all(|&x| x == 0) {
            warn!("missing equipment data for {:?}", player);
            return Some("".to_string());
        }

        // {player_name}{xxxx.xx}{xxx}{character_id}{equip_data}
        let data = format!(
            "{}{:.02}{}{}{}",
            player.name,
            player.gear_level,
            player.class_id,
            player.character_id,
            equip_data.iter().map(|x| x.to_string()).collect::<String>()
        );

        Some(format!("{:x}", compute(data)))
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

    pub fn broadcast(&mut self, message: &str) {
        self.window
            .emit("rdps", message)
            .expect("failed to emit rdps message");
    }

    pub fn send_raid_info(&mut self, state: &EncounterState) {
        if !((self.valid_zone
            && (state.raid_difficulty == "Normal" || state.raid_difficulty == "Hard"))
            || (state.raid_difficulty == "Inferno"
                || state.raid_difficulty == "Trial"
                || state.raid_difficulty == "The First"))
        {
            debug_print(format_args!("not valid for raid info"));
            return;
        }

        let players: HashMap<String, u64> = state
            .encounter
            .entities
            .iter()
            .filter_map(|(_, e)| {
                if e.entity_type == EntityType::PLAYER {
                    Some((e.name.clone(), e.character_id))
                } else {
                    None
                }
            })
            .collect();

        if players.len() > 16 {
            warn!("invalid zone. num players: {}", players.len());
            return;
        }

        let client = self.client.clone();
        let client_id = self.client_id.clone();
        let version = self.window.app_handle().package_info().version.to_string();
        let region = self.region.clone();
        let boss_name = state.encounter.current_boss_name.clone();
        let difficulty = state.raid_difficulty.clone();
        let cleared = state.raid_clear;

        tokio::task::spawn(async move {
            let request_body = json!({
                "id": client_id,
                "version": version,
                "region": region,
                "boss": boss_name,
                "difficulty": difficulty,
                "characters": players,
                "cleared": cleared,
            });

            match client
                .post(format!("{API_URL}/raid"))
                .json(&request_body)
                .send()
                .await
            {
                Ok(_) => {
                    debug_print(format_args!("sent raid info"));
                }
                Err(e) => {
                    warn!("failed to send raid info: {:?}", e);
                }
            }
        });
    }
}

#[async_recursion]
async fn make_request(
    client_id: &str,
    client: &Client,
    window: &Arc<Window<Wry>>,
    region: &str,
    stats_cache: Cache<String, PlayerStats>,
    request_cache: Cache<String, PlayerStats>,
    inflight_cache: Cache<String, u8>,
    cancel_queue: Cache<String, String>,
    mut player: PlayerHash,
    current_retries: usize,
    mut final_attempt: bool,
) {
    if current_retries >= 10 {
        warn!(
            "# of retries exceeded, failed to fetch player stats for {:?}",
            player
        );
        inflight_cache.invalidate(&player.hash);
        cancel_queue.invalidate(&player.name);

        if !final_attempt {
            final_attempt = true;
            player.hash = "".to_string();
            warn!("final attempt for {:?} without hash", player.name);
        } else {
            window
                .emit("rdps", "request_failed")
                .expect("failed to emit rdps message");
            warn!("unable to find player {:?} on {:?}", player.name, region);
            return;
        }
    }

    let version = window.app_handle().package_info().version.to_string();
    let request_body = json!({
        "id": client_id,
        "version": version,
        "region": region,
        "player": player.clone(),
    });
    debug_print(format_args!("requesting player stats for {:?}", player));
    // debug_print(format_args!("{:?}", players));
    // println!("{:?}", request_body);

    match client
        .post(format!("{API_URL}/query"))
        .json(&request_body)
        .send()
        .await
    {
        Ok(res) => match res.status() {
            StatusCode::OK => {
                let data = res.json::<PlayerStats>().await;
                match data {
                    Ok(data) => {
                        debug_print(format_args!("received player stats for {:?}", player.name));
                        inflight_cache.invalidate(&data.hash);
                        stats_cache.insert(player.name.clone(), data.clone());
                        request_cache.insert(data.hash.clone(), data);
                        window
                            .emit("rdps", "request_success")
                            .expect("failed to emit rdps message");
                    }
                    Err(e) => {
                        warn!("failed to parse player stats: {:?}", e);
                    }
                }
            }
            StatusCode::NOT_FOUND => {
                window
                    .emit("rdps", "request_failed_retrying")
                    .expect("failed to emit rdps message");
                for _ in 0..20 {
                    if let Some(cancel_hash) = cancel_queue.get(&player.name) {
                        if cancel_hash != player.hash {
                            cancel_queue.invalidate(&player.name);
                            debug_print(format_args!(
                                "request cancelled for {:?}, using newer hash: {:?}",
                                player, cancel_hash
                            ));
                            return;
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }
                warn!(
                    "missing stats for: {:?}, retrying, attempt {}",
                    player,
                    current_retries + 1
                );
                // retry request with missing players
                // until we receive stats for all players
                make_request(
                    client_id,
                    client,
                    window,
                    region,
                    stats_cache,
                    request_cache,
                    inflight_cache,
                    cancel_queue,
                    player,
                    current_retries + 1,
                    final_attempt,
                )
                .await;
            }
            _ => {
                warn!("failed to fetch player stats: error {:?}", res.status());
                inflight_cache.invalidate(&player.hash);
                window
                    .emit("rdps", "api_error")
                    .expect("failed to emit rdps message");
            }
        },
        Err(e) => {
            warn!("failed to send api request: {:?}", e);
            inflight_cache.invalidate(&player.hash);
            window
                .emit("rdps", "api_error")
                .expect("failed to emit rdps message");
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
    pub name: String,
    pub hash: String,
    pub stats: Stats,
    pub elixirs: Option<Vec<ElixirData>>,
    pub gems: Option<Vec<GemData>>,
    pub engravings: Option<Vec<Engraving>>,
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
