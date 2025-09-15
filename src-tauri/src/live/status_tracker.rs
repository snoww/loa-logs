use crate::data::*;
use crate::live::entity_tracker::Entity;
use crate::live::party_tracker::PartyTracker;
use crate::live::status_tracker::StatusEffectBuffCategory::{BattleItem, Bracelet, Elixir, Etc};
use crate::live::status_tracker::StatusEffectCategory::Debuff;
use crate::live::status_tracker::StatusEffectShowType::All;
use crate::live::utils::get_new_id;
use crate::models::{EncounterEntity, EntityType};
use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use meter_core::packets::structures::{PCStruct, StatusEffectData};
use std::cell::RefCell;
use std::rc::Rc;

// expire buff after 1 min delay
const TIMEOUT_DELAY_MS: i64 = 60_000;
const WORKSHOP_BUFF_ID: u32 = 9701;

pub type StatusEffectRegistry = HashMap<u32, StatusEffectDetails>;

pub struct StatusTracker {
    party_tracker: Rc<RefCell<PartyTracker>>,
    local_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
    party_status_effect_registry: HashMap<u64, StatusEffectRegistry>,
}

impl StatusTracker {
    pub fn new(party_tracker: Rc<RefCell<PartyTracker>>) -> Self {
        Self {
            party_tracker,
            local_status_effect_registry: HashMap::new(),
            party_status_effect_registry: HashMap::new(),
        }
    }

    pub fn new_pc(&mut self, pc_struct: PCStruct, local_character_id: u64) {
        let use_party_status_effects =
            self.should_use_party_status_effect(pc_struct.character_id, local_character_id);
        if use_party_status_effects {
            self.remove_party_object(pc_struct.character_id);
        } else {
            self.remove_local_object(pc_struct.character_id);
        }
        let (target_id, target_type) = if use_party_status_effects {
            (pc_struct.character_id, StatusEffectTargetType::Party)
        } else {
            (pc_struct.player_id, StatusEffectTargetType::Local)
        };
        let timestamp = Utc::now();
        for sed in pc_struct.status_effect_datas.into_iter() {
            let source_id = sed.source_id;
            let status_effect =
                build_status_effect(sed, target_id, source_id, target_type, timestamp, None);
            self.register_status_effect(status_effect);
        }
    }

    pub fn register_status_effect(&mut self, se: StatusEffectDetails) {
        let registry = match se.target_type {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        registry.entry(se.target_id).or_insert_with(HashMap::new);

        let ser = registry.get_mut(&se.target_id).unwrap();
        ser.insert(se.instance_id, se);
    }

    pub fn remove_local_object(&mut self, object_id: u64) {
        self.local_status_effect_registry.remove(&object_id);
    }

    pub fn remove_party_object(&mut self, object_id: u64) {
        self.party_status_effect_registry.remove(&object_id);
    }

    pub fn remove_status_effects(
        &mut self,
        target_id: u64,
        instance_id: Vec<u32>,
        reason: u8,
        sett: StatusEffectTargetType,
    ) -> (
        bool,
        Vec<StatusEffectDetails>,
        Vec<StatusEffectDetails>,
        bool,
    ) {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let mut has_shield_buff = false;
        let mut shields_broken: Vec<StatusEffectDetails> = Vec::new();
        let mut left_workshop = false;
        let mut effects_removed = Vec::new();

        if let Some(ser) = registry.get_mut(&target_id) {
            for id in instance_id {
                if let Some(se) = ser.remove(&id) {
                    if se.status_effect_id == WORKSHOP_BUFF_ID {
                        left_workshop = true;
                    }
                    if se.status_effect_type == StatusEffectType::Shield {
                        has_shield_buff = true;
                        if reason == 4 {
                            shields_broken.push(se);
                            continue;
                        }
                    }
                    effects_removed.push(se);
                }
            }
        }

        (
            has_shield_buff,
            shields_broken,
            effects_removed,
            left_workshop,
        )
    }

    pub fn update_status_duration(
        &mut self,
        instance_id: u32,
        target_id: u64,
        timestamp: u64,
        sett: StatusEffectTargetType,
    ) {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return,
        };

        if let Some(se) = ser.get_mut(&instance_id)
            && let Some(duration_ms) = timestamp.checked_sub(se.end_tick)
                && duration_ms > 0 && duration_ms < 10_000_000 {
                    se.end_tick = timestamp;
                    if let Some(expire_at) = se.expire_at {
                        se.expire_at = Some(
                            expire_at
                                + Duration::milliseconds(duration_ms as i64 + TIMEOUT_DELAY_MS),
                        );
                    }
                }
    }

