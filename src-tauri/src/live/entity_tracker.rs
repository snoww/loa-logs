use crate::data::*;
use crate::live::id_tracker::IdTracker;
use crate::live::inspect_stats::{InspectDerivedStats, derive_inspect_stats};
use crate::live::party_tracker::PartyTracker;
use crate::live::player_stats::RuntimeState;
use crate::live::rdps::snapshot_owner_player_stats_for_buffs;
use crate::live::status_tracker::{
    StatusEffectDetails, StatusEffectTargetType, StatusEffectType, StatusTracker,
    build_status_effect,
};
use crate::local::{LocalInfo, LocalPlayer};
use crate::models::EntityType::*;
use crate::models::{
    ArkPassiveData, ArkPassiveNode, EncounterEntity, EntityType, Esther, InspectInfo, TripodIndex,
    TripodLevel,
};
use chrono::{DateTime, Utc};
use hashbrown::{HashMap, HashSet};
use log::{info, warn};
use meter_defs::defs::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

const BOOTSTRAP_INSPECT_DELAY_MS: i64 = 500;
const DESTROYER_CLASS_ID: u32 = 103;
const DESTROYER_VORTEX_GRAVITY_SKILL_ID: u32 = 18011;
const DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID: u32 = 18030;
const RECENT_DESTROYER_SKILL_START_WINDOW_MS: i64 = 4_000;

pub struct EntityTracker {
    id_tracker: Rc<RefCell<IdTracker>>,
    party_tracker: Rc<RefCell<PartyTracker>>,
    status_tracker: Rc<RefCell<StatusTracker>>,

    pub entities: HashMap<u64, Entity>,
    pending_inspect_results_by_name: HashMap<String, PKTPCInspectResult>,
    inspect_requested_names: HashSet<String>,
    forced_refresh_names: HashSet<String>,
    bootstrap_refresh_sent_names: HashSet<String>,
    bootstrap_visible_fallback_requested: bool,
    bootstrap_visible_fallback_names: Vec<String>,
    next_bootstrap_inspect_at_ms: i64,
    last_reconnect_rebind: Option<(u64, u64)>,
    removed_player_entities_by_character_id: HashMap<u64, Entity>,
    removed_player_entities_by_name_class: HashMap<(String, u32), Entity>,

    pub local_entity_id: u64,
    pub local_character_id: u64,
    pub character_id_to_name: HashMap<u64, String>,
    status_effect_owner_round_robin: HashMap<(u32, bool, u32), usize>,
}

pub struct AppliedInspectResult {
    pub name: String,
    pub info: InspectInfo,
}

#[derive(Debug, Default, Clone)]
pub struct InspectSnapshot {
    pub gear_level: f32,
    pub stat_pairs: HashMap<u8, i64>,
    pub derived_stats: InspectDerivedStats,
    pub addon_values: Vec<InspectAddonValue>,
    pub engravings: Vec<InspectEngraving>,
    pub equipped_items: Vec<InspectItemSnapshot>,
    pub equipped_gems: Vec<InspectItemSnapshot>,
    pub cards: Vec<InspectCardSnapshot>,
    pub stigma_layouts: Vec<InspectStigmaLayoutSnapshot>,
    pub ark_grid_cores: Vec<InspectArkGridCoreSnapshot>,
    pub ark_passive_data: Option<ArkPassiveData>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectAddonValue {
    pub addon_type: u8,
    pub value: u32,
}

#[derive(Debug, Default, Clone)]
pub struct InspectEngraving {
    pub id: u32,
    pub unknown: u16,
    pub level: u16,
}

#[derive(Debug, Default, Clone)]
pub struct InspectItemSnapshot {
    pub unique_id: Option<u64>,
    pub raw_item_id: Option<u32>,
    pub raw_hone_level: Option<u16>,
    pub raw_slot_index: Option<u16>,
    pub data_type: Option<u8>,
    pub has_equippable_item_data: bool,
    pub has_ark_grid_gem_data: bool,
}

#[derive(Debug, Default, Clone)]
pub struct InspectCardSnapshot {
    pub id: u32,
    pub awakening_level: u32,
}

#[derive(Debug, Default, Clone)]
pub struct InspectStigmaLayoutSnapshot {
    pub stigma_id: u32,
    pub stigma_level: u32,
    pub stigma_rank: u32,
}

#[derive(Debug, Default, Clone)]
pub struct InspectArkGridCoreSnapshot {
    pub core_id: u32,
    pub base_id: u32,
    pub options: Vec<InspectArkGridCoreOptionSnapshot>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectArkGridCoreOptionSnapshot {
    pub willpower_rank: u32,
    pub item_id: u32,
    pub enabled: bool,
    pub order_rank: u32,
    pub slot_index: u32,
    pub values: Vec<InspectArkGridCoreValueSnapshot>,
}

#[derive(Debug, Default, Clone)]
pub struct InspectArkGridCoreValueSnapshot {
    pub option_id: u32,
    pub rank: u32,
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct SkillOptionSnapshot {
    pub layer_index: Option<u8>,
    pub start_stage_index: Option<u8>,
    pub transit_index: Option<u32>,
    pub stage_start_time: Option<u32>,
    pub farmost_dist: Option<f32>,
    pub tripod_index: Option<TripodIndex>,
    pub tripod_level: Option<TripodLevel>,
}

#[derive(Debug, Default, Clone)]
pub struct SkillRuntimeData {
    pub skill_level: u8,
    pub skill_option_data: Option<SkillOptionSnapshot>,
    pub last_cast_at_ms: Option<i64>,
    pub last_start_at_ms: Option<i64>,
    pub identity_gauge1_at_start: Option<u32>,
    pub identity_gauge2_at_start: Option<u32>,
    pub identity_gauge3_at_start: Option<u32>,
    pub cached_critical_hit_damage_bonus: f64,
    pub cached_critical_rate_bonus: f64,
    pub cached_attack_speed_bonus: f64,
    pub cached_critical_hit_damage_bonus_per_skill_effect: HashMap<u32, f64>,
    pub cached_critical_rate_bonus_per_skill_effect: HashMap<u32, f64>,
    pub cached_directional_mask: Option<i32>,
    pub cached_identity_category: Option<String>,
    pub buff_stat_changes: HashMap<u32, HashMap<String, (i64, bool)>>,
    pub buff_param_changes: HashMap<u32, (Vec<i64>, bool)>,
    pub buff_added_stats: HashMap<u32, Vec<crate::models::PassiveOption>>,
    pub added_chain_combat_effects: HashMap<u32, Vec<u32>>,
    pub removed_chain_combat_effects: Vec<u32>,
    pub changed_combat_effects: HashMap<u32, Vec<ChangedCombatEffect>>,
    pub addon_skill_feature_ids: Vec<u32>,
}

#[derive(Debug, Default, Clone)]
pub struct ChangedCombatEffect {
    pub combat_effect_id: u32,
    pub values: Vec<i64>,
    pub relative: bool,
}

impl EntityTracker {
    pub fn new(
        status_tracker: Rc<RefCell<StatusTracker>>,
        id_tracker: Rc<RefCell<IdTracker>>,
        party_tracker: Rc<RefCell<PartyTracker>>,
    ) -> Self {
        Self {
            status_tracker,
            id_tracker,
            party_tracker,
            entities: HashMap::new(),
            pending_inspect_results_by_name: HashMap::new(),
            inspect_requested_names: HashSet::new(),
            forced_refresh_names: HashSet::new(),
            bootstrap_refresh_sent_names: HashSet::new(),
            bootstrap_visible_fallback_requested: false,
            bootstrap_visible_fallback_names: Vec::new(),
            next_bootstrap_inspect_at_ms: 0,
            last_reconnect_rebind: None,
            removed_player_entities_by_character_id: HashMap::new(),
            removed_player_entities_by_name_class: HashMap::new(),
            local_entity_id: 0,
            local_character_id: 0,
            character_id_to_name: HashMap::new(),
            status_effect_owner_round_robin: HashMap::new(),
        }
    }

    pub fn init_env(&mut self, pkt: PKTInitEnv) -> Entity {
        if self.local_entity_id != 0 {
            let party_id = self
                .party_tracker
                .borrow_mut()
                .entity_id_to_party_id
                .get(&self.local_entity_id)
                .cloned();
            if let Some(party_id) = party_id {
                self.party_tracker
                    .borrow_mut()
                    .entity_id_to_party_id
                    .remove(&self.local_entity_id);
                self.party_tracker
                    .borrow_mut()
                    .entity_id_to_party_id
                    .insert(pkt.player_id, party_id);
            }
        }

        let mut local_player = self
            .entities
            .get(&self.local_entity_id)
            .cloned()
            .unwrap_or_else(|| Entity {
                entity_type: Player,
                name: "You".to_string(),
                class_id: 0,
                gear_level: 0.0,
                character_id: self.local_character_id,
                ..Default::default()
            });

        info!("init env: eid: {}->{}", self.local_entity_id, pkt.player_id);

        local_player.id = pkt.player_id;
        local_player.inspect_requested = false;
        local_player.inspect_ready_at_ms = 0;
        local_player.inspect_stale = false;
        local_player.inspect_info = None;
        local_player.inspect_result = None;
        local_player.inspect_snapshot = None;
        local_player.skill_runtime_data.clear();
        self.local_entity_id = pkt.player_id;

        self.entities.clear();
        self.pending_inspect_results_by_name.clear();
        self.inspect_requested_names.clear();
        self.forced_refresh_names.clear();
        self.bootstrap_refresh_sent_names.clear();
        self.bootstrap_visible_fallback_requested = false;
        self.bootstrap_visible_fallback_names.clear();
        self.next_bootstrap_inspect_at_ms = 0;
        self.last_reconnect_rebind = None;
        self.removed_player_entities_by_character_id.clear();
        self.removed_player_entities_by_name_class.clear();
        self.status_effect_owner_round_robin.clear();
        self.party_tracker.borrow_mut().reset_party_mappings();
        self.entities.insert(local_player.id, local_player.clone());
        self.character_id_to_name.clear();
        self.id_tracker.borrow_mut().clear();
        self.status_tracker.borrow_mut().clear();
        if local_player.character_id > 0 {
            self.id_tracker
                .borrow_mut()
                .add_mapping(local_player.character_id, local_player.id);
            if is_resolved_player_name(&local_player.name) {
                self.character_id_to_name
                    .insert(local_player.character_id, local_player.name.clone());
                self.party_tracker
                    .borrow_mut()
                    .set_name(local_player.name.clone());
            }
            self.party_tracker
                .borrow_mut()
                .complete_entry(local_player.character_id, local_player.id);
        }
        local_player
    }

