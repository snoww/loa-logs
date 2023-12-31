use crate::parser::id_tracker::IdTracker;
use crate::parser::models::EntityType::*;
use crate::parser::models::{EntityType, Esther, ESTHER_DATA, NPC_DATA, SKILL_DATA};
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{build_status_effect, StatusEffect, StatusEffectTargetType, StatusTracker};

use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::common::StatPair;
use meter_core::packets::definitions::*;
use meter_core::packets::structures::{NpcData, StatusEffectData};
use std::cell::RefCell;
use std::rc::Rc;
use chrono::{DateTime, Utc};

pub struct EntityTracker {
    id_tracker: Rc<RefCell<IdTracker>>,
    party_tracker: Rc<RefCell<PartyTracker>>,
    status_tracker: Rc<RefCell<StatusTracker>>,

    pub entities: HashMap<u64, Entity>,

    pub local_entity_id: u64,
    pub local_character_id: u64,
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
            local_entity_id: 0,
            local_character_id: 0,
        }
    }

    pub fn init_env(&mut self, pkt: PKTInitEnv) -> Entity {
        if !self.local_entity_id == 0 {
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
                entity_type: PLAYER,
                name: "You".to_string(),
                class_id: 0,
                gear_level: 0.0,
                character_id: self.local_character_id,
                ..Default::default()
            });

        info!("init env: eid: {}->{}", self.local_entity_id, pkt.player_id);

        local_player.id = pkt.player_id;
        self.local_entity_id = pkt.player_id;

        self.entities.clear();
        self.entities.insert(local_player.id, local_player.clone());
        self.id_tracker.borrow_mut().clear();
        self.status_tracker.borrow_mut().clear();
        if local_player.character_id > 0 {
            self.id_tracker
                .borrow_mut()
                .add_mapping(local_player.character_id, local_player.id);
            self.party_tracker
                .borrow_mut()
                .complete_entry(local_player.character_id, local_player.id);
        }
        local_player
    }

    pub fn init_pc(&mut self, pkt: PKTInitPC) -> Entity {
        let player = Entity {
            id: pkt.player_id,
            entity_type: PLAYER,
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
        self.build_and_register_status_effects(pkt.status_effect_datas, player.id);
        player
    }

    pub fn migration_execute(&mut self, pkt: PKTMigrationExecute) {
        if self
            .id_tracker
            .borrow()
            .get_local_character_id(self.local_entity_id)
            != 0
        {
            return;
        }

        let char_id = if pkt.account_character_id1 < pkt.account_character_id2 {
            pkt.account_character_id1
        } else {
            pkt.account_character_id2
        };

        info!("character id: {}->{}", self.local_character_id, char_id);
        self.local_character_id = char_id;
        self.id_tracker
            .borrow_mut()
            .add_mapping(char_id, self.local_entity_id);

        self.entities
            .entry(self.local_entity_id)
            .and_modify(|e| {
                e.character_id = char_id;
            })
            .or_insert_with(|| Entity {
                entity_type: PLAYER,
                name: "You".to_string(),
                character_id: char_id,
                ..Default::default()
            });
    }

    pub fn new_pc(&mut self, pkt: PKTNewPC) -> Entity {
        let entity = Entity {
            id: pkt.pc_struct.player_id,
            entity_type: PLAYER,
            name: pkt.pc_struct.name.clone(),
            class_id: pkt.pc_struct.class_id as u32,
            gear_level: truncate_gear_level(pkt.pc_struct.avg_item_level),
            character_id: pkt.pc_struct.character_id,
            ..Default::default()
        };
        self.entities.insert(entity.id, entity.clone());
        let old_entity_id = self
            .id_tracker
            .borrow()
            .get_entity_id(pkt.pc_struct.character_id);
        if let Some(old_entity_id) = old_entity_id {
            self.party_tracker
                .borrow_mut()
                .change_entity_id(old_entity_id, entity.id);
        }
        self.id_tracker
            .borrow_mut()
            .add_mapping(pkt.pc_struct.character_id, pkt.pc_struct.player_id);
        self.party_tracker
            .borrow_mut()
            .complete_entry(pkt.pc_struct.character_id, pkt.pc_struct.player_id);
        // println!("party status: {:?}", self.party_tracker.borrow().character_id_to_party_id);
        let local_character_id = if self.local_character_id != 0 {
            self.local_character_id
        } else {
            self.id_tracker
                .borrow()
                .get_local_character_id(self.local_entity_id)
        };
        self.status_tracker
            .borrow_mut()
            .new_pc(pkt, local_character_id);
        entity
    }

    pub fn new_npc(&mut self, pkt: PKTNewNpc, max_hp: i64) -> Entity {
        let (entity_type, name) = get_npc_entity_type_and_name(&pkt.npc_struct, max_hp);
        let npc = Entity {
            id: pkt.npc_struct.object_id,
            entity_type,
            name,
            npc_id: pkt.npc_struct.type_id,
            ..Default::default()
        };
        self.entities.insert(npc.id, npc.clone());
        self.status_tracker.borrow_mut().remove_local_object(npc.id);
        self.build_and_register_status_effects(pkt.npc_struct.status_effect_datas, npc.id);
        npc
    }

    pub fn new_npc_summon(&mut self, pkt: PKTNewNpcSummon, max_hp: i64) -> Entity {
        let (entity_type, name) = get_npc_entity_type_and_name(&pkt.npc_data, max_hp);
        let entity_type = if entity_type == NPC {
            SUMMON
        } else {
            entity_type
        };
        let npc = Entity {
            id: pkt.npc_data.object_id,
            entity_type,
            name,
            npc_id: pkt.npc_data.type_id,
            owner_id: pkt.owner_id,
            ..Default::default()
        };
        self.entities.insert(npc.id, npc.clone());
        self.status_tracker.borrow_mut().remove_local_object(npc.id);
        self.build_and_register_status_effects(pkt.npc_data.status_effect_datas, npc.id);
        npc
    }

    pub fn party_status_effect_add(&mut self, pkt: PKTPartyStatusEffectAddNotify) {
        let timestamp = Utc::now();
        for sed in pkt.status_effect_datas {
            let source_id = if pkt.player_id_on_refresh != 0 {
                pkt.player_id_on_refresh
            } else {
                sed.source_id
            };
            let entity = self.get_source_entity(source_id);
            // println!("entity: {:?}", entity);
            let status_effect = build_status_effect(
                sed,
                pkt.character_id,
                entity.id,
                StatusEffectTargetType::Party,
                timestamp
            );
            self.status_tracker
                .borrow_mut()
                .register_status_effect(status_effect);
        }
    }

    pub fn party_status_effect_remove(&mut self, pkt: PKTPartyStatusEffectRemoveNotify) {
        self.status_tracker.borrow_mut().remove_status_effects(
            pkt.character_id,
            pkt.status_effect_ids,
            StatusEffectTargetType::Party,
        );
    }

    pub fn new_projectile(&mut self, pkt: PKTNewProjectile) {
        let projectile = Entity {
            id: pkt.projectile_info.projectile_id,
            entity_type: PROJECTILE,
            name: format!("{:x}", pkt.projectile_info.projectile_id),
            owner_id: pkt.projectile_info.owner_id,
            skill_id: pkt.projectile_info.skill_id,
            skill_effect_id: pkt.projectile_info.skill_effect,
            ..Default::default()
        };
        self.entities.insert(projectile.id, projectile);
    }

    pub fn new_trap(&mut self, pkt: PKTNewTrap) {
        let trap = Entity {
            id: pkt.trap_data.object_id,
            entity_type: PROJECTILE,
            name: format!("{:x}", pkt.trap_data.object_id),
            owner_id: pkt.trap_data.owner_id,
            skill_id: pkt.trap_data.skill_id,
            skill_effect_id: pkt.trap_data.skill_effect,
            ..Default::default()
        };
        self.entities.insert(trap.id, trap);
    }

    pub fn party_info(&mut self, pkt: PKTPartyInfo, local_players: &HashMap<u64, String>) {
        let mut unknown_local = if let Some(local_player) = self.entities.get(&self.local_entity_id)
        {
            local_player.name.is_empty() || local_player.name == "You" || local_player.name.starts_with('0')
        } else {
            true
        };

        self.party_tracker
            .borrow_mut()
            .remove_party_mappings(pkt.party_instance_id);

        for member in pkt.member_datas {
            if unknown_local && local_players.contains_key(&member.character_id) {
                if let Some(local_player) = self.entities.get_mut(&self.local_entity_id) {
                    unknown_local = false;
                    warn!(
                        "unknown local player, inferring from cache: {}",
                        member.name
                    );
                    local_player.class_id = member.class_id as u32;
                    local_player.gear_level = truncate_gear_level(member.gear_level);
                    local_player.name = member.name.clone();
                    local_player.character_id = member.character_id;
                    self.id_tracker
                        .borrow_mut()
                        .add_mapping(member.character_id, self.local_entity_id);
                    self.party_tracker
                        .borrow_mut()
                        .set_name(member.name.clone());
                }
            }

            let entity_id = self.id_tracker.borrow().get_entity_id(member.character_id);

            if let Some(entity_id) = entity_id {
                if let Some(entity) = self.entities.get_mut(&entity_id) {
                    if entity.entity_type == PLAYER && entity.name == member.name {
                        entity.gear_level = truncate_gear_level(member.gear_level);
                        entity.class_id = member.class_id as u32;
                    }
                }

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

    pub fn get_source_entity(&mut self, id: u64) -> Entity {
        let id = if let Some(entity) = self.entities.get(&id) {
            if entity.entity_type == PROJECTILE || entity.entity_type == SUMMON {
                entity.owner_id
            } else {
                id
            }
        } else {
            id
        };

        if let Some(entity) = self.entities.get(&id) {
            entity.clone()
        } else {
            let entity = Entity {
                id,
                entity_type: UNKNOWN,
                name: format!("{:x}", id),
                ..Default::default()
            };
            self.entities.insert(entity.id, entity.clone());
            entity
        }
    }

    pub fn guess_is_player(&mut self, entity: &mut Entity, skill_id: u32) {
        if (entity.entity_type != UNKNOWN && entity.entity_type != PLAYER)
            || (entity.entity_type == PLAYER && entity.class_id != 0)
        {
            return;
        }

        let class_id = get_skill_class_id(&(skill_id as i32));
        if class_id != 0 {
            if entity.entity_type == PLAYER {
                if entity.class_id == class_id {
                    return;
                }
                entity.class_id = class_id;
            } else {
                entity.entity_type = PLAYER;
                entity.class_id = class_id;
            }
        }
        self.entities.insert(entity.id, entity.clone());
    }

    pub fn build_and_register_status_effect(&mut self, sed: &StatusEffectData, target_id: u64, timestamp: DateTime<Utc>) -> StatusEffect {
        let source_entity = self.get_source_entity(sed.source_id);
        let status_effect = build_status_effect(
            sed.clone(),
            target_id,
            source_entity.id,
            StatusEffectTargetType::Local,
            timestamp
        );

        self.status_tracker
            .borrow_mut()
            .register_status_effect(status_effect.clone());

        status_effect
    }

    fn build_and_register_status_effects(&mut self, seds: Vec<StatusEffectData>, target_id: u64) {
        let timestamp = Utc::now();
        for sed in seds.into_iter() {
            self.build_and_register_status_effect(&sed, target_id, timestamp);
        }
    }

    pub fn get_or_create_entity(&mut self, id: u64) -> Entity {
        if let Some(entity) = self.entities.get(&id) {
            return entity.clone();
        }

        let entity = Entity {
            id,
            entity_type: UNKNOWN,
            name: format!("{:x}", id),
            ..Default::default()
        };
        self.entities.insert(entity.id, entity.clone());
        entity
    }
}

pub fn get_current_and_max_hp(stat_pair: &Vec<StatPair>) -> (i64, i64) {
    let mut hp: Option<i64> = None;
    let mut max_hp: Option<i64> = None;

    for pair in stat_pair {
        match pair.stat_type as u32 {
            x if x == 1 => hp = Some(pair.value),
            x if x == 27 => max_hp = Some(pair.value),
            _ => {}
        }
        if hp.is_some() && max_hp.is_some() {
            break;
        }
    }

    (hp.unwrap_or_default(), max_hp.unwrap_or_default())
}

fn get_npc_entity_type_and_name(npc: &NpcData, max_hp: i64) -> (EntityType, String) {
    if let Some(esther) = get_esther_from_npc_id(npc.type_id) {
        return (ESTHER, esther.name);
    }

    if let Some((_, npc_info)) = NPC_DATA.get_key_value(&npc.type_id) {
        if (npc_info.grade == "boss"
            || npc_info.grade == "raid"
            || npc_info.grade == "epic_raid"
            || npc_info.grade == "commander")
            && max_hp > 10_000
            && !npc_info.name.contains('_')
            && npc_info.name.chars().all(|c| c.is_ascii())
        {
            (BOSS, npc_info.name.clone())
        } else {
            (NPC, npc_info.name.clone())
        }
    } else {
        (NPC, format!("{:x}", npc.object_id))
    }
}

fn get_esther_from_npc_id(npc_id: u32) -> Option<Esther> {
    ESTHER_DATA
        .iter()
        .find(|esther| esther.npc_ids.contains(&npc_id))
        .cloned()
}

pub fn get_skill_class_id(skill_id: &i32) -> u32 {
    if let Some(skill) = SKILL_DATA.get(skill_id) {
        skill.class_id
    } else {
        0
    }
}

fn truncate_gear_level(gear_level: f32) -> f32 {
    f32::trunc(gear_level * 100.) / 100.
}

#[derive(Debug, Default, Clone)]
pub struct Entity {
    pub id: u64,
    pub entity_type: EntityType,
    pub name: String,
    pub npc_id: u32,
    pub class_id: u32,
    pub gear_level: f32,
    pub character_id: u64,
    pub owner_id: u64,
    pub skill_effect_id: u32,
    pub skill_id: u32,
}