    pub fn sync_status_effect(
        &mut self,
        instance_id: u32,
        character_id: u64,
        object_id: u64,
        value: u64,
        local_character_id: u64,
    ) -> (Option<StatusEffectDetails>, u64) {
        let use_party = self.should_use_party_status_effect(character_id, local_character_id);
        let (target_id, sett) = if use_party {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (object_id, StatusEffectTargetType::Local)
        };
        if target_id == 0 {
            return (None, 0);
        }
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return (None, 0),
        };

        let se = match ser.get_mut(&instance_id) {
            Some(se) => se,
            None => return (None, 0),
        };

        let old_value = se.value;
        se.value = value;

        (Some(se.clone()), old_value)
    }

    pub fn get_status_effects(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        local_character_id: u64,
    ) -> (Vec<StatusEffectDetails>, Vec<StatusEffectDetails>) {
        let timestamp = Utc::now();

        let use_party_for_source = if source_entity.entity_type == EntityType::Player {
            self.should_use_party_status_effect(source_entity.character_id, local_character_id)
        } else {
            false
        };
        // println!("use_party_for_source: {:?}", use_party_for_source);
        let (source_id, source_type) = if use_party_for_source {
            (source_entity.character_id, StatusEffectTargetType::Party)
        } else {
            (source_entity.id, StatusEffectTargetType::Local)
        };
        // println!("source_id: {:?}, source_type: {:?}", source_id, source_type);

        let status_effects_on_source =
            self.actually_get_status_effects(source_id, source_type, timestamp);

        let use_party_for_target = if source_entity.entity_type == EntityType::Player {
            self.should_use_party_status_effect(target_entity.character_id, local_character_id)
        } else {
            false
        };
        // println!("use_party_for_target: {:?}", use_party_for_target);
        let source_party_id = self
            .party_tracker
            .borrow()
            .entity_id_to_party_id
            .get(&source_entity.id)
            .cloned();
        // println!("use_party_for_target: {:?}, source_party_id: {:?}", use_party_for_target, source_party_id);
        let mut status_effects_on_target = match (use_party_for_target, source_party_id) {
            (true, Some(source_party_id)) => self.get_status_effects_from_party(
                target_entity.character_id,
                StatusEffectTargetType::Party,
                &source_party_id,
                timestamp,
            ),
            (false, Some(source_party_id)) => self.get_status_effects_from_party(
                target_entity.id,
                StatusEffectTargetType::Local,
                &source_party_id,
                timestamp,
            ),
            (true, None) => self.actually_get_status_effects(
                target_entity.character_id,
                StatusEffectTargetType::Party,
                timestamp,
            ),
            (false, None) => self.actually_get_status_effects(
                target_entity.id,
                StatusEffectTargetType::Local,
                timestamp,
            ),
        };
        // println!("status_effects_on_target: {:?}", status_effects_on_target);
        // println!(
        //     "status_effects_on_source: {:?}, status_effects_on_target: {:?}",
        //     status_effects_on_source, status_effects_on_target);
        status_effects_on_target.retain(|se| {
            !(se.target_type == StatusEffectTargetType::Local
                && se.category == Debuff
                && se.source_id != source_id
                && se.db_target_type == "self")
        });
        (status_effects_on_source, status_effects_on_target)
    }

    pub fn actually_get_status_effects(
        &mut self,
        target_id: u64,
        sett: StatusEffectTargetType,
        timestamp: DateTime<Utc>,
    ) -> Vec<StatusEffectDetails> {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        // println!("registry: {:?}", registry);
        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return Vec::new(),
        };
        ser.retain(|_, se| se.expire_at.is_none_or(|expire_at| expire_at > timestamp));
        ser.values().cloned().collect()
    }

    pub fn get_status_effects_from_party(
        &mut self,
        target_id: u64,
        sett: StatusEffectTargetType,
        party_id: &u32,
        timestamp: DateTime<Utc>,
    ) -> Vec<StatusEffectDetails> {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };
        // println!("registry: {:?}", registry);
        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return Vec::new(),
        };

        // println!("ser before: {:?}", ser);
        ser.retain(|_, se| se.expire_at.is_none_or(|expire_at| expire_at > timestamp));
        let party_tracker = self.party_tracker.borrow();
        ser.values()
            .filter(|x| {
                is_valid_for_raid(x)
                    || *party_id
                        == party_tracker
                            .entity_id_to_party_id
                            .get(&x.source_id)
                            .cloned()
                            .unwrap_or(0)
            })
            .cloned()
            .collect()
    }

    fn should_use_party_status_effect(&self, character_id: u64, local_character_id: u64) -> bool {
        let party_tracker = self.party_tracker.borrow();
        let local_player_party_id = party_tracker
            .character_id_to_party_id
            .get(&local_character_id);
        let affected_player_party_id = party_tracker.character_id_to_party_id.get(&character_id);
        // println!("party character_id_to_party_id: {:?}", party_tracker.character_id_to_party_id);
        // println!("character_id: {}, local_character_id: {}", character_id, local_character_id);
        // println!(
        //     "local_player_party_id: {:?}, affected_player_party_id: {:?}",
        //     local_player_party_id, affected_player_party_id);

        match (
            local_player_party_id,
            affected_player_party_id,
            character_id == local_character_id,
        ) {
            (Some(local_party), Some(affected_party), false) => local_party == affected_party,
            _ => false,
        }
    }

    pub fn clear(&mut self) {
        self.local_status_effect_registry.clear();
        self.party_status_effect_registry.clear();
    }
}