    pub fn init_pc(&mut self, pkt: PKTInitPC) -> Entity {
        let player = Entity {
            id: pkt.player_id,
            entity_type: Player,
            name: pkt.name,
            class_id: pkt.class_id as u32,
            gear_level: truncate_gear_level(pkt.gear_level),
            character_id: pkt.character_id,
            ..Default::default()
        };

        self.local_entity_id = player.id;
        self.local_character_id = player.character_id;
        self.entities.clear();
        self.entities.insert(player.id, player.clone());
        self.id_tracker
            .borrow_mut()
            .add_mapping(player.character_id, player.id);
        self.party_tracker
            .borrow_mut()
            .set_name(player.name.clone());
        self.party_tracker
            .borrow_mut()
            .complete_entry(player.character_id, player.id);
        self.status_tracker
            .borrow_mut()
            .remove_local_object(player.id);
        self.character_id_to_name
            .insert(player.character_id, player.name.clone());
        self.build_and_register_status_effects(pkt.status_effect_datas, player.id);
        player
    }

    fn find_reconnect_entity_id(&self, name: &str, class_id: u32, entity_id: u64) -> Option<u64> {
        self.entities
            .values()
            .find(|entity| {
                entity.id != entity_id
                    && entity.entity_type == Player
                    && entity.name == name
                    && entity.class_id == class_id
            })
            .map(|entity| entity.id)
    }

    // pub fn migration_execute(&mut self, pkt: PKTMigrationExecute) {
    //     if self
    //         .id_tracker
    //         .borrow()
    //         .get_local_character_id(self.local_entity_id)
    //         != 0
    //     {
    //         return;
    //     }

    //     let char_id = if pkt.account_character_id1 < pkt.account_character_id2 {
    //         pkt.account_character_id1
    //     } else {
    //         pkt.account_character_id2
    //     };

    //     info!("character id: {}->{}", self.local_character_id, char_id);
    //     self.local_character_id = char_id;
    //     self.id_tracker
    //         .borrow_mut()
    //         .add_mapping(char_id, self.local_entity_id);

    //     self.entities
    //         .entry(self.local_entity_id)
    //         .and_modify(|e| {
    //             e.character_id = char_id;
    //         })
    //         .or_insert_with(|| Entity {
    //             entity_type: Player,
    //             name: "You".to_string(),
    //             character_id: char_id,
    //             ..Default::default()
    //         });
    // }

    pub fn new_pc(&mut self, pc_struct: PCStruct) -> Entity {
        self.last_reconnect_rebind = None;
        let mut entity = Entity {
            id: pc_struct.player_id,
            entity_type: Player,
            name: pc_struct.name.clone(),
            class_id: pc_struct.class_id as u32,
            gear_level: truncate_gear_level(pc_struct.max_item_level), // todo?
            character_id: pc_struct.character_id,
            stats: pc_struct
                .stat_pairs
                .iter()
                .map(|sp| (sp.stat_type, sp.value))
                .collect(),
            ..Default::default()
        };

        let previous_entity_id = {
            let mapped_entity_id = self
                .id_tracker
                .borrow()
                .get_entity_id(pc_struct.character_id)
                .filter(|entity_id| self.entities.contains_key(entity_id));
            mapped_entity_id
                .or_else(|| {
                    self.find_reconnect_entity_id(
                        &pc_struct.name,
                        pc_struct.class_id as u32,
                        pc_struct.player_id,
                    )
                })
                .or_else(|| {
                    self.take_removed_reconnect_entity(
                        pc_struct.character_id,
                        &pc_struct.name,
                        pc_struct.class_id as u32,
                    )
                    .map(|previous_entity| {
                        let previous_entity_id = previous_entity.id;
                        merge_player_identity_state(&mut entity, previous_entity);
                        previous_entity_id
                    })
                })
        };

        if let Some(previous_entity) = self
            .entities
            .get(&entity.id)
            .filter(|previous_entity| {
                previous_entity.entity_type == Player
                    && previous_entity.character_id == pc_struct.character_id
            })
            .cloned()
        {
            merge_player_identity_state(&mut entity, previous_entity);
        }

        if let Some(previous_entity_id) = previous_entity_id.filter(|old| *old != entity.id) {
            self.last_reconnect_rebind = Some((previous_entity_id, entity.id));
            if let Some(previous_entity) = self.entities.remove(&previous_entity_id) {
                merge_player_identity_state(&mut entity, previous_entity);
                if self.local_entity_id == previous_entity_id {
                    self.local_entity_id = entity.id;
                }
            }

            self.id_tracker
                .borrow_mut()
                .remove_entity_mapping(previous_entity_id);
            self.party_tracker
                .borrow_mut()
                .change_entity_id(previous_entity_id, entity.id);
        }

        self.entities.insert(entity.id, entity.clone());
        self.apply_pending_inspect_result_for_name(&pc_struct.name);
        self.id_tracker
            .borrow_mut()
            .add_mapping(pc_struct.character_id, pc_struct.player_id);
        self.party_tracker
            .borrow_mut()
            .complete_entry(pc_struct.character_id, pc_struct.player_id);
        self.character_id_to_name
            .insert(pc_struct.character_id, pc_struct.name.clone());
        // println!("party status: {:?}", self.party_tracker.borrow().character_id_to_party_id);
        let use_party_status_effects =
            self.should_use_party_status_effect_for_character(pc_struct.character_id);
        if use_party_status_effects {
            self.status_tracker
                .borrow_mut()
                .remove_party_object(pc_struct.character_id);
        } else {
            self.status_tracker
                .borrow_mut()
                .remove_local_object(pc_struct.player_id);
        }
        let (status_effect_target_id, status_effect_target_type) = if use_party_status_effects {
            (pc_struct.character_id, StatusEffectTargetType::Party)
        } else {
            (pc_struct.player_id, StatusEffectTargetType::Local)
        };
        let timestamp = Utc::now();
        for sed in &pc_struct.status_effect_datas {
            let source_entity = self.resolve_status_effect_source_entity(sed);
            let status_effect = self.build_status_effect_with_snapshots(
                sed,
                status_effect_target_id,
                status_effect_target_type,
                timestamp,
                None,
                Some(source_entity),
            );
            self.status_tracker
                .borrow_mut()
                .register_status_effect(status_effect);
        }
        entity
    }

    pub fn new_npc(&mut self, pkt: PKTNewNpc, max_hp: i64) -> Entity {
        let (entity_type, name, grade, bars) =
            get_npc_entity_type_name_grade_bars(&pkt.npc_struct, max_hp);
        let npc = Entity {
            id: pkt.npc_struct.object_id,
            entity_type,
            name,
            grade,
            npc_id: pkt.npc_struct.type_id,
            hp_bars: bars,
            level: pkt.npc_struct.level,
            push_immune: entity_type == Boss,
            stats: pkt
                .npc_struct
                .stat_pairs
                .iter()
                .map(|sp| (sp.stat_type, sp.value))
                .collect(),
            ..Default::default()
        };
        self.entities.insert(npc.id, npc.clone());
        self.status_tracker.borrow_mut().remove_local_object(npc.id);
        self.build_and_register_status_effects(pkt.npc_struct.status_effect_datas, npc.id);
        npc
    }

    pub fn new_npc_summon(&mut self, pkt: PKTNewNpcSummon, max_hp: i64) -> Entity {
        let (entity_type, name, grade, bars) =
            get_npc_entity_type_name_grade_bars(&pkt.npc_struct, max_hp);
        let entity_type = if entity_type == Npc {
            Summon
        } else {
            entity_type
        };
        let npc = Entity {
            id: pkt.npc_struct.object_id,
            entity_type,
            name,
            grade,
            npc_id: pkt.npc_struct.type_id,
            hp_bars: bars,
            owner_id: pkt.owner_id,
            level: pkt.npc_struct.level,
            push_immune: entity_type == Boss,
            stats: pkt
                .npc_struct
                .stat_pairs
                .iter()
                .map(|sp| (sp.stat_type, sp.value))
                .collect(),
            ..Default::default()
        };
        self.entities.insert(npc.id, npc.clone());
        self.status_tracker.borrow_mut().remove_local_object(npc.id);
        self.build_and_register_status_effects(pkt.npc_struct.status_effect_datas, npc.id);
        npc
    }

    pub fn party_status_effect_add(
        &mut self,
        pkt: PKTPartyStatusEffectAddNotify,
        entities: &HashMap<String, EncounterEntity>,
    ) -> Vec<StatusEffectDetails> {
        let timestamp = Utc::now();
        let mut shields: Vec<StatusEffectDetails> = Vec::new();
        let (target_id, target_type) = if pkt.character_id != 0
            && !self.should_use_party_status_effect_for_character(pkt.character_id)
            && pkt.character_id == self.get_local_character_id()
            && self.local_entity_id != 0
        {
            (self.local_entity_id, StatusEffectTargetType::Local)
        } else {
            (pkt.character_id, StatusEffectTargetType::Party)
        };
        for sed in pkt.status_effect_datas {
            let source_entity = self.resolve_status_effect_source_entity(&sed);
            let encounter_entity = entities.get(&source_entity.name);
            let status_effect = self.build_status_effect_with_snapshots(
                &sed,
                target_id,
                target_type,
                timestamp,
                encounter_entity,
                Some(source_entity),
            );
            if status_effect.status_effect_type == StatusEffectType::Shield {
                shields.push(status_effect.clone());
            }

            self.status_tracker
                .borrow_mut()
                .register_status_effect(status_effect);
        }
        shields
    }

    pub fn party_status_effect_remove(
        &mut self,
        pkt: PKTPartyStatusEffectRemoveNotify,
    ) -> (
        bool,
        Vec<StatusEffectDetails>,
        Vec<StatusEffectDetails>,
        bool,
    ) {
        let (target_id, target_type) = if pkt.character_id != 0
            && !self.should_use_party_status_effect_for_character(pkt.character_id)
            && pkt.character_id == self.get_local_character_id()
            && self.local_entity_id != 0
        {
            (self.local_entity_id, StatusEffectTargetType::Local)
        } else {
            (pkt.character_id, StatusEffectTargetType::Party)
        };
        self.status_tracker.borrow_mut().remove_status_effects(
            target_id,
            pkt.status_effect_instance_ids,
            pkt.reason,
            target_type,
        )
    }

