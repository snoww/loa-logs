use hashbrown::HashMap;
use serde::Serialize;
use log::*;
use sha2::{Digest, Sha256};
use crate::{models::{Encounter, EntityType}, utils::{boss_to_raid_map, is_valid_player}};

#[derive(Debug, Serialize, Clone)]
struct ParticipantInfo<'a> {
    pub class: &'a str,
    pub damage_done: i64,
    pub damage_taken: i64,
    pub counters: i64,
    pub died_at: Option<i64>,
    pub boss_hp_at_death: Option<i64>,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendHeartbeatArgs<'a> {
    pub id: &'a str,
    pub version: &'a str,
    pub region: &'a str
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GetCharacterInfoArgs<'a> {
    pub client_id: &'a str,
    pub version: &'a str,
    pub region: &'a str,
    pub raid_name: String,
    pub boss: &'a str,
    pub characters: Vec<&'a str>,
    pub difficulty: &'a str,
    pub cleared: bool,
}

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendRaidAnalyticsArgs<'a> {
    participants_hash: String,
    boss: &'a str,
    difficulty: &'a str,
    start_time: i64,
    duration: i64,
    clear: bool,
    #[serde(rename(serialize = "finalBossHP"))]
    final_boss_hp: i64,
    #[serde(rename(serialize = "bossMaxHP"))]
    boss_max_hp: i64,
    battle_items_used: HashMap<u32, u32>,
    crowd_control_debuffs: HashMap<u32, u32>,
    esther_casts: HashMap<u32, u32>,
    participants: Vec<ParticipantInfo<'a>>,
}

impl<'a> SendRaidAnalyticsArgs<'a> {
    pub fn new(
        encounter: &'a Encounter,
        difficulty: &'a str,
        battle_items_used: HashMap<u32, u32>,
        crowd_control_debuffs: HashMap<u32, u32>) -> Option<Self> {
        
        let Encounter { 
            last_combat_packet,
            fight_start,
            local_player,
            entities,
            current_boss_name,
            cleared,
            ..
        } = encounter;
        
        if let Some(local_player) = entities.get(local_player) {
            if !is_valid_player(local_player) {
                return None;
            }
        } else {
            return None;
        }

        let mut player_names: Vec<&'a str> = Vec::new();
        let mut participants: Vec<ParticipantInfo> = Vec::new();
        let entities_iter = entities.values();

        for player in entities_iter.filter(|e| e.entity_type == EntityType::Player)
        {
            player_names.push(&player.name);
            let info: ParticipantInfo = ParticipantInfo {
                class: &player.class,
                damage_done: player.damage_stats.damage_dealt,
                damage_taken: player.damage_stats.damage_taken,
                counters: player.skill_stats.counters,
                died_at: if player.damage_stats.deaths > 0 {
                    Some(player.damage_stats.death_time)
                } else {
                    None
                },
                boss_hp_at_death: player.damage_stats.boss_hp_at_death,
            };
            participants.push(info);
        }

        player_names.sort();

        let participants_hash = {
            let hash = Sha256::digest(player_names.join("").as_bytes());
            format!("{:x}", hash)
        };
        
        let duration = (last_combat_packet - fight_start) / 1000;
        let boss = match entities.get(current_boss_name) {
            Some(value) => value,
            None => return None,
        };

        let entities_iter = entities.values();
        let esther_casts = entities_iter
            .filter(|e| e.entity_type == EntityType::Esther)
            .flat_map(|e| {
                e.skills
                    .iter()
                    .map(|(&skill_id, skill)| (skill_id, skill.casts as u32))
            })
            .fold(HashMap::new(), |mut acc, (skill_id, casts)| {
                *acc.entry(skill_id).or_insert(0) += casts;
                acc
            });

        let final_boss_hp = boss.current_hp;
        let boss_max_hp = boss.max_hp;
        let boss = current_boss_name;
        let start_time = *fight_start;
        let clear = *cleared;

        Some(Self {
            participants_hash,
            boss,
            difficulty,
            start_time,
            duration,
            clear,
            final_boss_hp,
            boss_max_hp,
            battle_items_used,
            crowd_control_debuffs,
            esther_casts,
            participants
        })
    }
}

impl<'a> GetCharacterInfoArgs<'a> {
    pub fn new(encounter: &'a Encounter, difficulty: &'a str) -> Option<Self> {

        if difficulty.is_empty() || matches!(difficulty, "Inferno" | "Trial") {
            return None
        }
        
        let boss: &str = (!encounter.current_boss_name.is_empty()).then(|| encounter.current_boss_name.as_ref())?;

        let region = match encounter.region.as_ref() {
            Some(region) => region,
            None => {
                warn!("region is not set");
                return None;
            }
        };

        let raid_name = encounter
            .entities
            .get(&encounter.current_boss_name)
            .and_then(|boss| boss_to_raid_map(&encounter.current_boss_name, boss.max_hp))
            .unwrap_or_default();

        let characters: Vec<&str> = encounter
            .entities
            .iter()
            .filter_map(|(_, e)| {
                if is_valid_player(e) {
                    Some(e.name.as_ref())
                } else {
                    None
                }
            })
            .collect();

        if characters.len() > 16 {
            return None;
        }

        Some(GetCharacterInfoArgs {
            version: "",
            client_id: "",
            boss,
            raid_name,
            characters,
            cleared: encounter.cleared,
            region,
            difficulty
        })
    }
}
