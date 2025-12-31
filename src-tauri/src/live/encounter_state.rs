use crate::api::{GetCharacterInfoArgs, SendRaidAnalyticsArgs, StatsApi};
use crate::data::*;
use crate::database::Repository;
use crate::database::models::InsertEncounterArgs;
use crate::live::emitter::AppEmitter;
use crate::live::skill_tracker::SkillTracker;
use crate::live::sntp::TimeSyncClient;
use crate::live::utils::*;
use crate::models::*;
use crate::utils::{get_class_from_id, get_player_spec, is_support_class};
use chrono::Utc;
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::common::SkillMoveOptionData;
use meter_core::packets::structures::SkillCooldownStruct;
use rsntp::SntpClient;
use std::cmp::max;
use std::default::Default;
use tauri::{AppHandle, Emitter, Manager};
use tokio::task;

#[derive(Debug)]
pub struct EncounterState {
    pub encounter: Encounter,
    pub resetting: bool,
    pub boss_dead_update: bool,
    pub saved: bool,

    pub raid_clear: bool,

    damage_log: HashMap<String, Vec<(i64, i64)>>,
    cast_log: HashMap<String, HashMap<u32, Vec<i32>>>,

    boss_hp_log: HashMap<String, Vec<BossHpLog>>,

    // item_id -> count
    battle_item_tracker: HashMap<u32, u32>,
    // buff_id -> count
    crowd_control_tracker: HashMap<u32, u32>,

    pub intermission_start: Option<i64>,
    pub intermission_end: Option<i64>,

    pub party_info: Vec<Vec<String>>,
    pub raid_difficulty: String,
    pub raid_difficulty_id: u32,
    pub boss_only_damage: bool,
    pub region: Option<String>,

    sntp_client: SntpClient,
    ntp_fight_start: i64,

    pub rdps_valid: bool,

    pub skill_tracker: SkillTracker,

    pub custom_id_map: HashMap<u32, u32>,

    pub damage_is_valid: bool,
}

impl EncounterState {
    pub fn new() -> EncounterState {
        EncounterState {
            encounter: Encounter::default(),
            resetting: false,
            raid_clear: false,
            boss_dead_update: false,
            saved: false,

            damage_log: HashMap::new(),
            boss_hp_log: HashMap::new(),
            cast_log: HashMap::new(),
            battle_item_tracker: HashMap::new(),
            crowd_control_tracker: HashMap::new(),
            intermission_start: None,
            intermission_end: None,

            party_info: Vec::new(),
            raid_difficulty: "".to_string(),
            raid_difficulty_id: 0,
            boss_only_damage: false,
            region: None,

            sntp_client: SntpClient::new(),
            ntp_fight_start: 0,

            // todo
            rdps_valid: false,

            skill_tracker: SkillTracker::new(),

            custom_id_map: HashMap::new(),

            damage_is_valid: true,
        }
    }

    // keep all player entities, reset all stats
    pub fn soft_reset(&mut self, keep_bosses: bool) {
        let clone = self.encounter.clone();

        self.encounter.fight_start = 0;
        self.encounter.boss_only_damage = self.boss_only_damage;
        self.encounter.entities = HashMap::new();
        self.encounter.current_boss_name = "".to_string();
        self.encounter.encounter_damage_stats = Default::default();
        self.raid_clear = false;

        self.damage_log = HashMap::new();
        self.cast_log = HashMap::new();
        self.boss_hp_log = HashMap::new();
        self.battle_item_tracker = HashMap::new();
        self.crowd_control_tracker = HashMap::new();
        self.intermission_start = None;
        self.intermission_end = None;
        self.party_info = Vec::new();

        self.ntp_fight_start = 0;

        self.rdps_valid = false;

        self.skill_tracker = SkillTracker::new();

        self.custom_id_map = HashMap::new();

        for (key, entity) in clone.entities.into_iter().filter(|(_, e)| {
            e.entity_type == EntityType::Player
                || (keep_bosses && e.entity_type == EntityType::Boss)
        }) {
            self.encounter.entities.insert(
                key,
                EncounterEntity {
                    name: entity.name,
                    id: entity.id,
                    character_id: entity.character_id,
                    npc_id: entity.npc_id,
                    class: entity.class,
                    class_id: entity.class_id,
                    entity_type: entity.entity_type,
                    gear_score: entity.gear_score,
                    max_hp: entity.max_hp,
                    current_hp: entity.max_hp,
                    is_dead: entity.is_dead,
                    ..Default::default()
                },
            );
        }
    }

    // update local player as we get more info
    pub fn update_local_player(&mut self, entity: &Entity) {
        // we replace the existing local player if it exists, since its name might have changed (from hex or "You" to character name)
        if let Some(mut local) = self.encounter.entities.remove(&self.encounter.local_player) {
            // update local player name, insert back into encounter
            self.encounter.local_player.clone_from(&entity.name);
            update_player_entity(&mut local, entity);
            self.encounter
                .entities
                .insert(self.encounter.local_player.clone(), local);
        } else {
            // cannot find old local player by name, so we look by local player's entity id
            // this can happen when the user started meter late
            let old_local = self
                .encounter
                .entities
                .iter()
                .find(|(_, e)| e.id == entity.id)
                .map(|(key, _)| key.clone());

            // if we find the old local player, we update its name and insert back into encounter
            if let Some(old_local) = old_local {
                let mut new_local = self.encounter.entities[&old_local].clone();
                update_player_entity(&mut new_local, entity);
                self.encounter.entities.remove(&old_local);
                self.encounter.local_player.clone_from(&entity.name);
                self.encounter
                    .entities
                    .insert(self.encounter.local_player.clone(), new_local);
            }
        }
    }

    pub fn on_init_env<E: AppEmitter>(&mut self, entity: Entity, emitter: &E, app: &AppHandle, version: &str) {
        // if not already saved to db, we save again
        if !self.saved && !self.encounter.current_boss_name.is_empty() {
            self.save_to_db(app.to_owned(), version.to_string(), false);
        }

        // replace or insert local player
        if let Some(mut local_player) = self.encounter.entities.remove(&self.encounter.local_player)
        {
            update_player_entity(&mut local_player, &entity);
            self.encounter
                .entities
                .insert(entity.name.clone(), local_player);
        } else {
            let entity = encounter_entity_from_entity(&entity);
            self.encounter.entities.insert(entity.name.clone(), entity);
        }
        self.encounter.local_player = entity.name;

        // remove unrelated entities
        self.encounter.entities.retain(|_, e| {
            e.name == self.encounter.local_player || e.damage_stats.damage_dealt > 0
        });

        emitter.emit("zone-change", "");

        self.soft_reset(false);
    }