    pub fn new_projectile(&mut self, pkt: &PKTNewProjectile) {
        let projectile = Entity {
            id: pkt.projectile_info.projectile_id,
            entity_type: EntityType::Projectile,
            name: format!("{:x}", pkt.projectile_info.projectile_id),
            owner_id: pkt.projectile_info.owner_id,
            skill_id: pkt.projectile_info.skill_id,
            skill_effect_id: pkt.projectile_info.skill_effect,
            ..Default::default()
        };
        self.entities.insert(projectile.id, projectile);
    }

    pub fn new_trap(&mut self, pkt: &PKTNewTrap) {
        let trap: Entity = Entity {
            id: pkt.trap_struct.object_id,
            entity_type: EntityType::Projectile,
            name: format!("{:x}", pkt.trap_struct.object_id),
            owner_id: pkt.trap_struct.owner_id,
            skill_id: pkt.trap_struct.skill_id,
            skill_effect_id: pkt.trap_struct.skill_effect,
            ..Default::default()
        };
        self.entities.insert(trap.id, trap);
    }

    pub fn party_info(&mut self, pkt: PKTPartyInfo, local_info: &LocalInfo) {
        let mut unknown_local = if let Some(local_player) = self.entities.get(&self.local_entity_id)
        {
            local_player.name.is_empty()
                || local_player.name == "You"
                || local_player.name.starts_with('0')
        } else {
            true
        };

        self.party_tracker
            .borrow_mut()
            .remove_party_mappings(pkt.party_instance_id);

        let most_likely_local_name = if unknown_local {
            let party_members = pkt
                .party_member_datas
                .iter()
                .map(|m| m.character_id)
                .collect::<Vec<u64>>();
            let mut party_locals = local_info
                .local_players
                .iter()
                .filter_map(|(k, v)| {
                    if party_members.contains(k) {
                        Some(v)
                    } else {
                        None
                    }
                })
                .collect::<Vec<&LocalPlayer>>();
            party_locals.sort_by(|a, b| b.count.cmp(&a.count));
            party_locals
                .first()
                .map_or_else(String::new, |p| p.name.clone())
        } else {
            "".to_string()
        };

        for member in pkt.party_member_datas {
            self.character_id_to_name
                .insert(member.character_id, member.name.clone());
            if unknown_local
                && member.name == most_likely_local_name
                && let Some(local_player) = self.entities.get_mut(&self.local_entity_id)
            {
                unknown_local = false;
                warn!(
                    "unknown local player, inferring from cache: {}",
                    member.name
                );
                local_player.entity_type = Player;
                local_player.class_id = member.class_id as u32;
                local_player.gear_level = truncate_gear_level(member.gear_level);
                local_player.name.clone_from(&member.name);
                local_player.character_id = member.character_id;
                self.character_id_to_name
                    .insert(member.character_id, member.name.clone());
                self.id_tracker
                    .borrow_mut()
                    .add_mapping(member.character_id, self.local_entity_id);
                self.party_tracker
                    .borrow_mut()
                    .set_name(member.name.clone());
            }

            let entity_id = {
                let mapped_entity_id = self
                    .id_tracker
                    .borrow()
                    .get_entity_id(member.character_id)
                    .filter(|entity_id| self.entities.contains_key(entity_id));
                mapped_entity_id
                    .or_else(|| {
                        self.find_reconnect_entity_id(&member.name, member.class_id as u32, 0)
                    })
                    .or_else(|| {
                        self.take_removed_reconnect_entity(
                            member.character_id,
                            &member.name,
                            member.class_id as u32,
                        )
                        .map(|previous_entity| {
                            let entity_id = previous_entity.id;
                            self.entities
                                .entry(entity_id)
                                .and_modify(|entity| {
                                    merge_player_identity_state(entity, previous_entity.clone())
                                })
                                .or_insert(previous_entity);
                            entity_id
                        })
                    })
            };

            if let Some(entity_id) = entity_id {
                if let Some(entity) = self.entities.get_mut(&entity_id)
                    && entity.entity_type == Player
                {
                    entity.character_id = member.character_id;
                    entity.name.clone_from(&member.name);
                    entity.gear_level = truncate_gear_level(member.gear_level);
                    entity.class_id = member.class_id as u32;
                }

                self.id_tracker
                    .borrow_mut()
                    .add_mapping(member.character_id, entity_id);
                self.party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    member.character_id,
                    entity_id,
                    Some(member.name.clone()),
                );
            } else {
                self.party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    member.character_id,
                    0,
                    Some(member.name.clone()),
                );
            }
        }
    }

    pub fn remove_object(&mut self, entity_id: u64) {
        if let Some(entity) = self.entities.remove(&entity_id) {
            self.id_tracker
                .borrow_mut()
                .remove_entity_mapping(entity_id);
            self.party_tracker
                .borrow_mut()
                .remove_entity_mapping(entity_id);
            self.stash_removed_player_entity(&entity);
        }
    }

    pub fn collect_missing_party_inspects(&mut self, bootstrap_active: bool) -> Vec<String> {
        let now = Utc::now().timestamp_millis();
        let local_character_id = self.get_local_character_id();
        let forced_refresh_names = self.forced_refresh_names.clone();
        let bootstrap_refresh_sent_names = self.bootstrap_refresh_sent_names.clone();

        let mut names = Vec::new();
        let mut queued_names = HashSet::new();
        if let Some(local_entity) = self
            .entities
            .get_mut(&self.local_entity_id)
            .filter(|entity| entity.entity_type == Player)
            && is_resolved_player_name(&local_entity.name)
        {
            let bootstrap_refresh_needed =
                bootstrap_active && !bootstrap_refresh_sent_names.contains(&local_entity.name);
            let force_refresh = forced_refresh_names.contains(&local_entity.name);
            let already_ready = !force_refresh
                && !bootstrap_refresh_needed
                && local_entity.inspect_snapshot.is_some()
                && !local_entity.inspect_stale;
            let ready_for_request = force_refresh
                || bootstrap_refresh_needed
                || local_entity.inspect_ready_at_ms <= now;
            let needs_request =
                force_refresh || bootstrap_refresh_needed || local_entity.inspect_stale;
            if ready_for_request
                && needs_request
                && !already_ready
                && !local_entity.inspect_requested
                && !self.inspect_requested_names.contains(&local_entity.name)
                && !self
                    .pending_inspect_results_by_name
                    .contains_key(&local_entity.name)
            {
                local_entity.inspect_requested = true;
                self.inspect_requested_names
                    .insert(local_entity.name.clone());
                queued_names.insert(local_entity.name.clone());
                names.push(local_entity.name.clone());
            }
        }

        let registered_party_characters = self
            .party_tracker
            .borrow()
            .get_all_registered_party_characters();
        if registered_party_characters.is_empty() {
            return names;
        }

        for character_id in registered_party_characters {
            if character_id == local_character_id {
                continue;
            }
            let Some(name) = self.get_resolved_player_name_for_character(character_id) else {
                continue;
            };
            let bootstrap_refresh_needed =
                bootstrap_active && !bootstrap_refresh_sent_names.contains(&name);
            let force_refresh = forced_refresh_names.contains(&name);
            if !is_resolved_player_name(&name)
                || self.pending_inspect_results_by_name.contains_key(&name)
                || self.inspect_requested_names.contains(&name)
                || !queued_names.insert(name.clone())
            {
                continue;
            }

            let mut already_ready = false;
            let mut should_request = bootstrap_refresh_needed || force_refresh;
            for entity in self.entities.values_mut().filter(|entity| {
                entity.entity_type == Player
                    && (entity.character_id == character_id || entity.name == name)
            }) {
                if !force_refresh
                    && !bootstrap_refresh_needed
                    && ((entity.inspect_snapshot.is_some() && !entity.inspect_stale)
                        || entity.inspect_ready_at_ms > now)
                {
                    already_ready = true;
                    break;
                }
                should_request |= entity.inspect_stale;
            }
            if !already_ready {
                let (stashed_ready, stashed_should_request) = self
                    .evaluate_stashed_player_inspect_state(
                        character_id,
                        &name,
                        now,
                        force_refresh,
                        bootstrap_refresh_needed,
                    );
                already_ready = stashed_ready;
                should_request |= stashed_should_request;
            }
            if !already_ready && should_request {
                self.mark_matching_player_request_state(character_id, &name, true);
                self.inspect_requested_names.insert(name.clone());
                names.push(name);
            }
        }

        names
    }

    pub fn collect_missing_visible_player_fallback_inspects(&mut self) -> Vec<String> {
        if !self.bootstrap_visible_fallback_requested {
            self.bootstrap_visible_fallback_requested = true;
            self.bootstrap_visible_fallback_names.clear();
            if self.party_tracker.borrow().get_registered_party_count() < 2 {
                return Vec::new();
            }

            let tracked_party_characters = self
                .party_tracker
                .borrow()
                .get_all_registered_party_characters()
                .into_iter()
                .collect::<HashSet<_>>();
            let tracked_party_names = self
                .party_tracker
                .borrow()
                .get_tracked_party_names()
                .into_iter()
                .filter(|name| is_resolved_player_name(name))
                .collect::<HashSet<_>>();
            let tracked_party_entity_ids = self
                .entities
                .values()
                .filter(|entity| {
                    entity.entity_type == Player
                        && entity.character_id != 0
                        && tracked_party_characters.contains(&entity.character_id)
                })
                .map(|entity| entity.id)
                .collect::<HashSet<_>>();
            let local_character_id = self.get_local_character_id();
            let mut visible_names_seen = HashSet::new();

            for entity in self.entities.values() {
                if entity.entity_type != Player
                    || entity.id == self.local_entity_id
                    || !is_resolved_player_name(&entity.name)
                    || tracked_party_entity_ids.contains(&entity.id)
                    || (entity.character_id != 0
                        && tracked_party_characters.contains(&entity.character_id))
                    || tracked_party_names.contains(&entity.name)
                    || (entity.character_id != 0 && entity.character_id == local_character_id)
                    || !visible_names_seen.insert(entity.name.clone())
                {
                    continue;
                }

                self.bootstrap_visible_fallback_names
                    .push(entity.name.clone());
            }
        }

        let mut names = Vec::new();
        let pending_names = self.bootstrap_visible_fallback_names.clone();
        for name in pending_names {
            if self.bootstrap_refresh_sent_names.contains(&name)
                || self.pending_inspect_results_by_name.contains_key(&name)
                || self.inspect_requested_names.contains(&name)
            {
                continue;
            }

            for entity in self
                .entities
                .values_mut()
                .filter(|entity| entity.entity_type == Player && entity.name == name)
            {
                entity.inspect_requested = true;
            }

            self.inspect_requested_names.insert(name.clone());
            names.push(name);
        }

        names
    }

