use crate::parser::debug_print;
use crate::parser::encounter_state::EncounterState;
use crate::parser::entity_tracker::{Entity, EntityTracker};
use async_recursion::async_recursion;
use hashbrown::{HashMap, HashSet};
use log::{info, warn};
use md5::compute;
use moka::sync::Cache;
use reqwest::Client;
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::json;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{Manager, Window, Wry};

const API_URL: &str = "https://inspect.fau.dev/query";

pub struct StatsApi {
    pub client_id: String,
    cache_status: Arc<AtomicBool>,
    cancellation_flag: Arc<AtomicBool>,
    client: Arc<Client>,
    window: Arc<Window<Wry>>,
    pub valid_zone: bool,
    valid_stats: Option<bool>,
    stats_cache: Cache<String, PlayerStats>,
    request_cache: Cache<String, PlayerStats>,
    inflight_cache: Cache<String, u8>,
    pub status_message: String,
}

impl StatsApi {
    pub fn new(window: Window<Wry>) -> Self {
        Self {
            client_id: String::new(),
            window: Arc::new(window),
            cache_status: Arc::new(AtomicBool::new(false)),
            cancellation_flag: Arc::new(AtomicBool::new(false)),
            client: Arc::new(Client::new()),
            valid_zone: false,
            valid_stats: None,
            stats_cache: Cache::builder()
                .max_capacity(32)
                .time_to_idle(Duration::from_secs(60 * 10))
                .build(),
            request_cache: Cache::builder()
                .max_capacity(64)
                .time_to_idle(Duration::from_secs(60 * 30))
                .build(),
            inflight_cache: Cache::builder().max_capacity(16).build(),
            status_message: "".to_string(),
        }
    }

    pub fn sync(
        &mut self,
        party: &Vec<Vec<String>>,
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
            self.broadcast("missing_info");
            return;
        }
        if !self.valid_difficulty(&state.raid_difficulty) {
            debug_print(format_args!("stats not valid in current zone"));
            self.broadcast("invalid_zone");
            return;
        }

        let player_names = party.iter().flatten().cloned().collect::<HashSet<String>>();
        let mut player_hashes: Vec<PlayerHash> = Vec::new();
        for player in player_names.iter() {
            let entity_id = match state.encounter.entities.get(player) {
                Some(entity) => entity.id,
                None => continue,
            };
            if let Some(entity) = entity_tracker.entities.get(&entity_id) {
                if let Some(hash) = self.get_hash(entity) {
                    if let Some(cached) = self.request_cache.get(&hash) {
                        self.stats_cache.insert(player.clone(), cached.clone());
                    } else if !self.inflight_cache.contains_key(&hash) {
                        self.inflight_cache.insert(hash.clone(), 0);
                        player_hashes.push(PlayerHash {
                            name: player.clone(),
                            hash,
                        });
                    }
                } else {
                    debug_print(format_args!(
                        "missing info for {:?}, could not generate hash",
                        player
                    ));
                    self.broadcast("missing_info");
                    return;
                }
            }
        }
        
        if player_hashes.is_empty() {
            return;
        }
        
        debug_print(format_args!(
            "requesting for {}/{} players",
            player_hashes.len(),
            player_names.len()
        ));
        