    pub fn on_transit<E: AppEmitter>(&mut self, zone_id: u32, emitter: &E) {
        // do not reset on kazeros g2
        // Zones
        // Flickering Land
        // Land of Death and Order
        // Collapsed Diaspero
        if matches!(zone_id, 37544 | 37545 | 37546) {
            if zone_id == 37545 {
                let now = Utc::now().timestamp_millis();
                self.intermission_start = Some(now);
                info!("starting intermission");
                for entity in self
                    .encounter
                    .entities
                    .values_mut()
                    .filter(|e| e.entity_type == EntityType::Player)
                {
                    if let Some(death) = entity
                        .damage_stats
                        .death_info
                        .as_mut()
                        .and_then(|info| info.last_mut())
                    {
                        death.dead_for = Some(now - death.death_time);
                    }
                }
            }

            return;
        }

        emitter.emit("zone-change", "no-toast");

        self.soft_reset(false);
    }

    pub fn on_phase_transition<E: AppEmitter>(&mut self, phase_code: i32, emitter: &E, app: &AppHandle, version: &str) {
        emitter.emit("phase-transition", phase_code);

        match phase_code {
            0 | 2 | 3 | 4 => {
                if !self.encounter.current_boss_name.is_empty() {
                    self.save_to_db(app.to_owned(), version.to_owned(), false);
                    self.saved = true;
                }
                self.resetting = true;
            }
            _ => (),
        }
    }