    pub fn clear_inspect_request(&mut self, name: &str) {
        self.inspect_requested_names.remove(name);
        for entity in self
            .entities
            .values_mut()
            .filter(|entity| entity.name == name)
        {
            entity.inspect_requested = false;
        }
        self.update_removed_player_copies_by_name(name, |entity| entity.inspect_requested = false);
    }

    pub fn mark_inspect_stale(&mut self, character_id: u64) {
        let mut inspect_name = None;
        for entity in self
            .entities
            .values_mut()
            .filter(|entity| entity.entity_type == Player && entity.character_id == character_id)
        {
            entity.inspect_requested = false;
            entity.inspect_stale = true;
            if inspect_name.is_none() {
                inspect_name = Some(entity.name.clone());
            }
        }
        self.update_removed_player_copies_by_character(character_id, |entity| {
            entity.inspect_requested = false;
            entity.inspect_stale = true;
        });
        if let Some(name) =
            inspect_name.or_else(|| self.get_resolved_player_name_for_character(character_id))
        {
            self.inspect_requested_names.remove(&name);
        }
    }

    pub fn on_new_transit(&mut self) {
        self.pending_inspect_results_by_name.clear();
        self.inspect_requested_names.clear();
        self.forced_refresh_names.clear();
        self.bootstrap_refresh_sent_names.clear();
        self.bootstrap_visible_fallback_requested = false;
        self.bootstrap_visible_fallback_names.clear();
        self.next_bootstrap_inspect_at_ms = 0;
        for entity in self.entities.values_mut() {
            entity.inspect_requested = false;
        }
        let mut local_character = None;
        let mut local_name = None;
        if let Some(local_entity) = self
            .entities
            .get_mut(&self.local_entity_id)
            .filter(|entity| entity.entity_type == Player)
        {
            local_entity.inspect_ready_at_ms = 0;
            local_entity.inspect_stale = false;
            local_entity.inspect_info = None;
            local_entity.inspect_result = None;
            local_entity.inspect_snapshot = None;
            local_entity.skill_runtime_data.clear();
            local_character = (local_entity.character_id > 0).then_some(local_entity.character_id);
            local_name =
                is_resolved_player_name(&local_entity.name).then_some(local_entity.name.clone());
        }
        if let (Some(character_id), Some(name)) = (local_character, local_name) {
            self.character_id_to_name.insert(character_id, name.clone());
            self.party_tracker.borrow_mut().set_name(name);
        }
    }

    pub fn apply_inspect_result(
        &mut self,
        result: PKTPCInspectResult,
    ) -> Option<AppliedInspectResult> {
        let name = result.name.clone();
        self.inspect_requested_names.remove(&name);
        let snapshot = inspect_snapshot_from_result(&result);
        let info = inspect_info_from_result(&result);
        let has_live_match = self
            .entities
            .values()
            .any(|entity| entity.entity_type == Player && entity.name == name);
        let removed_character_ids = self
            .removed_player_entities_by_character_id
            .iter()
            .filter_map(|(character_id, entity)| {
                (entity.entity_type == Player && entity.name == name).then_some(*character_id)
            })
            .collect::<Vec<_>>();
        let removed_name_class_keys = self
            .removed_player_entities_by_name_class
            .keys()
            .filter(|(player_name, _)| player_name == &name)
            .cloned()
            .collect::<Vec<_>>();

        if !has_live_match && removed_character_ids.is_empty() && removed_name_class_keys.is_empty()
        {
            self.pending_inspect_results_by_name.insert(name, result);
            return None;
        }

        let result = Arc::new(result);

        for entity in self
            .entities
            .values_mut()
            .filter(|entity| entity.entity_type == Player && entity.name == name)
        {
            apply_inspect_payload_to_entity(entity, &snapshot, &info, result.clone());
        }
        for character_id in removed_character_ids {
            if let Some(entity) = self
                .removed_player_entities_by_character_id
                .get_mut(&character_id)
            {
                apply_inspect_payload_to_entity(entity, &snapshot, &info, result.clone());
            }
        }
        for key in removed_name_class_keys {
            if let Some(entity) = self.removed_player_entities_by_name_class.get_mut(&key) {
                apply_inspect_payload_to_entity(entity, &snapshot, &info, result.clone());
            }
        }

        self.forced_refresh_names.remove(&name);
        Some(AppliedInspectResult { name, info })
    }

    pub fn get_local_character_id(&self) -> u64 {
        if self.local_character_id != 0 {
            self.local_character_id
        } else {
            self.id_tracker
                .borrow()
                .get_local_character_id(self.local_entity_id)
        }
    }

    fn should_use_party_status_effect_for_character(&self, character_id: u64) -> bool {
        let local_character_id = self.get_local_character_id();
        let party_tracker = self.party_tracker.borrow();
        let local_player_party_id = party_tracker
            .character_id_to_party_id
            .get(&local_character_id);
        let affected_player_party_id = party_tracker.character_id_to_party_id.get(&character_id);

        matches!(
            (
                local_player_party_id,
                affected_player_party_id,
                character_id == local_character_id,
            ),
            (Some(local_party), Some(affected_party), false) if local_party == affected_party
        )
    }

    pub fn are_same_party_entities(&self, lhs_entity_id: u64, rhs_entity_id: u64) -> bool {
        if lhs_entity_id == 0 || rhs_entity_id == 0 {
            return false;
        }
        if lhs_entity_id == rhs_entity_id {
            return true;
        }

        let lhs_character_id = self
            .entities
            .get(&lhs_entity_id)
            .map(|entity| entity.character_id);
        let rhs_character_id = self
            .entities
            .get(&rhs_entity_id)
            .map(|entity| entity.character_id);
        matches!(
            (lhs_character_id, rhs_character_id),
            (Some(lhs), Some(rhs))
                if lhs != 0
                    && rhs != 0
                    && (lhs == rhs
                        || self
                            .party_tracker
                            .borrow()
                            .are_same_party_characters(lhs, rhs))
        )
    }

    pub fn are_same_party_characters(&self, lhs_character_id: u64, rhs_character_id: u64) -> bool {
        lhs_character_id != 0
            && rhs_character_id != 0
            && (lhs_character_id == rhs_character_id
                || self
                    .party_tracker
                    .borrow()
                    .are_same_party_characters(lhs_character_id, rhs_character_id))
    }

    pub fn is_gate_eligible_player_entity(&self, entity: &Entity) -> bool {
        if entity.entity_type != Player || !is_resolved_player_name(&entity.name) {
            return false;
        }
        if entity.id == self.local_entity_id {
            return true;
        }

        entity.character_id != 0
            && self
                .party_tracker
                .borrow()
                .get_all_registered_party_characters()
                .contains(&entity.character_id)
    }

    pub fn get_required_rdps_player_names(&self) -> Vec<String> {
        let mut names = self
            .get_required_rdps_player_character_ids()
            .into_iter()
            .filter_map(|character_id| self.get_resolved_player_name_for_character(character_id))
            .collect::<Vec<_>>();
        names.sort();
        names.dedup();
        names
    }

    pub fn get_tracked_same_party_player_entity_ids(&self) -> Vec<u64> {
        let local_character_id = self.get_local_character_id();
        if local_character_id == 0 {
            return Vec::new();
        }

        self.entities
            .values()
            .filter(|entity| {
                entity.entity_type == Player
                    && entity.character_id != 0
                    && entity.character_id != local_character_id
                    && self
                        .party_tracker
                        .borrow()
                        .are_same_party_characters(local_character_id, entity.character_id)
            })
            .map(|entity| entity.id)
            .collect()
    }

    pub fn get_group_number_for_entity(&self, entity_id: u64) -> Option<usize> {
        let character_id = self.entities.get(&entity_id)?.character_id;
        self.party_tracker
            .borrow()
            .get_group_number_for_character(character_id)
    }

    pub fn has_inspect_snapshot_for_name(&self, name: &str) -> bool {
        self.entities.values().any(|entity| {
            entity.entity_type == Player
                && entity.name == name
                && entity.inspect_snapshot.is_some()
                && !entity.inspect_stale
        }) || self.pending_inspect_results_by_name.contains_key(name)
            || self
                .removed_player_entities_by_character_id
                .values()
                .any(|entity| {
                    entity.entity_type == Player
                        && entity.name == name
                        && entity.inspect_snapshot.is_some()
                        && !entity.inspect_stale
                })
            || self
                .removed_player_entities_by_name_class
                .values()
                .any(|entity| {
                    entity.entity_type == Player
                        && entity.name == name
                        && entity.inspect_snapshot.is_some()
                        && !entity.inspect_stale
                })
    }

    pub fn can_send_bootstrap_inspect(&self, now: i64) -> bool {
        now >= self.next_bootstrap_inspect_at_ms
    }

    pub fn note_bootstrap_inspect_sent(&mut self, name: &str, now: i64) {
        self.next_bootstrap_inspect_at_ms = now + BOOTSTRAP_INSPECT_DELAY_MS;
        self.bootstrap_refresh_sent_names.insert(name.to_string());
        self.bootstrap_visible_fallback_names
            .retain(|queued_name| queued_name != name);
    }

    pub fn reset_bootstrap_inspect_throttle(&mut self) {
        self.inspect_requested_names.clear();
        for entity in self.entities.values_mut() {
            entity.inspect_requested = false;
        }
        for entity in self.removed_player_entities_by_character_id.values_mut() {
            entity.inspect_requested = false;
        }
        for entity in self.removed_player_entities_by_name_class.values_mut() {
            entity.inspect_requested = false;
        }
        self.next_bootstrap_inspect_at_ms = 0;
        self.bootstrap_refresh_sent_names.clear();
        self.bootstrap_visible_fallback_requested = false;
        self.bootstrap_visible_fallback_names.clear();
    }

    pub fn take_last_reconnect_rebind(&mut self) -> Option<(u64, u64)> {
        self.last_reconnect_rebind.take()
    }

    pub fn get_required_rdps_player_character_ids(&self) -> Vec<u64> {
        let mut character_ids = self
            .party_tracker
            .borrow()
            .get_all_registered_party_characters()
            .into_iter()
            .filter(|character_id| {
                self.id_tracker
                    .borrow()
                    .get_entity_id(*character_id)
                    .is_some_and(|entity_id| entity_id != 0)
            })
            .collect::<Vec<_>>();
        let local_character_id = self.get_local_character_id();
        if local_character_id != 0 && self.local_entity_id != 0 {
            character_ids.push(local_character_id);
        }
        character_ids.sort_unstable();
        character_ids.dedup();
        character_ids
    }

