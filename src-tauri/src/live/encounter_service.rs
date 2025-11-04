use std::collections::BTreeMap;

use crate::emitter::{AppEmitter, TauriAppEmitter};
use crate::live::utils::update_current_boss_name;
use crate::{database::Repository, live::stats_api::StatsApi};
use crate::models::*;
use hashbrown::HashMap;
use log::*;
use tauri::{AppHandle, Emitter, Manager};
use tokio::task;

#[cfg(test)]
use mockall::{automock, predicate::*};

use crate::database::models::InsertEncounterArgs;

#[derive(Debug, Clone)]
pub struct SaveEncounterArgs {
    pub encounter: Encounter,
    pub party_info: Vec<Vec<String>>,
    pub current_boss_name: String,
    pub damage_log: HashMap<String, Vec<(i64, i64)>>,
    pub cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,
    pub boss_hp_log: HashMap<String, Vec<BossHpLog>>,
    pub raid_clear: bool,
    pub raid_difficulty: String,
    pub region: Option<String>,
    pub ntp_fight_start: i64,
    pub rdps_valid: bool,
    pub manual: bool,
    pub skill_cast_log: HashMap<u64, HashMap<u32, BTreeMap<i64, SkillCast>>>,
    pub skill_cooldowns: HashMap<u32, Vec<CastEvent>>,
}

#[cfg_attr(test, automock)]
pub trait EncounterService {
    fn save(&self, args: SaveEncounterArgs);
    fn send(&self, boss_dead: bool, damage_valid: bool, clone: Encounter, party_info: Option<Vec<Vec<String>>>);
}

#[derive(Debug, Clone)]
pub struct DefaultEncounterService {
    app_handle: AppHandle,
    version: String
}

impl EncounterService for DefaultEncounterService {
    
    fn save(&self, args: SaveEncounterArgs) {

        let app_handle = self.app_handle.clone();
        let meter_version = self.version.clone();

        task::spawn(async move {

            let SaveEncounterArgs {
                raid_clear,
                raid_difficulty,
                current_boss_name,
                mut encounter,
                damage_log,
                cast_log,
                boss_hp_log,
                region,
                ntp_fight_start,
                rdps_valid,
                manual,
                skill_cast_log,
                skill_cooldowns,
                party_info
            } = args;

            info!(
                "saving to db - cleared: [{}], difficulty: [{}] {}",
                raid_clear, raid_difficulty, encounter.current_boss_name
            );

            let stats_api = app_handle.state::<StatsApi>();

            let player_info = if !raid_difficulty.is_empty()
                && raid_difficulty != "Inferno"
                && raid_difficulty != "Trial"
                && !current_boss_name.is_empty()
            {
                info!("fetching player info");
                stats_api.get_character_info(&encounter).await
            } else {
                None
            };

            let repository = app_handle.state::<Repository>();

            encounter.current_boss_name = update_current_boss_name(&encounter.current_boss_name);
            let args = InsertEncounterArgs {
                encounter,
                damage_log,
                cast_log,
                boss_hp_log,
                raid_clear,
                party_info,
                raid_difficulty,
                region,
                player_info,
                meter_version,
                ntp_fight_start,
                rdps_valid,
                manual,
                skill_cast_log,
                skill_cooldowns
            };

            let encounter_id = repository
                .insert_data(args)
                .expect("could not save encounter");

            info!("saved to db");

            if raid_clear {
                let emitter = app_handle.state::<TauriAppEmitter>();
                emitter.emit("clear-encounter", encounter_id);
            }
        });
    }
    
    fn send(&self, boss_dead: bool, damage_valid: bool, mut clone: Encounter, party_info: Option<Vec<Vec<String>>>) {
        
        let app_handle = self.app_handle.clone();

        tokio::task::spawn(async move {

            let emitter = app_handle.state::<TauriAppEmitter>();

            if !clone.current_boss_name.is_empty() {
                let current_boss = clone.entities.get(&clone.current_boss_name).cloned();
                if let Some(mut current_boss) = current_boss {
                    if boss_dead {
                        current_boss.is_dead = true;
                        current_boss.current_hp = 0;
                    }
                    clone.current_boss = Some(current_boss);
                } else {
                    clone.current_boss_name = String::new();
                }
            }
            clone.entities.retain(|_, e| {
                ((e.entity_type == EntityType::Player && e.class_id > 0)
                    || e.entity_type == EntityType::Esther
                    || e.entity_type == EntityType::Boss)
                    && e.damage_stats.damage_dealt > 0
            });

            if !clone.entities.is_empty() {
                if !damage_valid {
                    emitter.emit("invalid-damage", "");
                } else {
                    emitter.emit("encounter-update", Some(clone));

                    if party_info.is_some() {
                        emitter.emit("party-update", party_info);
                    }
                }
            }
        });
    }
}

impl DefaultEncounterService {
    pub fn new(app_handle: AppHandle, version: String) -> Self {
        Self {
            app_handle,
            version
        }
    }
}