    // replace local player
    pub fn on_init_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        self.encounter.entities.remove(&self.encounter.local_player);
        self.encounter.local_player.clone_from(&entity.name);
        let mut player = encounter_entity_from_entity(&entity);
        player.current_hp = hp;
        player.max_hp = max_hp;
        self.encounter.entities.insert(player.name.clone(), player);
    }

    // add or update player to encounter
    pub fn on_new_pc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        self.encounter
            .entities
            .entry(entity.name.clone())
            .and_modify(|player| {
                player.id = entity.id;
                player.gear_score = entity.gear_level;
                player.current_hp = hp;
                if max_hp > 0 {
                    player.max_hp = max_hp;
                }
                if entity.character_id > 0 {
                    player.character_id = entity.character_id;
                }
            })
            .or_insert_with(|| {
                let mut player = encounter_entity_from_entity(&entity);
                player.current_hp = hp;
                player.max_hp = max_hp;
                player
            });
    }

    // add or update npc to encounter
    // we set current boss if npc matches criteria
    pub fn on_new_npc(&mut self, entity: Entity, hp: i64, max_hp: i64) {
        let entity_name = entity.name.clone();
        self.encounter
            .entities
            .entry(entity_name.clone())
            .and_modify(|e| {
                if entity.entity_type != EntityType::Boss && e.entity_type != EntityType::Boss {
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                } else if entity.entity_type == EntityType::Boss && e.entity_type == EntityType::Npc
                {
                    e.entity_type = EntityType::Boss;
                    e.npc_id = entity.npc_id;
                    e.id = entity.id;
                    e.current_hp = hp;
                    e.max_hp = max_hp;
                }
            })
            .or_insert_with(|| {
                let mut npc = encounter_entity_from_entity(&entity);
                npc.current_hp = hp;
                npc.max_hp = max_hp;
                npc
            });

        if let Some(npc) = self.encounter.entities.get(&entity_name)
            && npc.entity_type == EntityType::Boss
        {
            // if current encounter has no boss, we set the boss
            // if current encounter has a boss, we check if new boss has more max hp, or if current boss is dead
            self.encounter.current_boss_name = if self
                .encounter
                .entities
                .get(&self.encounter.current_boss_name)
                .is_none_or(|boss| npc.max_hp >= boss.max_hp || boss.is_dead)
            {
                entity_name
            } else {
                self.encounter.current_boss_name.clone()
            };

            // set intermission end if boss is kazeros g2
            if self.encounter.current_boss_name == "Death Incarnate Kazeros"
                && self.intermission_start.is_some()
                && self.intermission_end.is_none()
            {
                self.intermission_end = Some(Utc::now().timestamp_millis());
                info!("ending intermission");
            }
        }
    }

    pub fn on_death(&mut self, dead_entity: &Entity) {
        // get current boss hp
        let boss_hp = self
            .encounter
            .entities
            .get(&self.encounter.current_boss_name)
            .map(|b| b.current_hp)
            .unwrap_or_default();

        let entity = self
            .encounter
            .entities
            .entry(dead_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dead_entity));

        if (dead_entity.entity_type != EntityType::Player
            && dead_entity.entity_type != EntityType::Boss)
            || entity.id != dead_entity.id
            || (entity.entity_type == EntityType::Boss && entity.npc_id != dead_entity.npc_id)
        {
            return;
        }

        if entity.entity_type == EntityType::Boss
            && dead_entity.entity_type == EntityType::Boss
            && entity.name == self.encounter.current_boss_name
            && !entity.is_dead
        {
            self.boss_dead_update = true;
        }

        let now = Utc::now().timestamp_millis();
        entity.current_hp = 0;
        entity.is_dead = true;
        entity.damage_stats.deaths += 1;
        entity.damage_stats.death_time = now;
        entity
            .damage_stats
            .death_info
            .get_or_insert_default()
            .push(DeathInfo {
                death_time: now,
                dead_for: None,
            });
        // record boss hp at time of death
        entity.damage_stats.boss_hp_at_death = Some(boss_hp);

        entity
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|x| x.timestamp + x.duration > entity.damage_stats.death_time)
            .for_each(|x| {
                // cap duration to death time if it exceeds it
                x.duration = x.timestamp - entity.damage_stats.death_time;
            });
    }

    pub fn on_skill_cooldown(&mut self, cooldown_struct: SkillCooldownStruct) {
        let now = Utc::now().timestamp_millis();

        let cooldown_duration = if cooldown_struct.skill_cooldown_stack_data.has_stacks > 0 {
            (cooldown_struct
                .skill_cooldown_stack_data
                .current_stack_cooldown
                .unwrap_or_default()
                * 1000.0) as i64
        } else {
            (cooldown_struct.current_cooldown * 1000.0) as i64
        };

        let cooldowns = self
            .skill_tracker
            .skill_cooldowns
            .entry(cooldown_struct.skill_id)
            .or_default();

        // check if this is a cooldown reduction event (e.g. quick recharge, instant cooldown reduction)
        if let Some(last_event) = cooldowns.last_mut() {
            let last_cooldown_end = last_event.timestamp + last_event.cooldown_duration_ms;

            // if skill is still on cooldown, this is a cooldown reduction
            if now < last_cooldown_end {
                // update the cooldown to end at: current_time + new_duration
                // this means the total cooldown duration from cast time is:
                // (timestamp - last_event.timestamp) + cooldown_duration
                last_event.cooldown_duration_ms = (now - last_event.timestamp) + cooldown_duration;
                return;
            }
        }
        cooldowns.push(CastEvent {
            timestamp: now,
            cooldown_duration_ms: cooldown_duration,
        });

        // info!("skill cooldowns: {cooldowns:#?}");
        // info!(
        //     "total available time for {}: {}ms",
        //     cooldown_struct.skill_id,
        //     self.skill_tracker.calculate_total_available_time(
        //         cooldown_struct.skill_id,
        //         self.encounter.fight_start,
        //         now
        //     )
        // );
    }

    pub fn on_skill_start(
        &mut self,
        source_entity: &Entity,
        skill_id: u32,
        tripod_index: Option<TripodIndex>,
        timestamp: i64,
    ) -> (u32, Option<Vec<u32>>) {
        // do not track skills if encounter not started
        if self.encounter.fight_start == 0 || skill_id == 0 {
            return (0, None);
        }

        let (skill_name, skill_icon, summons, _, is_hyper_awakening) =
            get_skill_name_and_icon(skill_id, 0, &self.skill_tracker, source_entity.id);

        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skills = HashMap::from([(
                    skill_id,
                    Skill {
                        id: skill_id,
                        name: skill_name.clone(),
                        icon: skill_icon.clone(),
                        tripod_index,
                        casts: 0,
                        is_hyper_awakening,
                        ..Default::default()
                    },
                )]);
                entity
            });

        if entity.class_id == 0
            && source_entity.entity_type == EntityType::Player
            && source_entity.class_id > 0
        {
            entity.class_id = source_entity.class_id;
            entity.class = get_class_from_id(&source_entity.class_id);
        }

        if entity.is_dead {
            entity.is_dead = false;
            if let Some(death) = entity
                .damage_stats
                .death_info
                .as_mut()
                .and_then(|info| info.last_mut())
                .filter(|death| death.dead_for.is_none())
            {
                death.dead_for = Some(timestamp - death.death_time);
            }
        }
        entity.skill_stats.casts += 1;

        // if skills have different ids but the same name, we group them together
        let mut skill_id = skill_id;
        if let Some(skill) = entity.skills.get_mut(&skill_id) {
            skill.casts += 1;
            if tripod_index.is_some() {
                skill.tripod_index = tripod_index;
            }
        } else if let Some(skill) = entity
            .skills
            .values_mut()
            .find(|s| s.name == skill_name.clone())
        {
            // no id match found, search skills by name
            skill.casts += 1;
            skill_id = skill.id;
            if tripod_index.is_some() {
                skill.tripod_index = tripod_index;
            }
        } else {
            // no match for id or name
            entity.skills.insert(
                skill_id,
                Skill {
                    id: skill_id,
                    name: skill_name,
                    icon: skill_icon,
                    tripod_index,
                    casts: 1,
                    ..Default::default()
                },
            );
        }

        let relative_timestamp = if self.encounter.fight_start == 0 {
            0
        } else {
            (timestamp - self.encounter.fight_start) as i32
        };

        self.cast_log
            .entry(entity.name.clone())
            .or_default()
            .entry(skill_id)
            .or_default()
            .push(relative_timestamp);

        // if this is a getup skill and we have an ongoing abnormal move incapacitation, this will end it
        if let Some(skill_data) = SKILL_DATA.get(&skill_id)
            && skill_data.skill_type == "getup"
        {
            for ongoing_event in entity
                .damage_stats
                .incapacitations
                .iter_mut()
                .rev()
                .take_while(|x| x.timestamp + x.duration > timestamp)
                .filter(|x| x.event_type == IncapacitationEventType::FALL_DOWN)
            {
                info!(
                    "Shortening down duration from {} to {} because of getup skill",
                    ongoing_event.duration,
                    timestamp - ongoing_event.timestamp
                );
                ongoing_event.duration = timestamp - ongoing_event.timestamp;
            }
        }

        // set spec for supports to determine buff source
        if is_support_class(&entity.class_id) && entity.spec.is_none() {
            let spec = get_player_spec(entity, &self.encounter.encounter_damage_stats.buffs, true);
            if spec != "Unknown" {
                entity.spec = Some(spec);
            }
        }

        (skill_id, summons)
    }

    pub fn on_damage<E: AppEmitter, T: TimeSyncClient>(&mut self, event: DamageEventContext, emitter: &E, sntp_client: &T) {

        let DamageEventContext {
            mut damage,
            stagger,
            skill_id,
            mut skill_effect_id,
            target_current_hp,
            target_max_hp,
            rdps_data,
            dmg_src_entity,
            proj_entity,
            dmg_target_entity,
            se_on_source_ids,
            se_on_target_ids,
            hit_flag,
            hit_option,
            timestamp,
            character_id_to_name
        } = event;

        if hit_flag == HitFlag::Invincible {
            return;
        }
      
        if hit_flag == HitFlag::DamageShare
            && skill_id == 0
            && skill_effect_id == 0
        {
            return;
        }

        let is_battle_item = is_battle_item(&proj_entity.skill_effect_id, "attack");
        if proj_entity.entity_type == EntityType::Projectile && is_battle_item {
            skill_effect_id = proj_entity.skill_effect_id;
        }

        // ensure source entity exists in encounter
        let source_entity = self
            .encounter
            .entities
            .entry(dmg_src_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(dmg_src_entity));
        let source_type = source_entity.entity_type;

        // get skill info here early for stagger tracking
        // since we can stagger mobs that are not bosses that would otherwise be ignored
        let mut skill_key = if is_battle_item {
            // pad battle item skill effect id to avoid overlap with skill ids
            skill_effect_id + 1_000_000
        } else if skill_id == 0 {
            skill_effect_id
        } else {
            skill_id
        };

        let (skill_name, skill_icon, skill_summon_sources, special, is_hyper_awakening) =
            get_skill_name_and_icon(
                skill_id,
                skill_effect_id,
                &self.skill_tracker,
                source_entity.id,
            );

        if !source_entity.skills.contains_key(&skill_key) {
            if let Some(skill) = source_entity
                .skills
                .values()
                .find(|&s| s.name == skill_name)
            {
                skill_key = skill.id;
            } else {
                source_entity.skills.insert(
                    skill_key,
                    Skill {
                        id: skill_key,
                        name: skill_name,
                        icon: skill_icon,
                        casts: 1,
                        ..Default::default()
                    },
                );
                source_entity.skill_stats.casts += 1;
            }
        }

        // add stagger damage here
        source_entity
            .skills
            .entry(skill_key)
            .and_modify(|s| s.stagger += stagger);
        source_entity.damage_stats.stagger += stagger;

        // ensure target entity exists in encounter
        let target_type = self
            .encounter
            .entities
            .entry(dmg_target_entity.name.clone())
            .or_insert_with(|| {
                let mut target_entity = encounter_entity_from_entity(dmg_target_entity);
                target_entity.current_hp = target_current_hp;
                target_entity.max_hp = target_max_hp;
                target_entity
            })
            .entity_type;

        if dmg_src_entity.name == dmg_target_entity.name {
            info!("ignoring self damage from {}", dmg_src_entity.name);
            return;
        }

        // if boss only damage is enabled
        // check if target is boss and not player
        // check if target is player and source is boss
        if self.boss_only_damage
            && ((target_type != EntityType::Boss && target_type != EntityType::Player)
                || (target_type == EntityType::Player && source_type != EntityType::Boss))
        {
            return;
        }

        if self.encounter.fight_start == 0 {
            self.encounter.fight_start = timestamp;
            self.skill_tracker.fight_start = timestamp;
            if target_type == EntityType::Player && skill_id > 0 {
                self.skill_tracker.new_cast(
                    dmg_src_entity.id,
                    skill_id,
                    None,
                    timestamp,
                );
            }

            match self.sntp_client.synchronize("time.cloudflare.com") {
                Ok(result) => {
                    let dt = result.datetime().into_chrono_datetime().unwrap_or_default();
                    self.ntp_fight_start = dt.timestamp_millis();
                    // debug_print(format_args!("fight start local: {}, ntp: {}", Utc::now().to_rfc3339(), dt.to_rfc3339()));
                }
                Err(e) => {
                    warn!("failed to get NTP timestamp: {}", e);
                }
            };

            self.encounter.boss_only_damage = self.boss_only_damage;
            emitter.emit("raid-start", timestamp);
        }

        self.encounter.last_combat_packet = timestamp;

        // apply pseudo rdps contributions
        for entry in rdps_data.iter() {
            // find entity that made this contribution and add to the skill for it.
            let contributor_entity = if let Some(name) = character_id_to_name
                .get(&entry.source_character_id)
            {
                self.encounter.entities.get_mut(name)
            } else {
                self.encounter
                    .entities
                    .values_mut()
                    .find(|entity| entity.character_id == entry.source_character_id)
            };
            if let Some(contributor_entity) = contributor_entity {
                if let Some(contributor_skill) = contributor_entity.skills.get_mut(&entry.skill_id)
                {
                    *contributor_skill
                        .rdps_contributed
                        .entry(entry.rdps_type)
                        .or_default() += entry.value;
                } else if let Some(skill_data) = SKILL_DATA.get(&entry.skill_id)
                    && let Some(skill_name) = skill_data.name.clone()
                    && let Some(contributor_skill) = contributor_entity
                        .skills
                        .values_mut()
                        .find(|s| s.name == skill_name)
                {
                    *contributor_skill
                        .rdps_contributed
                        .entry(entry.rdps_type)
                        .or_default() += entry.value;
                }
            }
        }

        let [Some(source_entity), Some(target_entity)] = self
            .encounter
            .entities
            .get_many_mut([&dmg_src_entity.name, &dmg_target_entity.name])
        else {
            warn!(
                "{}, {} not found in encounter entities",
                dmg_src_entity.name, dmg_target_entity.name
            );
            return;
        };

        source_entity.id = dmg_src_entity.id;

        if target_entity.id == dmg_target_entity.id {
            target_entity.current_hp = target_current_hp;
            target_entity.max_hp = target_max_hp;
        }

        if target_entity.entity_type != EntityType::Player && target_current_hp < 0 {
            damage += target_current_hp;
        }

        let skill = source_entity.skills.get_mut(&skill_key).unwrap();
        skill.is_hyper_awakening = is_hyper_awakening;
        if special {
            skill.special = Some(true);
        }

        let relative_timestamp = (timestamp - self.encounter.fight_start) as i32;
        let mut skill_hit = SkillHit {
            damage,
            stagger,
            timestamp: relative_timestamp as i64,
            ..Default::default()
        };

        skill.total_damage += damage;
        if damage > skill.max_damage {
            skill.max_damage = damage;
        }
        skill.last_timestamp = timestamp;

        source_entity.damage_stats.damage_dealt += damage;

        // apply pseudo rdps damage
        let mut buffed = 0_i64;
        for entry in rdps_data {
            *skill
                .rdps_received
                .entry(entry.rdps_type)
                .or_default()
                .entry(entry.skill_id)
                .or_default() += entry.value;
            if matches!(entry.rdps_type, 1 | 3 | 5) {
                buffed += entry.value;
            }
        }

        source_entity.damage_stats.buffed_damage += buffed;
        source_entity.damage_stats.unbuffed_damage =
            source_entity.damage_stats.damage_dealt - source_entity.damage_stats.buffed_damage;

        if is_hyper_awakening {
            source_entity.damage_stats.hyper_awakening_damage += damage;
        }

        target_entity.damage_stats.damage_taken += damage;

        source_entity.skill_stats.hits += 1;
        skill.hits += 1;

        if hit_flag == HitFlag::Critical || hit_flag == HitFlag::DotCritical {
            source_entity.skill_stats.crits += 1;
            source_entity.damage_stats.crit_damage += damage;
            skill.crits += 1;
            skill.crit_damage += damage;
            skill_hit.crit = true;
        }
        if hit_option == HitOption::BackAttack {
            source_entity.skill_stats.back_attacks += 1;
            source_entity.damage_stats.back_attack_damage += damage;
            skill.back_attacks += 1;
            skill.back_attack_damage += damage;
            skill_hit.back_attack = true;
        }
        if hit_option == HitOption::FrontalAttack {
            source_entity.skill_stats.front_attacks += 1;
            source_entity.damage_stats.front_attack_damage += damage;
            skill.front_attacks += 1;
            skill.front_attack_damage += damage;
            skill_hit.front_attack = true;
        }

        if source_entity.entity_type == EntityType::Player {
            self.encounter.encounter_damage_stats.total_damage_dealt += damage;
            self.encounter.encounter_damage_stats.top_damage_dealt = max(
                self.encounter.encounter_damage_stats.top_damage_dealt,
                source_entity.damage_stats.damage_dealt,
            );

            self.damage_log
                .entry(source_entity.name.clone())
                .or_default()
                .push((timestamp, damage));

            let mut is_buffed_by_support = false;
            let mut is_buffed_by_identity = false;
            let mut is_debuffed_by_support = false;
            let mut is_buffed_by_hat = false;

            if !special {

                for buff_id in se_on_source_ids.iter() {
                    if !self
                        .encounter
                        .encounter_damage_stats
                        .unknown_buffs
                        .contains(buff_id)
                        && !self
                            .encounter
                            .encounter_damage_stats
                            .buffs
                            .contains_key(buff_id)
                    {
                        let mut source_id: Option<u32> = None;
                        let original_buff_id =
                            if let Some(deref_id) = self.custom_id_map.get(buff_id) {
                                source_id = Some(get_skill_id(*buff_id, *deref_id));
                                *deref_id
                            } else {
                                *buff_id
                            };

                        if let Some(status_effect) =
                            get_status_effect_data(original_buff_id, source_id)
                        {
                            self.encounter
                                .encounter_damage_stats
                                .buffs
                                .insert(*buff_id, status_effect);
                        } else {
                            self.encounter
                                .encounter_damage_stats
                                .unknown_buffs
                                .insert(*buff_id);
                        }
                    }

                    // will count dps spec of supports as support buffs until proper spec is determined
                    let hat = is_hat_buff(buff_id) || is_hyper_hat_buff(buff_id);
                    if ((!is_buffed_by_support && !hat) || !is_buffed_by_identity)
                        && let Some(buff) = self.encounter.encounter_damage_stats.buffs.get(buff_id)
                    {
                        if !is_buffed_by_support
                            && !hat
                            && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                            && buff.buff_category == "supportbuff"
                            && SUPPORT_AP_GROUP.contains(&buff.unique_group)
                        {
                            is_buffed_by_support = true;
                        }

                        if !is_buffed_by_identity
                            && buff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                            && buff.buff_category == "supportbuff"
                            && SUPPORT_IDENTITY_GROUP.contains(&buff.unique_group)
                        {
                            is_buffed_by_identity = true;
                        }
                    }

                    // T skill has two buffs, one buffs hyper awakening damage, one buffs all other skill damage
                    // if normal skill, check if normal buff
                    // if hyper awakening, check if hyper buff
                    if !is_buffed_by_hat
                        && ((is_hat_buff(buff_id) && !is_hyper_awakening)
                            || (is_hyper_hat_buff(buff_id) && is_hyper_awakening))
                    {
                        is_buffed_by_hat = true;
                    }
                }

                for debuff_id in se_on_target_ids.iter() {
                    if !self
                        .encounter
                        .encounter_damage_stats
                        .unknown_buffs
                        .contains(debuff_id)
                        && !self
                            .encounter
                            .encounter_damage_stats
                            .debuffs
                            .contains_key(debuff_id)
                    {
                        let mut source_id: Option<u32> = None;
                        let original_debuff_id =
                            if let Some(deref_id) = self.custom_id_map.get(debuff_id) {
                                source_id = Some(get_skill_id(*debuff_id, *deref_id));
                                *deref_id
                            } else {
                                *debuff_id
                            };

                        if let Some(status_effect) =
                            get_status_effect_data(original_debuff_id, source_id)
                        {
                            self.encounter
                                .encounter_damage_stats
                                .debuffs
                                .insert(*debuff_id, status_effect);
                        } else {
                            self.encounter
                                .encounter_damage_stats
                                .unknown_buffs
                                .insert(*debuff_id);
                        }
                    }
                    if !is_debuffed_by_support
                        && let Some(debuff) =
                            self.encounter.encounter_damage_stats.debuffs.get(debuff_id)
                    {
                        is_debuffed_by_support = debuff.unique_group == 210230 // brand group
                                && debuff.buff_type & StatusEffectBuffTypeFlags::DMG.bits() != 0
                                && debuff.target == StatusEffectTarget::PARTY;
                    }
                }

                if is_buffed_by_support && !is_hyper_awakening {
                    skill.buffed_by_support += damage;
                    source_entity.damage_stats.buffed_by_support += damage;
                }
                if is_buffed_by_identity && !is_hyper_awakening {
                    skill.buffed_by_identity += damage;
                    source_entity.damage_stats.buffed_by_identity += damage;
                }
                if is_debuffed_by_support && !is_hyper_awakening {
                    skill.debuffed_by_support += damage;
                    source_entity.damage_stats.debuffed_by_support += damage;
                }
                if is_buffed_by_hat {
                    skill.buffed_by_hat += damage;
                    source_entity.damage_stats.buffed_by_hat += damage;
                }

                let stabilized_status_active =
                    (source_entity.current_hp as f64 / source_entity.max_hp as f64) > 0.65;
                let mut filtered_se_on_source_ids: Vec<u32> = vec![];

                for buff_id in se_on_source_ids.iter() {
                    // hyper only affected by hat buff
                    if is_hyper_awakening && !is_hyper_hat_buff(buff_id) {
                        continue;
                    } else if let Some(buff) =
                        self.encounter.encounter_damage_stats.buffs.get(buff_id)
                        && !stabilized_status_active
                        && buff.source.name.contains("Stabilized Status")
                    {
                        continue;
                    }

                    filtered_se_on_source_ids.push(*buff_id);

                    skill
                        .buffed_by
                        .entry(*buff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                    source_entity
                        .damage_stats
                        .buffed_by
                        .entry(*buff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                }
                for debuff_id in se_on_target_ids.iter() {
                    if is_hyper_awakening {
                        break;
                    }

                    skill
                        .debuffed_by
                        .entry(*debuff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                    source_entity
                        .damage_stats
                        .debuffed_by
                        .entry(*debuff_id)
                        .and_modify(|e| *e += damage)
                        .or_insert(damage);
                }

                skill_hit.buffed_by = filtered_se_on_source_ids;
                // no debuffs affect hyper
                if !is_hyper_awakening {
                    skill_hit.debuffed_by = se_on_target_ids;
                }
            }
        }

        if target_entity.entity_type == EntityType::Player {
            self.encounter.encounter_damage_stats.total_damage_taken += damage;
            self.encounter.encounter_damage_stats.top_damage_taken = max(
                self.encounter.encounter_damage_stats.top_damage_taken,
                target_entity.damage_stats.damage_taken,
            );
        }
        // update current_boss
        else if target_entity.entity_type == EntityType::Boss {
            self.encounter
                .current_boss_name
                .clone_from(&target_entity.name);
            target_entity.id = dmg_target_entity.id;
            target_entity.npc_id = dmg_target_entity.npc_id;

            let log = self
                .boss_hp_log
                .entry(target_entity.name.clone())
                .or_default();

            let current_hp = if target_entity.current_hp >= 0 {
                target_entity.current_hp + target_entity.current_shield as i64
            } else {
                0
            };
            let hp_percent = if target_entity.max_hp != 0 {
                current_hp as f32 / target_entity.max_hp as f32
            } else {
                0.0
            };

            let relative_timestamp_s = relative_timestamp / 1000;

            if log.is_empty() || log.last().unwrap().time != relative_timestamp_s {
                log.push(BossHpLog::new(relative_timestamp_s, current_hp, hp_percent));
            } else {
                let last = log.last_mut().unwrap();
                last.hp = current_hp;
                last.p = hp_percent;
            }
        }

        if skill_key > 0 {
            self.skill_tracker.on_hit(
                source_entity.id,
                proj_entity.id,
                skill_key,
                skill_hit,
                skill_summon_sources,
            );
        }
    }

    pub fn on_counterattack(&mut self, source_entity: &Entity) {
        let entity = self
            .encounter
            .entities
            .entry(source_entity.name.clone())
            .or_insert_with(|| {
                let mut entity = encounter_entity_from_entity(source_entity);
                entity.skill_stats = SkillStats {
                    counters: 0,
                    ..Default::default()
                };
                entity
            });
        entity.skill_stats.counters += 1;
    }

    pub fn on_abnormal_move(
        &mut self,
        victim_entity: &Entity,
        movement: &SkillMoveOptionData,
        timestamp: i64,
    ) {
        if victim_entity.entity_type != EntityType::Player {
            // we don't care about npc knockups
            return;
        }

        // only count movement events that would result in a knockup
        let Some(down_time) = movement.down_time else {
            return;
        };

        // todo: unclear if this is fully correct. It's hard to debug this, but it seems roughly accurate
        // if this is not accurate, we should probably factor out the stand_up_time and instead add in the
        // animation duration of the standup action for each class (seems to be 0.9s)
        let total_incapacitated_time = down_time
            + movement.move_time.unwrap_or_default()
            + movement.stand_up_time.unwrap_or_default();
        let incapacitated_time_ms = (total_incapacitated_time * 1000.0) as i64;

        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // see if we have a previous incapacitation event that is still in effect (i.e. the player was knocked up again before
        // they could stand up), in which case we should shorten the previous event duration to the current timestamp
        let prev_incapacitation = victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp) // stop as soon as we only hit expired events
            .find(|x| x.event_type == IncapacitationEventType::FALL_DOWN); // find an unexpired one that was caused by an abnormal move
        if let Some(prev_incapacitation) = prev_incapacitation {
            info!(
                "Shortening down duration from {} to {} because of new abnormal move",
                prev_incapacitation.duration,
                timestamp - prev_incapacitation.timestamp
            );
            prev_incapacitation.duration = timestamp - prev_incapacitation.timestamp;
        }

        let new_event = IncapacitatedEvent {
            timestamp,
            duration: incapacitated_time_ms,
            event_type: IncapacitationEventType::FALL_DOWN,
        };
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
        info!(
            "Player {} will be incapacitated for {}ms",
            victim_entity_state.name, incapacitated_time_ms
        );
    }

    pub fn on_cc_applied(&mut self, victim_entity: &Entity, status_effect: &StatusEffectDetails) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        // track number of crowd control effects applied
        *self
            .crowd_control_tracker
            .entry(status_effect.status_effect_id)
            .or_insert(0) += 1;

        // expiration delay is zero or negative for infinite effects. Instead of applying them now,
        // only apply them after they've been removed (this avoids an issue where if we miss the removal
        // we end up applying a very long incapacitation)
        if status_effect.is_infinite() {
            return;
        }

        let duration_ms = status_effect.expiration_delay * 1000.0;
        let new_event = IncapacitatedEvent {
            timestamp: status_effect.timestamp.timestamp_millis(),
            duration: duration_ms as i64,
            event_type: IncapacitationEventType::CROWD_CONTROL,
        };
        info!(
            "Player {} will be status-effect incapacitated for {}ms by buff {}",
            victim_entity_state.name, duration_ms, status_effect.status_effect_id
        );
        victim_entity_state
            .damage_stats
            .incapacitations
            .push(new_event);
    }

    pub fn on_cc_removed(
        &mut self,
        victim_entity: &Entity,
        status_effect: &StatusEffectDetails,
        timestamp: i64,
    ) {
        let victim_entity_state = self
            .encounter
            .entities
            .entry(victim_entity.name.clone())
            .or_insert_with(|| encounter_entity_from_entity(victim_entity));

        if status_effect.is_infinite() {
            // this status effect was infinite, meaning we didn't apply it on_cc_applied
            // apply it now retroactively, then sort the events to ensure that our sorted
            // invariant does not get violated
            let duration_ms = timestamp - status_effect.timestamp.timestamp_millis();
            let new_event = IncapacitatedEvent {
                timestamp: status_effect.timestamp.timestamp_millis(),
                duration: duration_ms,
                event_type: IncapacitationEventType::CROWD_CONTROL,
            };
            info!(
                "Player {} was incapacitated by an infinite status effect buff for {}ms",
                victim_entity_state.name, duration_ms
            );
            victim_entity_state
                .damage_stats
                .incapacitations
                .push(new_event);
            victim_entity_state
                .damage_stats
                .incapacitations
                .sort_by_key(|x| x.timestamp);
            return;
        }

        // we use the application timestamp as the key. Attempt to find all buff instances that started
        // at this time and cap their duration to the current timestamp
        for event in victim_entity_state
            .damage_stats
            .incapacitations
            .iter_mut()
            .rev()
            .take_while(|e| e.timestamp + e.duration > timestamp)
        {
            if event.event_type == IncapacitationEventType::CROWD_CONTROL
                && event.timestamp == status_effect.timestamp.timestamp_millis()
            {
                info!(
                    "Removing status-effect {} incapacitation for player {} (shortened {}ms to {}ms)",
                    status_effect.status_effect_id,
                    victim_entity_state.name,
                    event.duration,
                    timestamp - event.timestamp
                );
                event.duration = timestamp - event.timestamp;
            }
        }
    }

    // pub fn on_identity_gain(&mut self, pkt: &PKTIdentityGaugeChangeNotify) {
    //     if self.encounter.fight_start == 0 {
    //         return;
    //     }
    //
    //     if self.encounter.local_player.is_empty() {
    //         if let Some((_, entity)) = self
    //             .encounter
    //             .entities
    //             .iter()
    //             .find(|(_, e)| e.id == pkt.player_id)
    //         {
    //             self.encounter.local_player.clone_from(&entity.name);
    //         } else {
    //             return;
    //         }
    //     }
    //
    //     if let Some(entity) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.local_player)
    //     {
    //         self.identity_log
    //             .entry(entity.name.clone())
    //             .or_default()
    //             .push((
    //                 Utc::now().timestamp_millis(),
    //                 (
    //                     pkt.identity_gauge1,
    //                     pkt.identity_gauge2,
    //                     pkt.identity_gauge3,
    //                 ),
    //             ));
    //     }
    // }

    // pub fn on_stagger_change(&mut self, pkt: &PKTParalyzationStateNotify) {
    //     if self.encounter.current_boss_name.is_empty() || self.encounter.fight_start == 0 {
    //         return;
    //     }

    //     if let Some(boss) = self
    //         .encounter
    //         .entities
    //         .get_mut(&self.encounter.current_boss_name)
    //     {
    //         let timestamp = Utc::now().timestamp_millis();
    //         let current_stagger = pkt.paralyzation_point as i32;
    //         let max_stagger = pkt.paralyzation_max_point as i32;
    //         if boss.id == pkt.object_id {
    //             if current_stagger == max_stagger {
    //                 let staggered_in =
    //                     (timestamp - self.encounter.encounter_damage_stats.stagger_start) / 1000;
    //                 self.stagger_intervals
    //                     .push((staggered_in as i32, max_stagger))
    //             } else if current_stagger != 0 && self.prev_stagger == 0 {
    //                 self.encounter.encounter_damage_stats.stagger_start = timestamp;
    //             }

    //             self.prev_stagger = current_stagger;

    //             let relative_timestamp_s = ((timestamp - self.encounter.fight_start) / 1000) as i32;
    //             let stagger_percent = (1.0 - (current_stagger as f32 / max_stagger as f32)) * 100.0;
    //             if let Some(last) = self.stagger_log.last_mut() {
    //                 if last.0 == relative_timestamp_s {
    //                     last.1 = stagger_percent;
    //                 } else {
    //                     self.stagger_log
    //                         .push((relative_timestamp_s, stagger_percent));
    //                 }
    //             } else {
    //                 self.stagger_log
    //                     .push((relative_timestamp_s, stagger_percent));
    //             }

    //             if max_stagger > self.encounter.encounter_damage_stats.max_stagger {
    //                 self.encounter.encounter_damage_stats.max_stagger = max_stagger;
    //             }
    //         }
    //     }
    // }

    pub fn on_boss_shield(&mut self, target_entity: &Entity, shield: u64) {
        if target_entity.entity_type == EntityType::Boss
            && target_entity.name == self.encounter.current_boss_name
        {
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .and_modify(|e| {
                    e.current_shield = shield;
                });
        }
    }

    pub fn on_shield_applied(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield: u64,
    ) {
        if source_entity.entity_type == EntityType::Player
            && target_entity.entity_type == EntityType::Player
        {
            if !self
                .encounter
                .encounter_damage_stats
                .applied_shield_buffs
                .contains_key(&buff_id)
            {
                let mut source_id: Option<u32> = None;
                let original_buff_id = if let Some(deref_id) = self.custom_id_map.get(&buff_id) {
                    source_id = Some(get_skill_id(buff_id, *deref_id));
                    *deref_id
                } else {
                    buff_id
                };

                if let Some(status_effect) = get_status_effect_data(original_buff_id, source_id) {
                    self.encounter
                        .encounter_damage_stats
                        .applied_shield_buffs
                        .insert(buff_id, status_effect);
                }
            }

            self.encounter.encounter_damage_stats.total_shielding += shield;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.shields_received += shield;
                source_entity_state.damage_stats.shields_given += shield;
                source_entity_state
                    .damage_stats
                    .shields_given_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);
                source_entity_state
                    .damage_stats
                    .shields_received_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield)
                    .or_insert(shield);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_many_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.shields_received += shield;
            source_entity_state.damage_stats.shields_given += shield;
            source_entity_state
                .damage_stats
                .shields_given_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
            target_entity_state
                .damage_stats
                .shields_received_by
                .entry(buff_id)
                .and_modify(|e| *e += shield)
                .or_insert(shield);
        }
    }

    pub fn on_shield_used(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        buff_id: u32,
        shield_removed: u64,
    ) {
        if source_entity.entity_type == EntityType::Player
            && target_entity.entity_type == EntityType::Player
        {
            self.encounter
                .encounter_damage_stats
                .total_effective_shielding += shield_removed;

            let source_entity_state = self
                .encounter
                .entities
                .entry(source_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(source_entity));

            // shields on self
            if source_entity.id == target_entity.id || source_entity.name == target_entity.name {
                source_entity_state.damage_stats.damage_absorbed += shield_removed;
                source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
                source_entity_state
                    .damage_stats
                    .damage_absorbed_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);
                source_entity_state
                    .damage_stats
                    .damage_absorbed_on_others_by
                    .entry(buff_id)
                    .and_modify(|e| *e += shield_removed)
                    .or_insert(shield_removed);

                return;
            }

            // shields on others
            self.encounter
                .entities
                .entry(target_entity.name.clone())
                .or_insert_with(|| encounter_entity_from_entity(target_entity));

            let [Some(source_entity_state), Some(target_entity_state)] = self
                .encounter
                .entities
                .get_many_mut([&source_entity.name, &target_entity.name])
            else {
                warn!(
                    "{}, {} not found in encounter entities",
                    source_entity.name, target_entity.name
                );
                return;
            };

            target_entity_state.damage_stats.damage_absorbed += shield_removed;
            source_entity_state.damage_stats.damage_absorbed_on_others += shield_removed;
            target_entity_state
                .damage_stats
                .damage_absorbed_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
            source_entity_state
                .damage_stats
                .damage_absorbed_on_others_by
                .entry(buff_id)
                .and_modify(|e| *e += shield_removed)
                .or_insert(shield_removed);
        }
    }

    // track battle items used in an encounter
    pub fn on_battle_item_use(&mut self, battle_item_id: &u32) {
        if self.encounter.fight_start == 0 {
            return;
        }

        self.battle_item_tracker
            .entry(*battle_item_id)
            .and_modify(|e| *e += 1)
            .or_insert(1);
    }

    pub fn save_to_db(&mut self, app: AppHandle, meter_version: String, manual: bool) {
        if !manual
            && (self.encounter.fight_start == 0
                || self.encounter.current_boss_name.is_empty()
                || !self
                    .encounter
                    .entities
                    .contains_key(&self.encounter.current_boss_name)
                || !self.encounter.entities.values().any(|e| {
                    e.entity_type == EntityType::Player && e.damage_stats.damage_dealt > 0
                }))
        {
            info!("not saving to db, no players with damage dealt");
            return;
        }

        if !self.damage_is_valid {
            warn!("damage decryption is invalid, not saving to db");
        }

        let mut encounter = self.encounter.clone();

        let damage_log = self.damage_log.clone();
        let cast_log = self.cast_log.clone();
        let boss_hp_log = self.boss_hp_log.clone();
        let raid_clear = self.raid_clear;
        encounter.cleared = raid_clear;
        let party_info = self.party_info.clone();
        let raid_difficulty = self.raid_difficulty.clone();
        encounter.difficulty = raid_difficulty.clone().into();
        let region = self.region.clone();

        let ntp_fight_start = self.ntp_fight_start;

        let rdps_valid = self.rdps_valid;

        let skill_cast_log = self.skill_tracker.get_cast_log();
        let skill_cooldowns = self.skill_tracker.skill_cooldowns.clone();
        let battle_items = self.battle_item_tracker.clone();
        let cc_tracker = self.crowd_control_tracker.clone();
        let intermission_start = self.intermission_start;
        let intermission_end = self.intermission_end;

        // debug_print(format_args!("skill cast log:\n{}", serde_json::to_string(&skill_cast_log).unwrap()));

        // debug_print(format_args!("rdps_data valid: [{}]", rdps_valid));
        info!(
            "saving to db - cleared: [{}], difficulty: [{}] {}",
            raid_clear, self.raid_difficulty, encounter.current_boss_name
        );

        encounter.current_boss_name = update_current_boss_name(&encounter.current_boss_name);

        task::spawn(async move {
            let mut player_info = None;
            let stats_api = app.state::<StatsApi>();

            player_info =
                if let Some(args) = GetCharacterInfoArgs::new(&encounter, &raid_difficulty) {
                    info!("fetching player info");

                    if let Some(args) = SendRaidAnalyticsArgs::new(
                        &encounter,
                        &raid_difficulty,
                        battle_items,
                        cc_tracker,
                    ) {
                        stats_api.send_raid_analytics(args).await;
                    }

                    stats_api.get_character_info(args).await
                } else {
                    None
                };

            let repository = app.state::<Repository>();

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
                skill_cooldowns,
                intermission_start,
                intermission_end,
            };

            let encounter_id = repository
                .insert_data(args)
                .expect("could not save encounter");

            info!("saved to db");

            if raid_clear {
                app.emit("clear-encounter", encounter_id)
                    .expect("failed to emit clear-encounter");
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use mockall::predicate::{eq, always};

    use crate::live::{emitter::MockAppEmitter, sntp::MockTimeSyncClient};

    use super::*;

    fn setup_encounter_state() -> EncounterState {
        let current_dir = std::env::current_dir().unwrap();
        AssetPreloader::new(&current_dir).unwrap();

        EncounterState::new()
    }

    fn make_player_entity() -> Entity {
        Entity {
            id: 1,
            character_id: 1,
            class_id: 102,
            entity_type: EntityType::Player,
            name: "Player".into(),
            ..Default::default()
        }
    }

    fn make_boss_entity() -> Entity {
        Entity {
            id: 2,
            entity_type: EntityType::Boss,
            npc_id: 486001,
            name: "Armoche, Sentinel of the Abyss".into(),
            ..Default::default()
        }
    }

    #[test]
    fn should_set_intermission() {
        let mut state = setup_encounter_state();
        let emitter = MockAppEmitter::new();
        
        let entity = Entity {
            id: 1,
            entity_type: EntityType::Boss,
            npc_id: 486301,
            name: "Death Incarnate Kazeros".into(),
            ..Default::default()
        };
        
        state.on_transit(37545, &emitter);
        state.on_new_npc(entity, 1e16 as i64, 1e16 as i64);

        assert!(state.intermission_start.is_some());
        assert!(state.intermission_end.is_some());
    }

    #[test]
    fn should_update_damage_stats() {
        let mut state = setup_encounter_state();

        let source_entity = make_player_entity();
        let target_entity = make_boss_entity();

        let mut sntp_client = MockTimeSyncClient::new();
        let mut emitter = MockAppEmitter::new();

        sntp_client
            .expect_synchronize()
            .times(1)
            .return_const(1);

        emitter
            .expect_emit::<i64>()
            .with(eq("raid-start"), always())
            .times(1)
            .return_const(());

        let damage = 100_000_000;

        let event = DamageEventContext {
            damage,
            stagger: 1,
            skill_id: 16140,
            skill_effect_id: 0,
            target_current_hp: damage,
            target_max_hp: damage,
            rdps_data: vec![],
            dmg_src_entity: &source_entity,
            proj_entity: &source_entity,
            dmg_target_entity: &target_entity,
            se_on_source_ids: vec![],
            se_on_target_ids: vec![],
            hit_flag: HitFlag::Normal,
            hit_option: HitOption::FlankAttack,
            timestamp: Utc::now().timestamp_millis(),
            character_id_to_name: &HashMap::from([(1u64, String::from(&source_entity.name))]),
        };

        state.on_damage(event, &emitter, &sntp_client);

        let player = &state.encounter.entities[&source_entity.name];

        assert_eq!(player.damage_stats.damage_dealt, damage);
        assert_eq!(state.encounter.encounter_damage_stats.total_damage_dealt, damage);
        assert_eq!(state.encounter.encounter_damage_stats.top_damage_dealt, damage);
        assert_eq!(player.skills[&16140].total_damage, damage);

        let boss = &state.encounter.entities[&target_entity.name];

        assert_eq!(boss.damage_stats.damage_taken, damage);
    }

    #[test]
    fn should_update_buffed_stats() {
        let mut state = setup_encounter_state();

        let source_entity = make_player_entity();
        let target_entity = make_boss_entity();

        let damage = 100_000_000;

        let mut sntp_client = MockTimeSyncClient::new();
        let mut emitter = MockAppEmitter::new();

        sntp_client
            .expect_synchronize()
            .times(1)
            .return_const(1);

        emitter
            .expect_emit::<i64>()
            .with(eq("raid-start"), always())
            .times(1)
            .return_const(());

        let rdps_data = vec![
            RdpsData {
                rdps_type: 1,
                skill_id: 16140,
                source_character_id: 1,
                value: damage,
            }
        ];


        // LWC
        let card_debuff_id = 610001002;
        // Bard
        let atk_buff_id = 211606;
        let identity_buff_id = 211420;
        let hat_buff_id = 212305;
        // "Note Brand"
        let brand_buff_id = 210230;
        let brand_skill_id = 21020;
        let custom_id  = get_new_id(brand_skill_id + brand_buff_id);
        state.custom_id_map.insert(custom_id, brand_buff_id);

        let event = DamageEventContext {
            damage,
            stagger: 1,
            skill_id: 16140,
            skill_effect_id: 0,
            target_current_hp: damage,
            target_max_hp: damage,
            rdps_data,
            dmg_src_entity: &source_entity,
            proj_entity: &source_entity,
            dmg_target_entity: &target_entity,
            se_on_source_ids: vec![atk_buff_id, identity_buff_id, hat_buff_id],
            se_on_target_ids: vec![card_debuff_id, custom_id],
            hit_flag: HitFlag::Critical,
            hit_option: HitOption::FlankAttack,
            timestamp: Utc::now().timestamp_millis(),
            character_id_to_name: &HashMap::from([(1u64, String::from(&source_entity.name))]),
        };

        state.on_damage(event, &emitter, &sntp_client);

        let player = &state.encounter.entities[&source_entity.name];

        assert_eq!(player.damage_stats.buffed_by[&atk_buff_id], damage);
        assert_eq!(player.damage_stats.buffed_damage, damage);
        assert_eq!(player.damage_stats.unbuffed_damage, 0);
        assert_eq!(player.damage_stats.crit_damage, damage);
        assert_eq!(player.damage_stats.debuffed_by[&card_debuff_id], damage);
        assert_eq!(player.damage_stats.debuffed_by[&custom_id], damage);
        assert_eq!(player.damage_stats.debuffed_by_support, damage);
        assert_eq!(player.damage_stats.buffed_by_support, damage);
        assert_eq!(player.damage_stats.buffed_by_identity, damage);
        assert_eq!(player.damage_stats.buffed_by_hat, damage);
    }
}