    pub fn queue_forced_inspect_refresh(&mut self, name: &str) {
        if !is_resolved_player_name(name) {
            return;
        }

        self.clear_inspect_request(name);
        self.forced_refresh_names.insert(name.to_string());
    }

    pub fn is_startup_barrier_ready(
        &self,
        required_character_ids: &[u64],
        required_names: &[String],
    ) -> bool {
        if required_character_ids.is_empty() && required_names.is_empty() {
            return false;
        }
        required_character_ids
            .iter()
            .copied()
            .all(|character_id| self.has_inspect_snapshot_for_character(character_id))
            && required_names
                .iter()
                .all(|name| self.has_inspect_snapshot_for_name(name))
    }

    pub fn get_inspect_snapshot(&self, entity_id: u64) -> Option<&InspectSnapshot> {
        self.entities.get(&entity_id)?.inspect_snapshot.as_ref()
    }

    pub fn record_skill_cast(
        &mut self,
        entity_id: u64,
        skill_id: u32,
        skill_level: u8,
        timestamp: i64,
    ) {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        if entity.entity_type != Player || skill_id == 0 {
            return;
        }

        let skill_runtime = entity.skill_runtime_data.entry(skill_id).or_default();
        if skill_runtime.skill_level == 0 {
            skill_runtime.skill_level = skill_level;
        }
        skill_runtime.last_cast_at_ms = Some(timestamp);
    }

    pub fn record_skill_start(
        &mut self,
        entity_id: u64,
        skill_id: u32,
        skill_level: u8,
        skill_option_data: &meter_defs::types::SkillOptionData,
        timestamp: i64,
    ) {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        if entity.entity_type != Player || skill_id == 0 {
            return;
        }

        let skill_option_snapshot = SkillOptionSnapshot::from_skill_option_data(skill_option_data);
        let skill_runtime = entity.skill_runtime_data.entry(skill_id).or_default();
        let needs_refresh = skill_runtime.skill_level != skill_level
            || skill_runtime.skill_option_data.as_ref() != Some(&skill_option_snapshot);
        if needs_refresh {
            skill_runtime.skill_level = skill_level;
            skill_runtime.skill_option_data = Some(skill_option_snapshot);
            populate_skill_runtime_data(skill_runtime, skill_id);
        }
        skill_runtime.last_start_at_ms = Some(timestamp);
        skill_runtime.identity_gauge1_at_start = Some(entity.identity_gauge1);
        skill_runtime.identity_gauge2_at_start = Some(entity.identity_gauge2);
        skill_runtime.identity_gauge3_at_start = Some(entity.identity_gauge3);
        entity.identity_last_skill_start_id = skill_id;
        entity.identity_last_skill_start_at_ms = timestamp;
        if skill_id == DESTROYER_VORTEX_GRAVITY_SKILL_ID
            || skill_id == DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID
        {
            entity.destroyer_recent_consumed_cores = 0;
            entity.destroyer_recent_consumed_at_ms = 0;
        }
    }

    pub fn record_skill_start_snapshot(
        &mut self,
        entity_id: u64,
        skill_id: u32,
        skill_level: u8,
        skill_option_snapshot: Option<SkillOptionSnapshot>,
        timestamp: i64,
    ) {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        if entity.entity_type != Player || skill_id == 0 {
            return;
        }

        let skill_runtime = entity.skill_runtime_data.entry(skill_id).or_default();
        let needs_refresh = skill_runtime.skill_level != skill_level
            || skill_runtime.skill_option_data != skill_option_snapshot;
        if needs_refresh {
            skill_runtime.skill_level = skill_level;
            skill_runtime.skill_option_data = skill_option_snapshot;
            populate_skill_runtime_data(skill_runtime, skill_id);
        }
        skill_runtime.last_start_at_ms = Some(timestamp);
        skill_runtime.identity_gauge1_at_start = Some(entity.identity_gauge1);
        skill_runtime.identity_gauge2_at_start = Some(entity.identity_gauge2);
        skill_runtime.identity_gauge3_at_start = Some(entity.identity_gauge3);
        entity.identity_last_skill_start_id = skill_id;
        entity.identity_last_skill_start_at_ms = timestamp;
        if skill_id == DESTROYER_VORTEX_GRAVITY_SKILL_ID
            || skill_id == DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID
        {
            entity.destroyer_recent_consumed_cores = 0;
            entity.destroyer_recent_consumed_at_ms = 0;
        }
    }

    pub fn get_skill_runtime_data(
        &self,
        entity_id: u64,
        skill_id: u32,
    ) -> Option<&SkillRuntimeData> {
        self.entities
            .get(&entity_id)?
            .skill_runtime_data
            .get(&skill_id)
    }

    pub fn record_identity_gauge_change(
        &mut self,
        entity_id: u64,
        gauge1: u32,
        gauge2: u32,
        gauge3: u32,
        timestamp: i64,
    ) {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        let previous_gauge2 = entity.identity_gauge2;
        entity.identity_gauge1_prev = entity.identity_gauge1;
        entity.identity_gauge2_prev = entity.identity_gauge2;
        entity.identity_gauge3_prev = entity.identity_gauge3;
        entity.identity_gauge1 = gauge1;
        entity.identity_gauge2 = gauge2;
        entity.identity_gauge3 = gauge3;
        let recent_destroyer_skill = entity.identity_last_skill_start_id
            == DESTROYER_VORTEX_GRAVITY_SKILL_ID
            || entity.identity_last_skill_start_id == DESTROYER_HYPERGRAVITY_BASIC_ATTACK_SKILL_ID;
        if entity.class_id == DESTROYER_CLASS_ID
            && recent_destroyer_skill
            && timestamp >= entity.identity_last_skill_start_at_ms
            && timestamp - entity.identity_last_skill_start_at_ms
                <= RECENT_DESTROYER_SKILL_START_WINDOW_MS
            && previous_gauge2 > gauge2
        {
            entity.destroyer_recent_consumed_cores = (previous_gauge2 - gauge2) as i32;
            entity.destroyer_recent_consumed_at_ms = timestamp;
        }
    }

    pub fn record_identity_stance(&mut self, entity_id: u64, stance: u8) {
        let Some(entity) = self.entities.get_mut(&entity_id) else {
            return;
        };
        entity.stance = stance;
    }

    pub fn get_source_entity(&mut self, id: u64) -> Entity {
        let id = self.entities.get(&id).map_or(id, |entity| {
            if entity.entity_type == EntityType::Projectile
                || entity.entity_type == EntityType::Summon
            {
                entity.owner_id
            } else {
                id
            }
        });

        self.entities.get(&id).cloned().unwrap_or_else(|| {
            let entity = Entity {
                id,
                entity_type: EntityType::Unknown,
                name: format!("{:x}", id),
                ..Default::default()
            };
            self.entities.insert(entity.id, entity.clone());
            entity
        })
    }

    pub fn id_is_player(&mut self, id: u64) -> bool {
        if let Some(entity) = self.entities.get(&id) {
            entity.entity_type == EntityType::Player
        } else {
            false
        }
    }

    pub fn guess_is_player(&mut self, entity: &mut Entity, skill_id: u32) {
        if (entity.entity_type != EntityType::Unknown && entity.entity_type != EntityType::Player)
            || (entity.entity_type == EntityType::Player && entity.class_id != 0)
        {
            return;
        }

        let class_id = get_skill_class_id(&skill_id);
        if class_id != 0 {
            if entity.entity_type == EntityType::Player {
                if entity.class_id == class_id {
                    return;
                }
                entity.class_id = class_id;
            } else {
                entity.entity_type = Player;
                entity.class_id = class_id;
            }
            self.entities.insert(entity.id, entity.clone());
        }
    }

    pub fn build_and_register_status_effect(
        &mut self,
        sed: &StatusEffectData,
        target_id: u64,
        timestamp: DateTime<Utc>,
        entities: Option<&HashMap<String, EncounterEntity>>,
    ) -> StatusEffectDetails {
        let source_entity = self.resolve_status_effect_source_entity(sed);
        let source_encounter_entity =
            entities.and_then(|entities| entities.get(&source_entity.name));
        let status_effect = self.build_status_effect_with_snapshots(
            sed,
            target_id,
            StatusEffectTargetType::Local,
            timestamp,
            source_encounter_entity,
            Some(source_entity),
        );

        self.status_tracker
            .borrow_mut()
            .register_status_effect(status_effect.clone());

        status_effect
    }

    fn build_status_effect_with_snapshots(
        &mut self,
        sed: &StatusEffectData,
        target_id: u64,
        target_type: StatusEffectTargetType,
        timestamp: DateTime<Utc>,
        source_encounter_entity: Option<&EncounterEntity>,
        resolved_source_entity: Option<Entity>,
    ) -> StatusEffectDetails {
        let mut source_entity =
            resolved_source_entity.unwrap_or_else(|| self.resolve_status_effect_source_entity(sed));
        if source_entity.entity_type != EntityType::Player
            && sed.source_id == 0
            && let Some(skill_buff) = SKILL_BUFF_DATA.get(&sed.status_effect_id)
            && !skill_buff.target.eq_ignore_ascii_case("party")
            && !skill_buff.target.eq_ignore_ascii_case("self_party")
            && let Some(target_entity) =
                self.resolve_status_effect_target_entity(target_id, target_type)
            && matches!(target_entity.entity_type, EntityType::Player)
        {
            source_entity = target_entity;
        }
        let mut status_effect = build_status_effect(
            sed,
            target_id,
            source_entity.id,
            target_type,
            timestamp,
            source_encounter_entity,
        );

        self.populate_status_effect_snapshots(&mut status_effect, &source_entity, timestamp);

        status_effect
    }

    fn populate_status_effect_snapshots(
        &mut self,
        status_effect: &mut StatusEffectDetails,
        source_entity: &Entity,
        timestamp: DateTime<Utc>,
    ) {
        if source_entity.entity_type != EntityType::Player {
            status_effect.owner_player_stats_snapshot = None;
            status_effect.source_skill_runtime_snapshot = None;
            return;
        }

        let self_effects = self.status_tracker.borrow_mut().get_source_status_effects(
            source_entity,
            self.local_character_id,
            timestamp,
        );
        status_effect.owner_player_stats_snapshot = snapshot_owner_player_stats_for_buffs(
            source_entity,
            status_effect.source_skill_id,
            &self_effects,
            timestamp.timestamp_millis(),
            self,
        );
        status_effect.source_skill_runtime_snapshot = status_effect
            .source_skill_id
            .and_then(|skill_id| source_entity.skill_runtime_data.get(&skill_id).cloned());
    }