fn is_valid_for_raid(status_effect: &StatusEffectDetails) -> bool {
    (status_effect.buff_category == BattleItem
        || status_effect.buff_category == Bracelet
        || status_effect.buff_category == Elixir
        || status_effect.buff_category == Etc)
        && status_effect.category == Debuff
        && status_effect.show_type == All
}

pub fn build_status_effect(
    se_data: StatusEffectData,
    target_id: u64,
    source_id: u64,
    target_type: StatusEffectTargetType,
    timestamp: DateTime<Utc>,
    source_entity: Option<&EncounterEntity>,
) -> StatusEffectDetails {
    let value = get_status_effect_value(&se_data.value.bytearray_0);
    let mut status_effect_category = StatusEffectCategory::Other;
    let mut buff_category = StatusEffectBuffCategory::Other;
    let mut show_type = StatusEffectShowType::Other;
    let mut status_effect_type = StatusEffectType::Other;
    let mut name = "Unknown".to_string();
    let mut db_target_type = "".to_string();
    let mut custom_id = 0;
    if let Some(effect) = SKILL_BUFF_DATA.get(&se_data.status_effect_id) {
        name = effect.name.clone().unwrap_or_default();
        if effect.category.as_str() == "debuff" {
            status_effect_category = Debuff
        }
        match effect.buff_category.clone().unwrap_or_default().as_str() {
            "bracelet" => buff_category = Bracelet,
            "etc" => buff_category = Etc,
            "battleitem" => buff_category = BattleItem,
            "elixir" => buff_category = Elixir,
            _ => {}
        }
        if effect.icon_show_type.clone().unwrap_or_default() == "all" {
            show_type = All
        }
        status_effect_type = match effect.buff_type.as_str() {
            "shield" => StatusEffectType::Shield,
            "freeze" | "fear" | "stun" | "sleep" | "earthquake" | "electrocution"
            | "polymorph_pc" | "forced_move" | "mind_control" | "paralyzation" => {
                StatusEffectType::HardCrowdControl
            }
            _ => StatusEffectType::Other,
        };
        db_target_type = effect.target.to_string();

        if let Some(source_skills) = effect.source_skills.as_ref() {
            // if skill has multiple source skills, we need to find the one that was last used
            // e.g. bard brands have same buff id, but have different source skills (sound shock, harp)
            // if skills only have one source skill, we dont care about it here and it gets handled later
            if source_skills.len() > 1
                && let Some(source_entity) = source_entity {
                    let mut last_time = i64::MIN;
                    let mut last_skill = 0_u32;
                    for source_skill in source_skills {
                        if let Some(skill) = source_entity.skills.get(source_skill) {
                            if skill.name.is_empty() {
                                continue;
                            }
                            // hard code check for stigma brand tripod
                            // maybe set up a map of tripods for other skills in future idk??
                            if skill.id == 21090 {
                                if let Some(tripods) = skill.tripod_index {
                                    if tripods.second != 2 {
                                        continue;
                                    }
                                } else {
                                    continue;
                                }
                            }
                            if skill.last_timestamp > last_time {
                                last_skill = *source_skill;
                                last_time = skill.last_timestamp;
                            }
                        }
                    }

                    // if such a skill exists, we assign a new custom buff id to distinguish it.
                    // we encode the buff id as well too because there are multiple buffs that have
                    // the same source skill, that also have multiple source skills.
                    // without it, it leads to customids that are different but end up sharing the same id
                    if last_skill > 0 {
                        custom_id = get_new_id(last_skill + (effect.id as u32));
                    }
                }
        }
    }

    let expiry = if se_data.total_time > 0. && se_data.total_time < 604800. {
        Some(
            timestamp
                + Duration::milliseconds((se_data.total_time as i64) * 1000 + TIMEOUT_DELAY_MS),
        )
    } else {
        None
    };

    StatusEffectDetails {
        instance_id: se_data.status_effect_instance_id,
        source_id,
        target_id,
        status_effect_id: se_data.status_effect_id,
        custom_id,
        target_type,
        db_target_type,
        value,
        stack_count: se_data.stack_count,
        buff_category,
        category: status_effect_category,
        status_effect_type,
        show_type,
        expiration_delay: se_data.total_time,
        expire_at: expiry,
        end_tick: se_data.end_tick,
        name,
        timestamp,
    }
}