        self.status_message = "".to_string();
        self.valid_stats = None;
        self.request(region, player_hashes);
    }

    fn request(&mut self, region: String, players: Vec<PlayerHash>) {
        let client_clone = Arc::clone(&self.client);
        let cache_status = Arc::clone(&self.cache_status);
        let client_id_clone = self.client_id.clone();

        let stats_cache = self.stats_cache.clone();
        let request_cache = self.request_cache.clone();
        let inflight_cache = self.inflight_cache.clone();

        self.cancellation_flag.store(true, Ordering::SeqCst);
        let new_cancellation_flag = Arc::new(AtomicBool::new(false));
        self.cancellation_flag = Arc::clone(&new_cancellation_flag);

        self.broadcast("requesting_stats");
        let window_clone = Arc::clone(&self.window);
        tokio::task::spawn(async move {
            make_request(
                &client_id_clone,
                &client_clone,
                &window_clone,
                &region,
                stats_cache,
                request_cache,
                inflight_cache,
                cache_status,
                new_cancellation_flag,
                players,
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
        difficulty: &str,
        party: &[Vec<String>],
        raid_duration: i64,
    ) -> Option<Cache<String, PlayerStats>> {
        if self.valid_stats.is_none() {
            let valid = party
                .iter()
                .flatten()
                .all(|player| self.stats_cache.contains_key(player));

            if valid || raid_duration >= 15_000 {
                self.valid_stats = Some(valid);
            }
        }

        if !self.valid_difficulty(difficulty) {
            return None;
        }
        if !self.valid_stats.unwrap_or(false) {
            if self.valid_stats.is_some() {
                self.broadcast("invalid_stats");
            }
            return None;
        }

        if self.cache_status.load(Ordering::Relaxed) {
            Some(self.stats_cache.clone())
        } else {
            None
        }
    }

    fn valid_difficulty(&self, difficulty: &str) -> bool {
        (difficulty == "Normal" || difficulty == "Hard" || difficulty == "The First")
            && self.valid_zone
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
    cache_status: Arc<AtomicBool>,
    cancellation: Arc<AtomicBool>,
    players: Vec<PlayerHash>,
    current_retries: usize,
) {
    if current_retries >= 10 {
        warn!(
            "# of retries exceeded, failed to fetch player stats for {:?}",
            players
        );
        remove_from_in_flight_cache(&inflight_cache, &players);
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
        "characters": players.clone(),
    });
    debug_print(format_args!("requesting player stats"));
    // debug_print(format_args!("{:?}", players));
    // println!("{:?}", request_body);

    match client.post(API_URL).json(&request_body).send().await {
        Ok(response) => match response.json::<HashMap<String, PlayerStats>>().await {
            Ok(data) => {
                let missing_players: Vec<PlayerHash> = players
                    .iter()
                    .filter(|player| !data.contains_key(&player.name))
                    .cloned()
                    .collect();

                for (name, stats) in data {
                    inflight_cache.remove(&stats.hash);
                    stats_cache.insert(name.clone(), stats.clone());
                    request_cache.insert(stats.hash.clone(), stats);
                }
                // debug_print(format_args!("{:?}", stats_cache_clone));

                if missing_players.is_empty() {
                    debug_print(format_args!("received player stats"));
                    cache_status.store(true, Ordering::Relaxed);
                    window
                        .emit("rdps", "request_success")
                        .expect("failed to emit rdps message");
                } else {
                    cache_status.store(false, Ordering::Relaxed);
                    window
                        .emit("rdps", "request_failed_retrying")
                        .expect("failed to emit rdps message");
                    if cancellation.load(Ordering::SeqCst) {
                        debug_print(format_args!("request cancelled"));
                        remove_from_in_flight_cache(&inflight_cache, &players);
                        return;
                    }
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if cancellation.load(Ordering::SeqCst) {
                        debug_print(format_args!("request cancelled"));
                        remove_from_in_flight_cache(&inflight_cache, &players);
                        return;
                    }
                    debug_print(format_args!(
                        "missing stats for: {:?}, retrying, attempt {}",
                        missing_players,
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
                        cache_status,
                        cancellation,
                        missing_players,
                        current_retries + 1,
                    )
                    .await;
                }
            }
            Err(e) => {
                cache_status.store(false, Ordering::Relaxed);
                warn!("failed to parse player stats: {:?}", e);
            }
        },
        Err(e) => {
            cache_status.store(false, Ordering::Relaxed);
            warn!("failed to fetch player stats: {:?}", e);
        }
    }
}

fn remove_from_in_flight_cache(in_flight_cache: &Cache<String, u8>, players: &[PlayerHash]) {
    for player in players.iter() {
        in_flight_cache.remove(&player.hash);
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