    pub fn refresh_status_effect_snapshots(
        &mut self,
        status_effect: &mut StatusEffectDetails,
        timestamp: DateTime<Utc>,
    ) {
        let source_entity = self.get_source_entity(status_effect.source_id);
        self.populate_status_effect_snapshots(status_effect, &source_entity, timestamp);
    }

    fn resolve_status_effect_target_entity(
        &self,
        target_id: u64,
        target_type: StatusEffectTargetType,
    ) -> Option<Entity> {
        match target_type {
            StatusEffectTargetType::Local => self.entities.get(&target_id).cloned(),
            StatusEffectTargetType::Party => self
                .id_tracker
                .borrow()
                .get_entity_id(target_id)
                .and_then(|entity_id| self.entities.get(&entity_id).cloned())
                .or_else(|| {
                    self.entities
                        .values()
                        .find(|entity| {
                            entity.entity_type == Player && entity.character_id == target_id
                        })
                        .cloned()
                })
                .or_else(|| {
                    self.removed_player_entities_by_character_id
                        .get(&target_id)
                        .cloned()
                })
                .or_else(|| {
                    self.removed_player_entities_by_name_class
                        .values()
                        .find(|entity| {
                            entity.entity_type == Player && entity.character_id == target_id
                        })
                        .cloned()
                }),
        }
    }

    fn resolve_status_effect_source_entity(&mut self, sed: &StatusEffectData) -> Entity {
        let source_entity = self.get_source_entity(sed.source_id);
        let Some(skill_buff) = SKILL_BUFF_DATA.get(&sed.status_effect_id) else {
            return source_entity;
        };

        if source_entity.entity_type != EntityType::Player {
            return source_entity;
        }

        let party_id = {
            let party_tracker = self.party_tracker.borrow();
            party_tracker
                .get_party_id_for_character(source_entity.character_id)
                .or_else(|| party_tracker.get_party_id_for_entity(source_entity.id))
        };

        let Some(party_id) = party_id else {
            return source_entity;
        };

        let owns_effect = |entity: &Entity| {
            entity.inspect_snapshot.as_ref().is_some_and(|snapshot| {
                snapshot
                    .derived_stats
                    .buff_id_ownership
                    .contains(&sed.status_effect_id)
                    || (skill_buff.unique_group > 0
                        && snapshot
                            .derived_stats
                            .buff_unique_group_ownership
                            .contains(&skill_buff.unique_group))
            })
        };

        let is_same_party = |entity: &Entity| {
            let party_tracker = self.party_tracker.borrow();
            party_tracker
                .get_party_id_for_character(entity.character_id)
                .or_else(|| party_tracker.get_party_id_for_entity(entity.id))
                .is_some_and(|tracked_party_id| tracked_party_id == party_id)
        };

        let same_identity = |lhs: &Entity, rhs: &Entity| {
            if lhs.character_id != 0 && rhs.character_id != 0 {
                lhs.character_id == rhs.character_id
            } else {
                lhs.id == rhs.id && lhs.class_id == rhs.class_id && lhs.name == rhs.name
            }
        };

        let mut eligible = self
            .entities
            .values()
            .chain(self.removed_player_entities_by_character_id.values())
            .filter(|entity| {
                entity.entity_type == Player && is_same_party(entity) && owns_effect(entity)
            })
            .cloned()
            .collect::<Vec<_>>();

        if eligible.is_empty() {
            return source_entity;
        }

        eligible.sort_by(|lhs, rhs| {
            let lhs_character_id = if lhs.character_id != 0 {
                lhs.character_id
            } else {
                u64::MAX
            };
            let rhs_character_id = if rhs.character_id != 0 {
                rhs.character_id
            } else {
                u64::MAX
            };

            lhs_character_id
                .cmp(&rhs_character_id)
                .then_with(|| lhs.id.cmp(&rhs.id))
                .then_with(|| lhs.class_id.cmp(&rhs.class_id))
                .then_with(|| lhs.name.cmp(&rhs.name))
        });
        eligible.dedup_by(|lhs, rhs| same_identity(lhs, rhs));

        if eligible.len() == 1 {
            return eligible.remove(0);
        }

        let use_unique_group = skill_buff.unique_group != 0;
        let effective_buff_key = if use_unique_group {
            skill_buff.unique_group
        } else {
            sed.status_effect_id
        };
        let round_robin_key = (party_id, use_unique_group, effective_buff_key);
        let next_index = self
            .status_effect_owner_round_robin
            .entry(round_robin_key)
            .or_insert(0);
        let selected_index = *next_index % eligible.len();
        *next_index = (selected_index + 1) % eligible.len();
        eligible.remove(selected_index)
    }

    fn build_and_register_status_effects(&mut self, seds: Vec<StatusEffectData>, target_id: u64) {
        let timestamp = Utc::now();
        for sed in seds.into_iter() {
            self.build_and_register_status_effect(&sed, target_id, timestamp, None);
        }
    }

    pub fn get_or_create_entity(&mut self, id: u64) -> Entity {
        if let Some(entity) = self.entities.get(&id) {
            return entity.clone();
        }

        let entity = Entity {
            id,
            entity_type: EntityType::Unknown,
            name: format!("{:x}", id),
            ..Default::default()
        };
        self.entities.insert(entity.id, entity.clone());
        entity
    }

    pub fn get_entity_ref(&self, id: u64) -> Option<&Entity> {
        self.entities.get(&id)
    }

    fn apply_pending_inspect_result_for_name(&mut self, name: &str) {
        let Some(result) = self.pending_inspect_results_by_name.remove(name) else {
            return;
        };
        let _ = self.apply_inspect_result(result);
    }

    fn has_inspect_snapshot_for_character(&self, character_id: u64) -> bool {
        if character_id == 0 {
            return false;
        }

        if self.entities.values().any(|entity| {
            entity.entity_type == Player
                && entity.character_id == character_id
                && entity.inspect_snapshot.is_some()
                && !entity.inspect_stale
        }) {
            return true;
        }

        if self
            .removed_player_entities_by_character_id
            .get(&character_id)
            .is_some_and(|entity| entity.inspect_snapshot.is_some() && !entity.inspect_stale)
        {
            return true;
        }

        let Some(name) = self.get_resolved_player_name_for_character(character_id) else {
            return false;
        };

        self.has_inspect_snapshot_for_name(&name)
    }

    fn evaluate_stashed_player_inspect_state(
        &self,
        character_id: u64,
        name: &str,
        now: i64,
        force_refresh: bool,
        bootstrap_refresh_needed: bool,
    ) -> (bool, bool) {
        let mut already_ready = false;
        let mut should_request = bootstrap_refresh_needed || force_refresh;

        if let Some(entity) = self
            .removed_player_entities_by_character_id
            .get(&character_id)
            .filter(|entity| entity.name == name)
        {
            if entity.entity_type == Player {
                if !force_refresh
                    && !bootstrap_refresh_needed
                    && ((entity.inspect_snapshot.is_some() && !entity.inspect_stale)
                        || entity.inspect_ready_at_ms > now)
                {
                    already_ready = true;
                } else {
                    should_request |= entity.inspect_stale;
                }
            }
        }
        if !already_ready {
            for entity in self
                .removed_player_entities_by_name_class
                .values()
                .filter(|entity| {
                    entity.name == name
                        && (character_id == 0 || entity.character_id == character_id)
                })
            {
                if entity.entity_type != Player {
                    continue;
                }
                if !force_refresh
                    && !bootstrap_refresh_needed
                    && ((entity.inspect_snapshot.is_some() && !entity.inspect_stale)
                        || entity.inspect_ready_at_ms > now)
                {
                    already_ready = true;
                    break;
                }
                should_request |= entity.inspect_stale;
            }
        }

        (already_ready, should_request)
    }

    fn mark_matching_player_request_state(
        &mut self,
        character_id: u64,
        name: &str,
        requested: bool,
    ) {
        for entity in self.entities.values_mut().filter(|entity| {
            entity.entity_type == Player
                && (entity.character_id == character_id || entity.name == name)
        }) {
            entity.inspect_requested = requested;
        }
        self.update_removed_player_copies_by_character(character_id, |entity| {
            if entity.name == name {
                entity.inspect_requested = requested;
            }
        });
        self.update_removed_player_copies_by_name(name, |entity| {
            if character_id == 0 || entity.character_id == 0 || entity.character_id == character_id
            {
                entity.inspect_requested = requested;
            }
        });
    }

    fn update_removed_player_copies_by_character<F>(&mut self, character_id: u64, mut update: F)
    where
        F: FnMut(&mut Entity),
    {
        if character_id == 0 {
            return;
        }
        if let Some(entity) = self
            .removed_player_entities_by_character_id
            .get_mut(&character_id)
        {
            update(entity);
        }
        for entity in self
            .removed_player_entities_by_name_class
            .values_mut()
            .filter(|entity| entity.character_id == character_id)
        {
            update(entity);
        }
    }

    fn update_removed_player_copies_by_name<F>(&mut self, name: &str, mut update: F)
    where
        F: FnMut(&mut Entity),
    {
        if name.is_empty() {
            return;
        }
        for entity in self
            .removed_player_entities_by_character_id
            .values_mut()
            .filter(|entity| entity.name == name)
        {
            update(entity);
        }
        for entity in self
            .removed_player_entities_by_name_class
            .values_mut()
            .filter(|entity| entity.name == name)
        {
            update(entity);
        }
    }

    fn stash_removed_player_entity(&mut self, entity: &Entity) {
        if entity.entity_type != Player {
            return;
        }

        if entity.character_id != 0 {
            self.removed_player_entities_by_character_id
                .insert(entity.character_id, entity.clone());
        }
        if !entity.name.is_empty() {
            self.removed_player_entities_by_name_class
                .insert((entity.name.clone(), entity.class_id), entity.clone());
        }
    }