pub fn get_status_effect_value(value: &Option<Vec<u8>>) -> u64 {
    value.as_ref().map_or(0, |v| {
        let c1 = v
            .get(0..8)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        let c2 = v
            .get(8..16)
            .map_or(0, |bytes| u64::from_le_bytes(bytes.try_into().unwrap()));
        c1.min(c2)
    })
}
#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectTargetType {
    #[default]
    Party = 0,
    Local = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectCategory {
    #[default]
    Other = 0,
    Debuff = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectBuffCategory {
    #[default]
    Other = 0,
    Bracelet = 1,
    Etc = 2,
    BattleItem = 3,
    Elixir = 4,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectShowType {
    #[default]
    Other = 0,
    All = 1,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum StatusEffectType {
    #[default]
    Shield = 0,
    Other = 1,
    HardCrowdControl = 2, // stun, root, MC, etc
}

#[derive(Debug, Default, Clone)]
pub struct StatusEffectDetails {
    pub instance_id: u32,
    pub status_effect_id: u32,
    pub custom_id: u32,
    pub target_id: u64,
    pub source_id: u64,
    pub target_type: StatusEffectTargetType,
    pub db_target_type: String,
    pub value: u64,
    pub stack_count: u8,
    pub category: StatusEffectCategory,
    pub buff_category: StatusEffectBuffCategory,
    pub show_type: StatusEffectShowType,
    pub status_effect_type: StatusEffectType,
    pub expiration_delay: f32,
    pub expire_at: Option<DateTime<Utc>>,
    pub end_tick: u64,
    pub timestamp: DateTime<Utc>,
    pub name: String,
}
