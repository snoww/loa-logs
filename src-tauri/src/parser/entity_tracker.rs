use crate::parser::id_tracker::IdTracker;
use crate::parser::models::EntityType::*;
use crate::parser::models::{EntityType, Esther, ESTHER_DATA, NPC_DATA, SKILL_DATA, STAT_TYPE_MAP};
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::{build_status_effect, StatusEffectTargetType, StatusTracker};
use chrono::Utc;
use hashbrown::HashMap;
use pcap_test::packets::common::StatPair;
use pcap_test::packets::definitions::*;
use pcap_test::packets::structures::{NpcData, StatusEffectData};
use std::cell::RefCell;
use std::rc::Rc;

pub struct EntityTracker {
    id_tracker: Rc<RefCell<IdTracker>>,
    party_tracker: Rc<RefCell<PartyTracker>>,
    status_tracker: Rc<RefCell<StatusTracker>>,

    pub entities: HashMap<u64, Entity>,

    pub local_player_id: u64,
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
            local_player_id: 0,
        }
    }

    pub fn init_env(&mut self, pkt: PKTInitEnv) -> Entity {
        if self.local_player_id == 0 {
            self.local_player_id = pkt.player_id;
        } else {
            let party_id = self.party_tracker.borrow_mut().entity_id_to_party_id.get(&self.local_player_id).cloned();
            if let Some(party_id) = party_id {
                self.party_tracker.borrow_mut().entity_id_to_party_id.remove(&self.local_player_id);
                self.party_tracker.borrow_mut().entity_id_to_party_id.insert(pkt.player_id, party_id);
            }
        }

        let mut local_player = match self.entities.get(&self.local_player_id).cloned() {
            Some(player) => player,
            None => Entity {
                entity_type: PLAYER,
                name: "You".to_string(),
                class_id: 0,
                gear_level: 0.0,
                character_id: 0,
                ..Default::default()
            },
        };

        local_player.id = pkt.player_id;
        self.local_player_id = pkt.player_id;

        self.entities.clear();
        self.entities.insert(local_player.id, local_player.clone());
        self.id_tracker.borrow_mut().clear();
        self.status_tracker.borrow_mut().clear();
        if local_player.character_id > 0 {
            self.id_tracker
                .borrow_mut()
                .add_mapping(local_player.character_id, local_player.id);
        }
        if local_player.character_id > 0 {
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
            gear_level: f32::trunc(pkt.gear_level * 100.) / 100.,
            character_id: pkt.character_id,
            ..Default::default()
        };

        self.local_player_id = player.id;
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
        if self.local_player_id != 0 {
            return;
        }
        self.entities.entry(self.local_player_id).and_modify(|e| {
            if pkt.account_character_id1 < pkt.account_character_id2 {
                e.character_id = pkt.account_character_id1;
            } else {
                e.character_id = pkt.account_character_id2;
            }
        });
    }

    pub fn new_pc(&mut self, pkt: PKTNewPC) -> Entity {
        let entity = Entity {
            id: pkt.pc_struct.player_id,
            entity_type: PLAYER,
            name: pkt.pc_struct.name.clone(),
            class_id: pkt.pc_struct.class_id as u32,
            gear_level: f32::trunc(pkt.pc_struct.gear_level * 100.) / 100.,
            character_id: pkt.pc_struct.character_id,
            ..Default::default()
        };
        self.entities.insert(entity.id, entity.clone());
        let old_entity_id = self.id_tracker.borrow_mut().get_entity_id(pkt.pc_struct.character_id);
        if let Some(old_entity_id) = old_entity_id {
            self.party_tracker.borrow_mut().change_entity_id(old_entity_id, entity.id);
        }
        self.id_tracker
            .borrow_mut()
            .add_mapping(pkt.pc_struct.character_id, pkt.pc_struct.player_id);
        self.party_tracker
            .borrow_mut()
            .complete_entry(pkt.pc_struct.character_id, pkt.pc_struct.player_id);
        self.status_tracker.borrow_mut().new_pc(&pkt, self.local_player_id);
        entity
    }

    pub fn new_npc(&mut self, pkt: PKTNewNpc) -> Entity {
        let (entity_type, name) = get_npc_entity_type_and_name(&pkt.npc_data);
        let npc = Entity {
            id: pkt.npc_data.object_id,
            entity_type,
            name,
            npc_id: pkt.npc_data.type_id,
            ..Default::default()
        };
        self.entities.insert(npc.id, npc.clone());
        self.status_tracker.borrow_mut().remove_local_object(npc.id);
        self.build_and_register_status_effects(pkt.npc_data.status_effect_datas, npc.id);
        npc
    }

    pub fn new_npc_summon(&mut self, pkt: PKTNewNpcSummon) -> Entity {
        let (_entity_type, name) = get_npc_entity_type_and_name(&pkt.npc_data);
        let npc = Entity {
            id: pkt.npc_data.object_id,
            entity_type: SUMMON,
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
        for sed in pkt.status_effect_datas {
            let source_id = if pkt.player_id_on_refresh != 0 {
                pkt.player_id_on_refresh
            } else {
                sed.source_id
            };
            if let Some(entity) = self.entities.get(&source_id) {
                let status_effect = build_status_effect(
                    sed,
                    pkt.character_id,
                    entity.id,
                    StatusEffectTargetType::Party,
                );
                self.status_tracker
                    .borrow_mut()
                    .register_status_effect(status_effect);
            }
        }
    }

    pub fn party_status_effect_remove(&mut self, pkt: PKTPartyStatusEffectRemoveNotify) {
        for se_id in pkt.status_effect_ids {
            self.status_tracker.borrow_mut().remove_status_effect(
                pkt.character_id,
                se_id,
                StatusEffectTargetType::Party,
            );
        }
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

    pub fn party_info(&mut self, pkt: PKTPartyInfo) {
        let local_player = self.entities.get(&self.local_player_id).unwrap();
        if pkt.member_datas.len() == 1 {
            if let Some(first) = pkt.member_datas.get(0) {
                if first.name == local_player.name {
                    self.party_tracker
                        .borrow_mut()
                        .remove(pkt.party_instance_id, first.name.to_string());
                    return;
                }
            }
        }

        let local_player_id = local_player.character_id;
        self.party_tracker
            .borrow_mut()
            .remove_party_mappings(pkt.party_instance_id);
        for member in pkt.member_datas {
            if member.character_id == local_player_id {
                self.party_tracker
                    .borrow_mut()
                    .set_name(member.name.clone());
            }
            let entity = self
                .id_tracker
                .borrow_mut()
                .get_entity_id(member.character_id);

            if let Some(entity_id) = entity
            {
                if let Some(entity) = self.entities.get_mut(&entity_id) {
                    if entity.entity_type == PLAYER && entity.name == member.name {
                        entity.gear_level = member.gear_level;
                        entity.name = member.name.to_string();
                        entity.class_id = member.class_id as u32;
                    }
                }

                self.party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    member.character_id,
                    entity_id,
                    Some(member.name.to_string()),
                );
            } else {
                self.party_tracker.borrow_mut().add(
                    pkt.raid_instance_id,
                    pkt.party_instance_id,
                    member.character_id,
                    0,
                    Some(member.name.to_string()),
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
                entity_type: NPC,
                name: format!("{:x}", id),
                ..Default::default()
            };
            self.entities.insert(entity.id, entity.clone());
            entity
        }
    }

    pub fn guess_is_player(&mut self, mut entity: Entity, skill_id: u32) -> Entity {
        let class_id = get_skill_class_id(skill_id);
        if class_id != 0 {
            if entity.entity_type == PLAYER {
                if entity.class_id == class_id {
                    return entity;
                }
                entity.class_id = class_id;
            } else {
                entity.entity_type = PLAYER;
                entity.class_id = class_id;
            }
        }
        self.entities.insert(entity.id, entity.clone());
        entity
    }

    pub fn build_and_register_status_effect(&mut self, sed: &StatusEffectData, target_id: u64) {
        let source_entity = self.get_source_entity(sed.source_id);
        let status_effect = build_status_effect(
            sed.clone(),
            target_id,
            source_entity.id,
            StatusEffectTargetType::Local,
        );
        self.status_tracker
            .borrow_mut()
            .register_status_effect(status_effect);
    }

    fn build_and_register_status_effects(&mut self, seds: Vec<StatusEffectData>, target_id: u64) {
        for sed in seds.into_iter() {
            let source_entity = self.get_source_entity(sed.source_id);
            let status_effect = build_status_effect(
                sed,
                target_id,
                source_entity.id,
                StatusEffectTargetType::Local,
            );
            self.status_tracker
                .borrow_mut()
                .register_status_effect(status_effect);
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
    let mut hp = 0;
    let mut max_hp = 0;

    let stat_type_hp = STAT_TYPE_MAP["hp"];
    let stat_type_max_hp = STAT_TYPE_MAP["max_hp"];

    for pair in stat_pair {
        match pair.stat_type as u32 {
            x if x == stat_type_hp => hp = pair.value,
            x if x == stat_type_max_hp => max_hp = pair.value,
            _ => {}
        }
    }

    (hp, max_hp)
}

fn get_npc_entity_type_and_name(npc: &NpcData) -> (EntityType, String) {
    if let Some(esther) = get_esther_from_npc_id(npc.type_id) {
        return (ESTHER, esther.name);
    }

    if let Some((_, npc_info)) = NPC_DATA.get_key_value(&npc.type_id) {
        if (npc_info.grade == "boss"
            || npc_info.grade == "raid"
            || npc_info.grade == "epic_raid"
            || npc_info.grade == "commander")
            // && npc.max_hp > 10_000 // todo
            && !npc_info.name.contains('_')
            && npc_info.name.chars().all(|c| c.is_alphabetic() || c.is_ascii())
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

fn get_skill_class_id(skill_id: u32) -> u32 {
    if let Some(skill) = SKILL_DATA.get(&(skill_id as i32)) {
        skill.class_id
    } else {
        0
    }
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