    fn take_removed_reconnect_entity(
        &mut self,
        character_id: u64,
        name: &str,
        class_id: u32,
    ) -> Option<Entity> {
        let removed_by_character = if character_id != 0 {
            self.removed_player_entities_by_character_id
                .remove(&character_id)
        } else {
            None
        };

        let removed_by_name = if !name.is_empty() {
            self.removed_player_entities_by_name_class
                .remove(&(name.to_string(), class_id))
        } else {
            None
        };

        let removed_entity = removed_by_character.or(removed_by_name)?;

        if removed_entity.character_id != 0 {
            self.removed_player_entities_by_character_id
                .remove(&removed_entity.character_id);
        }
        if !removed_entity.name.is_empty() {
            self.removed_player_entities_by_name_class
                .remove(&(removed_entity.name.clone(), removed_entity.class_id));
        }

        Some(removed_entity)
    }

    fn is_gate_tracked_character(&self, character_id: u64) -> bool {
        character_id != 0
            && (self
                .entities
                .values()
                .any(|entity| entity.entity_type == Player && entity.character_id == character_id)
                || self
                    .removed_player_entities_by_character_id
                    .contains_key(&character_id))
    }

    pub fn get_resolved_player_name_for_character(&self, character_id: u64) -> Option<String> {
        if character_id == 0 {
            return None;
        }

        if let Some(name) = self
            .entities
            .values()
            .find(|entity| {
                entity.entity_type == Player
                    && entity.character_id == character_id
                    && is_resolved_player_name(&entity.name)
            })
            .map(|entity| entity.name.clone())
        {
            return Some(name);
        }

        if let Some(name) = self
            .removed_player_entities_by_character_id
            .get(&character_id)
            .filter(|entity| is_resolved_player_name(&entity.name))
            .map(|entity| entity.name.clone())
        {
            return Some(name);
        }

        self.character_id_to_name
            .get(&character_id)
            .filter(|name| is_resolved_player_name(name))
            .cloned()
    }
}

pub fn get_current_and_max_hp(stat_pair: &Vec<StatPair>) -> (i64, i64) {
    let mut hp: Option<i64> = None;
    let mut max_hp: Option<i64> = None;

    for pair in stat_pair {
        match pair.stat_type as u32 {
            1 => hp = Some(pair.value),
            27 => max_hp = Some(pair.value),
            _ => {}
        }
        if hp.is_some() && max_hp.is_some() {
            break;
        }
    }

    (hp.unwrap_or_default(), max_hp.unwrap_or_default())
}

fn get_npc_entity_type_name_grade_bars(
    npc: &NpcStruct,
    max_hp: i64,
) -> (EntityType, String, String, Option<u32>) {
    if let Some(esther) = get_esther_from_npc_id(npc.type_id) {
        return (EntityType::Esther, esther.name, "none".to_string(), None);
    }

    if let Some((_, npc_info)) = NPC_DATA.get_key_value(&npc.type_id) {
        let hp_bars = if npc_info.hp_bars > 1 {
            Some(npc_info.hp_bars)
        } else {
            None
        };
        let npc_name = npc_info.name.clone().unwrap_or_default();
        if (npc_info.grade == "boss"
            || npc_info.grade == "raid"
            || npc_info.grade == "epic_raid"
            || npc_info.grade == "commander")
            && max_hp > 10_000
            && !npc_name.is_empty()
            && !npc_name.contains('_')
            && npc_name.is_ascii()
        {
            (Boss, npc_name.clone(), npc_info.grade.clone(), hp_bars)
        } else {
            (
                EntityType::Npc,
                npc_name.clone(),
                npc_info.grade.clone(),
                hp_bars,
            )
        }
    } else {
        (
            EntityType::Npc,
            format!("{:x}", npc.object_id),
            "none".to_string(),
            None,
        )
    }
}

fn get_esther_from_npc_id(npc_id: u32) -> Option<Esther> {
    ESTHER_DATA
        .iter()
        .find(|esther| esther.npc_ids.contains(&npc_id))
        .cloned()
}

pub fn get_skill_class_id(skill_id: &u32) -> u32 {
    if let Some(skill) = SKILL_DATA.get(skill_id) {
        skill.class_id
    } else {
        0
    }
}

fn truncate_gear_level(gear_level: f32) -> f32 {
    f32::trunc(gear_level * 100.) / 100.
}

pub fn is_resolved_player_name(name: &str) -> bool {
    !name.is_empty() && name != "You" && !name.starts_with('0')
}

fn apply_inspect_payload_to_entity(
    entity: &mut Entity,
    snapshot: &InspectSnapshot,
    info: &InspectInfo,
    result: Arc<PKTPCInspectResult>,
) {
    if snapshot.gear_level > 0.0 {
        entity.gear_level = snapshot.gear_level;
    }
    entity.stats = snapshot.stat_pairs.clone();
    entity.inspect_requested = false;
    entity.inspect_ready_at_ms = 0;
    entity.inspect_stale = false;
    entity.inspect_snapshot = Some(snapshot.clone());
    entity.inspect_info = Some(info.clone());
    entity.inspect_result = Some(result);
}

fn merge_player_identity_state(target: &mut Entity, previous: Entity) {
    target.inspect_requested = previous.inspect_requested;
    target.inspect_ready_at_ms = previous.inspect_ready_at_ms;
    target.inspect_stale = previous.inspect_stale;
    target.inspect_info = previous.inspect_info;
    target.inspect_result = previous.inspect_result;
    target.inspect_snapshot = previous.inspect_snapshot;
    target.skill_runtime_data = previous.skill_runtime_data;
    target.stance = previous.stance;
    target.identity_gauge1 = previous.identity_gauge1;
    target.identity_gauge2 = previous.identity_gauge2;
    target.identity_gauge3 = previous.identity_gauge3;
    target.identity_gauge1_prev = previous.identity_gauge1_prev;
    target.identity_gauge2_prev = previous.identity_gauge2_prev;
    target.identity_gauge3_prev = previous.identity_gauge3_prev;
    target.identity_last_skill_start_id = previous.identity_last_skill_start_id;
    target.identity_last_skill_start_at_ms = previous.identity_last_skill_start_at_ms;
    target.destroyer_recent_consumed_cores = previous.destroyer_recent_consumed_cores;
    target.destroyer_recent_consumed_at_ms = previous.destroyer_recent_consumed_at_ms;
}

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub id: u64,
    pub entity_type: EntityType,
    pub name: String,
    pub npc_id: u32,
    pub hp_bars: Option<u32>,
    pub class_id: u32,
    pub gear_level: f32,
    pub character_id: u64,
    pub owner_id: u64,
    pub skill_effect_id: u32,
    pub skill_id: u32,
    pub stats: HashMap<u8, i64>,
    pub inspect_requested: bool,
    pub inspect_ready_at_ms: i64,
    pub inspect_stale: bool,
    pub inspect_info: Option<InspectInfo>,
    pub inspect_result: Option<Arc<PKTPCInspectResult>>,
    pub inspect_snapshot: Option<InspectSnapshot>,
    pub skill_runtime_data: HashMap<u32, SkillRuntimeData>,
    pub stance: u8,
    pub identity_gauge1: u32,
    pub identity_gauge2: u32,
    pub identity_gauge3: u32,
    pub identity_gauge1_prev: u32,
    pub identity_gauge2_prev: u32,
    pub identity_gauge3_prev: u32,
    pub identity_last_skill_start_id: u32,
    pub identity_last_skill_start_at_ms: i64,
    pub destroyer_recent_consumed_cores: i32,
    pub destroyer_recent_consumed_at_ms: i64,
    pub grade: String,
    pub push_immune: bool,
    pub level: u16,
}

impl Entity {
    pub fn runtime_state(&self) -> RuntimeState {
        let stat_value = |name: &str| -> i64 {
            let Some(stat_id) = STAT_TYPE_MAP
                .get(name)
                .copied()
                .and_then(|stat_id| u8::try_from(stat_id).ok())
            else {
                return 0;
            };
            self.stats.get(&stat_id).copied().unwrap_or_default()
        };
        RuntimeState {
            identity_stance: self.stance,
            identity_gauge1: self.identity_gauge1,
            identity_gauge2: self.identity_gauge2,
            identity_gauge3: self.identity_gauge3,
            identity_gauge1_prev: self.identity_gauge1_prev,
            identity_gauge2_prev: self.identity_gauge2_prev,
            identity_gauge3_prev: self.identity_gauge3_prev,
            last_skill_start_id: self.identity_last_skill_start_id,
            last_skill_start_at_ms: self.identity_last_skill_start_at_ms,
            destroyer_recent_consumed_cores: self.destroyer_recent_consumed_cores,
            destroyer_recent_consumed_at_ms: self.destroyer_recent_consumed_at_ms,
            current_hp: stat_value("hp"),
            max_hp: stat_value("max_hp"),
            current_mp: stat_value("mp"),
            max_mp: stat_value("max_mp"),
            combat_mp_recovery: stat_value("combat_mp_recovery"),
            ..Default::default()
        }
    }
}

impl SkillOptionSnapshot {
    pub fn from_skill_option_data(skill_option_data: &meter_defs::types::SkillOptionData) -> Self {
        Self {
            layer_index: skill_option_data.layer_index,
            start_stage_index: skill_option_data.start_stage_index,
            transit_index: skill_option_data.transit_index,
            stage_start_time: skill_option_data.stage_start_time,
            farmost_dist: skill_option_data.farmost_dist,
            tripod_index: skill_option_data
                .tripod_index
                .as_ref()
                .map(|tripod_index| TripodIndex {
                    first: tripod_index.first,
                    second: tripod_index.second,
                    third: tripod_index.third,
                }),
            tripod_level: skill_option_data
                .tripod_level
                .as_ref()
                .map(|tripod_level| TripodLevel {
                    first: tripod_level.first,
                    second: tripod_level.second,
                    third: tripod_level.third,
                }),
        }
    }
}

