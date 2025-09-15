use crate::data::*;
use crate::live::id_tracker::IdTracker;
use crate::live::party_tracker::PartyTracker;
use crate::live::status_tracker::{
    build_status_effect, StatusEffectDetails, StatusEffectTargetType, StatusEffectType,
    StatusTracker,
};
use crate::local::{LocalInfo, LocalPlayer};
use crate::models::EntityType::*;
use crate::models::{EncounterEntity, EntityType, Esther};

use chrono::{DateTime, Utc};
use hashbrown::HashMap;
use log::{info, warn};
use meter_core::packets::definitions::*;
use meter_core::packets::structures::{NpcStruct, PCStruct, StatPair, StatusEffectData};
use std::cell::RefCell;
use std::rc::Rc;

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
            entity_type: Player,
            name: pkt.name,
            class_id: pkt.class_id as u32,
            gear_level: truncate_gear_level(pkt.gear_level),
            character_id: pkt.character_id,
            stats: pkt
                .stat_pairs
                .iter()
                .map(|sp| (sp.stat_type, sp.value))
                .collect(),
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
        let entity = Entity {
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

        self.entities.insert(entity.id, entity.clone());
        let old_entity_id = self
            .id_tracker
            .borrow()
            .get_entity_id(pc_struct.character_id);
        if let Some(old_entity_id) = old_entity_id {
            self.party_tracker
                .borrow_mut()
                .change_entity_id(old_entity_id, entity.id);
        }
        self.id_tracker
            .borrow_mut()
            .add_mapping(pc_struct.character_id, pc_struct.player_id);
        self.party_tracker
            .borrow_mut()
            .complete_entry(pc_struct.character_id, pc_struct.player_id);
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
            .new_pc(pc_struct, local_character_id);
        entity
    }

    pub fn new_npc(&mut self, pkt: PKTNewNpc, max_hp: i64) -> Entity {
        let (entity_type, name, grade) = get_npc_entity_type_name_grade(&pkt.npc_struct, max_hp);
        let npc = Entity {
            id: pkt.npc_struct.object_id,
            entity_type,
            name,
            grade,
            npc_id: pkt.npc_struct.type_id,
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
        let (entity_type, name, grade) = get_npc_entity_type_name_grade(&pkt.npc_struct, max_hp);
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
        for sed in pkt.status_effect_datas {
            let entity = self.get_source_entity(sed.source_id);
            let encounter_entity = entities.get(&entity.name);
            // println!("entity: {:?}", entity);
            let status_effect = build_status_effect(
                sed,
                pkt.character_id,
                entity.id,
                StatusEffectTargetType::Party,
                timestamp,
                encounter_entity,
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
        self.status_tracker.borrow_mut().remove_status_effects(
            pkt.character_id,
            pkt.status_effect_instance_ids,
            pkt.reason,
            StatusEffectTargetType::Party,
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
            if unknown_local && member.name == most_likely_local_name
                && let Some(local_player) = self.entities.get_mut(&self.local_entity_id) {
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
                    self.id_tracker
                        .borrow_mut()
                        .add_mapping(member.character_id, self.local_entity_id);
                    self.party_tracker
                        .borrow_mut()
                        .set_name(member.name.clone());
                }

            let entity_id = self.id_tracker.borrow().get_entity_id(member.character_id);

            if let Some(entity_id) = entity_id {
                if let Some(entity) = self.entities.get_mut(&entity_id)
                    && entity.entity_type == Player && entity.name == member.name {
                        entity.gear_level = truncate_gear_level(member.gear_level);
                        entity.class_id = member.class_id as u32;
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
        let id = self.entities.get(&id).map_or(id, |entity| {
            if entity.entity_type == EntityType::Projectile || entity.entity_type == EntityType::Summon {
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
        let source_entity = self.get_source_entity(sed.source_id);
        let source_encounter_entity =
            entities.and_then(|entities| entities.get(&source_entity.name));
        let status_effect = build_status_effect(
            sed.clone(),
            target_id,
            source_entity.id,
            StatusEffectTargetType::Local,
            timestamp,
            source_encounter_entity,
        );

        self.status_tracker
            .borrow_mut()
            .register_status_effect(status_effect.clone());

        status_effect
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

fn get_npc_entity_type_name_grade(npc: &NpcStruct, max_hp: i64) -> (EntityType, String, String) {
    if let Some(esther) = get_esther_from_npc_id(npc.type_id) {
        return (EntityType::Esther, esther.name, "none".to_string());
    }

    if let Some((_, npc_info)) = NPC_DATA.get_key_value(&npc.type_id) {
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
            (Boss, npc_name.clone(), npc_info.grade.clone())
        } else {
            (EntityType::Npc, npc_name.clone(), npc_info.grade.clone())
        }
    } else {
        (EntityType::Npc, format!("{:x}", npc.object_id), "none".to_string())
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
    pub stats: HashMap<u8, i64>,
    pub stance: u8,
    pub grade: String,
    pub push_immune: bool,
    pub level: u16,
}
