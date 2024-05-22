use crate::parser::debug_print;
use crate::parser::encounter_state::EncounterState;
use crate::parser::entity_tracker::Entity;
use crate::parser::models::EntityType;
use async_recursion::async_recursion;
use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use log::warn;
use md5::compute;
use moka::sync::Cache;
use reqwest::Client;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::fmt;
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, Window, Wry};

const API_URL: &str = "https://inspect.fau.dev/query";

pub struct StatsApi {
    pub client_id: String,
    client: Arc<Client>,
    window: Arc<Window<Wry>>,
    pub valid_zone: bool,
    valid_stats: Option<bool>,
    stats_cache: Cache<String, PlayerStats>,
    request_cache: Cache<String, PlayerStats>,
    inflight_cache: Cache<String, u8>,
    cancel_queue: Cache<String, String>,
    pub status_message: String,
    last_broadcast: DateTime<Utc>,
}

impl StatsApi {
    pub fn new(window: Window<Wry>) -> Self {
        Self {
            client_id: String::new(),
            window: Arc::new(window),
            client: Arc::new(Client::new()),
            valid_zone: false,
            valid_stats: None,
            stats_cache: Cache::builder().max_capacity(32).build(),
            request_cache: Cache::builder().max_capacity(64).build(),
            inflight_cache: Cache::builder()
                .max_capacity(16)
                .time_to_live(Duration::from_secs(30))
                .build(),
            cancel_queue: Cache::builder()
                .max_capacity(16)
                .time_to_live(Duration::from_secs(30))
                .build(),
            status_message: "".to_string(),
            last_broadcast: Utc::now(),
        }
    }

    pub fn sync(&mut self, player: &Entity, state: &EncounterState) {
        if !self.valid_difficulty(&state.raid_difficulty) {
            self.broadcast("invalid_zone");
            return;
        }

        let region = match state.region.as_ref() {
            Some(region) => region.clone(),
            None => "".to_string(),
        };

        if region.is_empty() {
            debug_print(format_args!("region is not set"));
            self.broadcast("missing_info");
            return;
        }

        let player_hash = if let Some(hash) = self.get_hash(player) {
            if let Some(cached) = self.request_cache.get(&hash) {
                self.stats_cache.insert(player.name.clone(), cached.clone());
                return;
            } else if !self.inflight_cache.contains_key(&hash) {
                self.inflight_cache.insert(hash.clone(), 0);
                self.stats_cache.invalidate(&player.name);
                self.cancel_queue.insert(player.name.clone(), hash.clone());
                PlayerHash {
                    name: player.name.clone(),
                    hash,
                }
            } else {
                return;
            }
        } else {
            debug_print(format_args!(
                "missing info for {:?}, could not generate hash",
                player
            ));
            self.broadcast("missing_info");
            return;
        };

        self.status_message = "".to_string();
        self.valid_stats = None;
        self.request(region, player_hash);
    }

    fn request(&mut self, region: String, player: PlayerHash) {
        let client_clone = Arc::clone(&self.client);
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
            )
            .await;
        });
    }

    pub fn get_hash(&self, player: &Entity) -> Option<String> {
        if player.items.equip_list.is_none()
            || player.gear_level < 0.0
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

    pub fn get_stats(
        &mut self,
        state: &EncounterState,
        raid_duration: i64,
    ) -> Option<Cache<String, PlayerStats>> {
        if !self.valid_difficulty(&state.raid_difficulty) {
            return None;
        }

        if self.valid_stats.is_none() {
            let valid = state
                .encounter
                .entities
                .iter()
                .filter(|(_, e)| e.entity_type == EntityType::PLAYER)
                .all(|(name, _)| self.stats_cache.contains_key(name));

            if valid || raid_duration >= 15_000 {
                self.valid_stats = Some(valid);
            }
        }

        if !self.valid_stats.unwrap_or(false) {
            let now = Utc::now();
            let duration = now.signed_duration_since(self.last_broadcast).num_seconds();
            if self.valid_stats.is_some() && duration >= 10 {
                self.broadcast("invalid_stats");
                self.last_broadcast = now;
            }
            return None;
        }

        Some(self.stats_cache.clone())
    }

    fn valid_difficulty(&self, difficulty: &str) -> bool {
        self.valid_zone
            && (difficulty == "Normal" || difficulty == "Hard" || difficulty == "The First" || difficulty == "Trial")
    }

    pub fn broadcast(&mut self, message: &str) {
        self.status_message = message.to_string();
        self.window
            .emit("rdps", message)
            .expect("failed to emit rdps message");
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
    player: PlayerHash,
    current_retries: usize,
) {
    if current_retries >= 20 {
        warn!(
            "# of retries exceeded, failed to fetch player stats for {:?}",
            player
        );
        inflight_cache.invalidate(&player.hash);
        cancel_queue.invalidate(&player.name);
        window
            .emit("rdps", "request_failed")
            .expect("failed to emit rdps message");
        return;
    }

    let version = window.app_handle().package_info().version.to_string();
    let request_body = json!({
        "id": client_id,
        "version": version,
        "region": region.clone(),
        "characters": vec![player.clone()],
    });
    debug_print(format_args!("requesting player stats for {:?}", player));
    // debug_print(format_args!("{:?}", players));
    // println!("{:?}", request_body);

    match client.post(API_URL).json(&request_body).send().await {
        Ok(response) => match response.json::<HashMap<String, PlayerStats>>().await {
            Ok(data) => {
                if data.contains_key(&player.name) {
                    // should only contain 1 element
                    for (name, stats) in data {
                        inflight_cache.invalidate(&stats.hash);
                        stats_cache.insert(name.clone(), stats.clone());
                        request_cache.insert(stats.hash.clone(), stats);
                    }
                    debug_print(format_args!("received player stats for {:?}", player.name));
                    window
                        .emit("rdps", "request_success")
                        .expect("failed to emit rdps message");
                } else {
                    window
                        .emit("rdps", "request_failed_retrying")
                        .expect("failed to emit rdps message");
                    for _ in 0..30 {
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
                    debug_print(format_args!(
                        "missing stats for: {:?}, retrying, attempt {}",
                        player,
                        current_retries + 1
                    ));
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
                    )
                    .await;
                }
            }
            Err(e) => {
                warn!("failed to parse player stats: {:?}", e);
            }
        },
        Err(e) => {
            warn!("failed to fetch player stats: {:?}", e);
            window
                .emit("rdps", "api_error")
                .expect("failed to emit rdps message");
        }
    }
}

// #[derive(Debug, Default, Clone)]
// pub struct Stats {
//     pub crit: u32,
//     pub spec: u32,
//     pub atk_power: u32,
//     pub add_dmg: u32,
// }

#[derive(Debug, Default, Clone)]
pub struct Stats(pub Vec<u32>);

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
        let mut stats = vec![0; 6];
        while let Some((key, value)) = map.next_entry::<usize, u32>()? {
            if key < stats.len() {
                stats[key] = value;
            }
        }
        Ok(Stats(stats))
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