fn populate_skill_runtime_data(skill_runtime: &mut SkillRuntimeData, skill_id: u32) {
    skill_runtime.cached_critical_hit_damage_bonus = 0.0;
    skill_runtime.cached_critical_rate_bonus = 0.0;
    skill_runtime.cached_attack_speed_bonus = 0.0;
    skill_runtime
        .cached_critical_hit_damage_bonus_per_skill_effect
        .clear();
    skill_runtime
        .cached_critical_rate_bonus_per_skill_effect
        .clear();
    skill_runtime.cached_directional_mask = None;
    skill_runtime.cached_identity_category = None;
    skill_runtime.buff_stat_changes.clear();
    skill_runtime.buff_param_changes.clear();
    skill_runtime.buff_added_stats.clear();
    skill_runtime.added_chain_combat_effects.clear();
    skill_runtime.removed_chain_combat_effects.clear();
    skill_runtime.changed_combat_effects.clear();
    skill_runtime.addon_skill_feature_ids.clear();

    let Some(skill_option_data) = skill_runtime.skill_option_data.as_ref() else {
        return;
    };
    let Some(skill_feature) = EXTERNAL_SKILL_FEATURE_DATA.get(&skill_id) else {
        return;
    };
    let Some(tripod_index) = skill_option_data.tripod_index.as_ref() else {
        return;
    };
    let selected_tripods = [
        ("first", u32::from(tripod_index.first), 0_u32),
        ("second", u32::from(tripod_index.second), 3_u32),
        ("third", u32::from(tripod_index.third), 6_u32),
    ];

    for (row_name, selected_index, key_offset) in selected_tripods {
        let tripod_key = selected_index + key_offset;
        if tripod_key == 0 {
            continue;
        }
        let Some(tripod) = skill_feature.tripods.get(&tripod_key) else {
            let skill_name = SKILL_DATA
                .get(&skill_id)
                .and_then(|skill| skill.name.as_deref())
                .unwrap_or("<unknown>");
            warn!(
                "missing tripod row '{}' for skill '{}' ({}): selected index {}, key {}",
                row_name, skill_name, skill_id, selected_index, tripod_key
            );
            continue;
        };
        for entry in &tripod.entries {
            if (entry.level > 0 && entry.level != 1)
                || entry.target_mode_type.eq_ignore_ascii_case("b")
            {
                continue;
            }
            match entry.feature_type.as_str() {
                "change_dam_critical" => {
                    if entry.parameters.len() > 1 {
                        let skill_effect_id = entry.parameters[0] as u32;
                        let value = entry.parameters[1] as f64 / 10000.0;
                        if skill_effect_id == 0 {
                            skill_runtime.cached_critical_hit_damage_bonus += value;
                        } else {
                            *skill_runtime
                                .cached_critical_hit_damage_bonus_per_skill_effect
                                .entry(skill_effect_id)
                                .or_default() += value;
                        }
                    }
                }
                "change_attack_mask" => {
                    if entry.parameters.len() > 1 {
                        skill_runtime.cached_directional_mask = Some(entry.parameters[1] as i32);
                    }
                }
                "change_dam_critical_rate" => {
                    if entry.parameters.len() > 1 {
                        let skill_effect_id = entry.parameters[0] as u32;
                        let value = entry.parameters[1] as f64 / 10000.0;
                        if skill_effect_id == 0 {
                            skill_runtime.cached_critical_rate_bonus += value;
                        } else {
                            *skill_runtime
                                .cached_critical_rate_bonus_per_skill_effect
                                .entry(skill_effect_id)
                                .or_default() += value;
                        }
                    }
                }
                "change_attack_stage_speed" => {
                    if let Some(value) = entry.parameters.first() {
                        skill_runtime.cached_attack_speed_bonus += *value as f64 / 100.0;
                    }
                }
                "change_buff_stat" => {
                    if entry.parameters.len() >= 4 {
                        let buff_id = entry.parameters[1] as u32;
                        let relative = !entry.parameter_type.eq_ignore_ascii_case("absolute");
                        let changes = skill_runtime.buff_stat_changes.entry(buff_id).or_default();
                        for pair in entry.parameters[2..].chunks(2) {
                            if pair.len() != 2 {
                                continue;
                            }
                            if let Some(name) = stat_type_name_from_id(pair[0] as u32) {
                                changes.insert(name.clone(), (pair[1], relative));
                            }
                        }
                    }
                }
                "change_buff_param" => {
                    if entry.parameters.len() >= 4 {
                        let buff_id = entry.parameters[1] as u32;
                        let relative = entry.parameters[2] != 0;
                        skill_runtime
                            .buff_param_changes
                            .insert(buff_id, (entry.parameters[3..].to_vec(), relative));
                    }
                }
                "add_buff_stat" => {
                    if entry.parameters.len() >= 4 {
                        let buff_id = entry.parameters[1] as u32;
                        let stat_name =
                            stat_type_name_from_id(entry.parameters[2] as u32).unwrap_or_default();
                        skill_runtime
                            .buff_added_stats
                            .entry(buff_id)
                            .or_default()
                            .push(crate::models::PassiveOption {
                                option_type: "stat".to_string(),
                                key_stat: stat_name,
                                key_index: 0,
                                value: entry.parameters[3] as i32,
                            });
                    }
                }
                "add_chain_combat_effect" => {
                    if entry.parameters.len() > 1 {
                        skill_runtime
                            .added_chain_combat_effects
                            .entry(entry.parameters[0] as u32)
                            .or_default()
                            .push(entry.parameters[1] as u32);
                    }
                }
                "remove_chain_combat_effect" => {
                    if let Some(value) = entry.parameters.first() {
                        skill_runtime
                            .removed_chain_combat_effects
                            .push(*value as u32);
                    }
                }
                "change_combat_effect_arg" => {
                    if entry.parameters.len() >= 2 {
                        let skill_effect_id = entry.parameters[0] as u32;
                        let relative = entry.parameter_type.eq_ignore_ascii_case("relative");
                        skill_runtime
                            .changed_combat_effects
                            .entry(skill_effect_id)
                            .or_default()
                            .push(ChangedCombatEffect {
                                combat_effect_id: entry.parameters[1] as u32,
                                values: entry.parameters[2..].to_vec(),
                                relative,
                            });
                    }
                }
                "change_identity_category" => {
                    if let Some(value) = entry.parameters.first() {
                        skill_runtime.cached_identity_category = Some(value.to_string());
                    }
                }
                "add_skill_feature" => {
                    if let Some(value) = entry.parameters.first() {
                        skill_runtime.addon_skill_feature_ids.push(*value as u32);
                    }
                }
                _ => {}
            }
        }
    }
}

fn inspect_snapshot_from_result(result: &PKTPCInspectResult) -> InspectSnapshot {
    InspectSnapshot {
        gear_level: truncate_gear_level(result.gear_level1.max(result.gear_level2)),
        stat_pairs: result
            .stat_pairs
            .iter()
            .map(|sp| (sp.stat_type, sp.value))
            .collect(),
        derived_stats: derive_inspect_stats(result),
        addon_values: result
            .addon_value_datas
            .iter()
            .map(|addon| InspectAddonValue {
                addon_type: addon.addon_type,
                value: addon.value,
            })
            .collect(),
        engravings: result
            .engraving_datas
            .iter()
            .map(|engraving| InspectEngraving {
                id: engraving.id,
                unknown: engraving.unknown,
                level: engraving.level,
            })
            .collect(),
        equipped_items: result
            .equipped_items
            .iter()
            .map(inspect_item_snapshot_from_item_data)
            .collect(),
        equipped_gems: result
            .equipped_gems
            .iter()
            .map(inspect_item_snapshot_from_item_data)
            .collect(),
        cards: result
            .card_datas
            .iter()
            .map(|card| InspectCardSnapshot {
                id: card.id,
                awakening_level: card.awakening_level,
            })
            .collect(),
        stigma_layouts: result
            .stigma_layout_datas
            .iter()
            .map(|stigma| InspectStigmaLayoutSnapshot {
                stigma_id: stigma.stigma_id,
                stigma_level: stigma.stigma_level,
                stigma_rank: stigma.stigma_rank,
            })
            .collect(),
        ark_grid_cores: result
            .ark_grid_cores
            .core_entries
            .iter()
            .enumerate()
            .map(|(index, options)| InspectArkGridCoreSnapshot {
                core_id: result
                    .ark_grid_cores
                    .core_ids
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                base_id: result
                    .ark_grid_cores
                    .base_ids
                    .get(index)
                    .copied()
                    .unwrap_or_default(),
                options: options
                    .iter()
                    .map(|option| InspectArkGridCoreOptionSnapshot {
                        willpower_rank: option.willpower_rank,
                        item_id: option.item_id,
                        enabled: option.enabled == 1,
                        order_rank: option.order_rank,
                        slot_index: option.slot_index,
                        values: option
                            .values
                            .iter()
                            .map(|value| InspectArkGridCoreValueSnapshot {
                                option_id: value.option_id,
                                rank: value.rank,
                            })
                            .collect(),
                    })
                    .collect(),
            })
            .collect(),
        ark_passive_data: ark_passive_data_from_result(&result.ark_passive_tree_data_inspect),
    }
}

fn inspect_item_snapshot_from_item_data(item_data: &ItemData) -> InspectItemSnapshot {
    InspectItemSnapshot {
        unique_id: item_data.s64_2,
        raw_item_id: item_data.u32_1,
        raw_hone_level: item_data.u16_0,
        raw_slot_index: item_data.u16_1,
        data_type: item_data.item_data_typed.as_ref().map(|typed| typed.b_0),
        has_equippable_item_data: item_data
            .item_data_typed
            .as_ref()
            .and_then(|typed| typed.equippable_item_data.as_ref())
            .is_some(),
        has_ark_grid_gem_data: item_data
            .item_data_typed
            .as_ref()
            .and_then(|typed| typed.ark_grid_gem_data.as_ref())
            .is_some(),
    }
}

fn inspect_info_from_result(result: &PKTPCInspectResult) -> InspectInfo {
    let ark_passive_data = ark_passive_data_from_result(&result.ark_passive_tree_data_inspect);
    let engravings = (!result.engraving_datas.is_empty()).then(|| {
        result
            .engraving_datas
            .iter()
            .map(|engraving| engraving.id)
            .collect()
    });

    InspectInfo {
        combat_power: None,
        ark_passive_enabled: ark_passive_data.is_some(),
        ark_passive_data,
        engravings,
        gems: None,
        loadout_snapshot: None,
    }
}

fn ark_passive_data_from_result(tree: &ArkPassiveTreeDataInspect) -> Option<ArkPassiveData> {
    if tree.ark_passive_node_datas.is_empty() {
        return None;
    }

    let sections = tree
        .ark_passive_node_datas
        .iter()
        .map(|section| {
            section
                .iter()
                .map(|node| ArkPassiveNode {
                    id: node.ark_passive_id,
                    lv: node.points.unwrap_or(1).min(u8::MAX as u32) as u8,
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    let non_empty = |index: usize| {
        sections
            .get(index)
            .filter(|section| !section.is_empty())
            .cloned()
    };

    Some(ArkPassiveData {
        evolution: non_empty(0),
        enlightenment: non_empty(1),
        leap: non_empty(2),
    })
}
