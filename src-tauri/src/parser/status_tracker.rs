use crate::parser::entity_tracker::Entity;
use crate::parser::models::{EntityType, SKILL_BUFF_DATA};
use crate::parser::party_tracker::PartyTracker;
use crate::parser::status_tracker::StatusEffectBuffCategory::{BattleItem, Bracelet, Etc};
use crate::parser::status_tracker::StatusEffectCategory::Debuff;
use crate::parser::status_tracker::StatusEffectShowType::All;
use chrono::{DateTime, Duration, Utc};
use hashbrown::HashMap;
use meter_core::packets::definitions::PKTNewPC;
use meter_core::packets::structures::StatusEffectData;
use std::cell::RefCell;
use std::rc::Rc;

const TIMEOUT_DELAY_MS: i64 = 1000;

pub type StatusEffectRegistry = HashMap<u32, StatusEffect>;

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

    pub fn new_pc(&mut self, pkt: &PKTNewPC, local_character_id: u64) {
        let use_party_status_effects =
            self.should_use_party_status_effect(pkt.pc_struct.character_id, local_character_id);
        if use_party_status_effects {
            self.remove_party_object(pkt.pc_struct.character_id);
        } else {
            self.remove_local_object(pkt.pc_struct.character_id);
        }
        let (target_id, target_type) = if use_party_status_effects {
            (pkt.pc_struct.character_id, StatusEffectTargetType::Party)
        } else {
            (pkt.pc_struct.player_id, StatusEffectTargetType::Local)
        };
        for sed in pkt.pc_struct.status_effect_datas.iter() {
            let source_id = sed.source_id;
            let status_effect = build_status_effect(sed.clone(), target_id, source_id, target_type);
            self.register_status_effect(status_effect);
        }
    }

    pub fn register_status_effect(&mut self, mut se: StatusEffect) {
        let registry = match se.target_type {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        registry.entry(se.target_id).or_insert_with(HashMap::new);

        let ser = registry.get_mut(&se.target_id).unwrap();
        add_status_effect_timeout(&mut se);
        ser.insert(se.instance_id, se.clone());
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
        sett: StatusEffectTargetType,
    ) {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };
        if let Some(ser) = registry.get_mut(&target_id) {
            for id in instance_id {
                ser.remove(&id);
            }
        }
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

        if let Some(se) = ser.get_mut(&instance_id) {
            if let Some(expire_at) = se.expire_at {
                let extension_ms = (timestamp - se.end_tick) as i64;
                se.expire_at = Some(expire_at + Duration::milliseconds(extension_ms));
                se.end_tick = timestamp;
            }
        }
    }

    /*    pub fn sync_status_effect(&mut self, instance_id: u32, character_id: u64, object_id: u64, value: u32, local_character_id: u64) {
        let use_party = self.should_use_party_status_effect(character_id, local_character_id);
        let (target_id, sett) = if use_party {
            (character_id, StatusEffectTargetType::Party)
        } else {
            (object_id, StatusEffectTargetType::Local)
        };
        if target_id == 0 {
            return;
        }
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return,
        };

        let se = match ser.get_mut(&instance_id) {
            Some(se) => se,
            None => return,
        };

        se.value = value;
    }*/

    pub fn get_status_effects(
        &mut self,
        source_entity: &Entity,
        target_entity: &Entity,
        local_character_id: u64,
    ) -> (Vec<(u32, u64)>, Vec<(u32, u64)>) {
        let use_party_for_source = if source_entity.entity_type == EntityType::PLAYER {
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

        let source_effects = self.actually_get_status_effects(source_id, source_type);
        let status_effects_on_source: Vec<(u32, u64)> = source_effects
            .iter()
            .map(|x| (x.status_effect_id, x.source_id))
            .collect();
        // println!("status_effects_on_source: {:?}", status_effects_on_source);

        let use_party_for_target = if source_entity.entity_type == EntityType::PLAYER {
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
        let target_effects = match (use_party_for_target, source_party_id) {
            (true, Some(source_party_id)) => self.get_status_effects_from_party(
                target_entity.character_id,
                StatusEffectTargetType::Party,
                &source_party_id,
            ),
            (false, Some(source_party_id)) => self.get_status_effects_from_party(
                target_entity.id,
                StatusEffectTargetType::Local,
                &source_party_id,
            ),
            (true, None) => self.actually_get_status_effects(
                target_entity.character_id,
                StatusEffectTargetType::Party,
            ),
            (false, None) => {
                self.actually_get_status_effects(target_entity.id, StatusEffectTargetType::Local)
            }
        };
        let status_effects_on_target: Vec<(u32, u64)> = target_effects
            .iter()
            .map(|x| (x.status_effect_id, x.source_id))
            .collect();
        // println!("status_effects_on_target: {:?}", status_effects_on_target);
        // println!(
        //     "status_effects_on_source: {:?}, status_effects_on_target: {:?}",
        //     status_effects_on_source, status_effects_on_target);
        (status_effects_on_source, status_effects_on_target)
    }

    pub fn actually_get_status_effects(
        &mut self,
        target_id: u64,
        sett: StatusEffectTargetType,
    ) -> Vec<StatusEffect> {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };

        // println!("registry: {:?}", registry);
        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return Vec::new(),
        };
        let timestamp = Utc::now();
        ser.retain(|_, se| se.expire_at.map_or(true, |expire_at| expire_at > timestamp));
        ser.values().cloned().collect()
    }

    pub fn get_status_effects_from_party(
        &mut self,
        target_id: u64,
        sett: StatusEffectTargetType,
        party_id: &u32,
    ) -> Vec<StatusEffect> {
        let registry = match sett {
            StatusEffectTargetType::Local => &mut self.local_status_effect_registry,
            StatusEffectTargetType::Party => &mut self.party_status_effect_registry,
        };
        // println!("registry: {:?}", registry);
        let ser = match registry.get_mut(&target_id) {
            Some(ser) => ser,
            None => return Vec::new(),
        };

        let timestamp = Utc::now();
        // println!("ser before: {:?}", ser);
        ser.retain(|_, se| se.expire_at.map_or(true, |expire_at| expire_at > timestamp));
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

fn is_valid_for_raid(status_effect: &StatusEffect) -> bool {
    (status_effect.buff_category == BattleItem
        || status_effect.buff_category == Bracelet
        || status_effect.buff_category == Etc)
        && status_effect.category == Debuff
        && status_effect.show_type == All
}

pub fn add_status_effect_timeout(se: &mut StatusEffect) {
    if se.expiration_delay > 0. && se.expiration_delay < 604800. {
        let expiration_delay = (se.expiration_delay * 1000.) as i64;
        let timeout_delay = Duration::milliseconds(expiration_delay + TIMEOUT_DELAY_MS);

        se.expire_at = Some(se.timestamp + timeout_delay);
        // println!("===\nstatus_id: {:?}, start_date: {:?}, expiration_delay: {:?}, timeout_delay: {:?}, expire_at: {:?}", se.status_effect_id,
        //          start_date, expiration_delay, timeout_delay, se.expire_at);
    }
}

pub fn build_status_effect(
    se_data: StatusEffectData,
    target_id: u64,
    source_id: u64,
    target_type: StatusEffectTargetType,
) -> StatusEffect {
    let c1 = se_data
        .value
        .as_ref()
        .and_then(|v| {
            v.get(0..4)
                .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        })
        .unwrap_or(0);

    let c2 = se_data
        .value
        .as_ref()
        .and_then(|v| {
            v.get(4..8)
                .map(|bytes| u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
        })
        .unwrap_or(0);

    let val = c1.min(c2);
    let mut status_effect_category = StatusEffectCategory::Other;
    let mut buff_category = StatusEffectBuffCategory::Other;
    let mut show_type = StatusEffectShowType::Other;
    let mut status_effect_type = StatusEffectType::Other;
    let mut name = "Unknown".to_string();
    if let Some(effect) = SKILL_BUFF_DATA.get(&(se_data.status_effect_id as i32)) {
        name = effect.name.to_string();
        if effect.category.as_str() == "debuff" {
            status_effect_category = Debuff
        }
        match effect.buff_category.as_str() {
            "bracelet" => buff_category = Bracelet,
            "etc" => buff_category = Etc,
            "battleitem" => buff_category = BattleItem,
            _ => {}
        }
        if effect.icon_show_type.as_str() == "all" {
            show_type = All
        }
        if effect.buff_type.as_str() == "shield" {
            status_effect_type = StatusEffectType::Shield
        }
    }

    StatusEffect {
        instance_id: se_data.effect_instance_id,
        source_id,
        target_id,
        status_effect_id: se_data.status_effect_id,
        target_type,
        value: val,
        buff_category,
        category: status_effect_category,
        status_effect_type,
        show_type,
        expiration_delay: se_data.total_time,
        expire_at: None,
        end_tick: se_data.end_tick,
        name,
        timestamp: Utc::now(),
    }
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
}

#[derive(Debug, Default, Clone)]
pub struct StatusEffect {
    instance_id: u32,
    status_effect_id: u32,
    target_id: u64,
    source_id: u64,
    target_type: StatusEffectTargetType,
    value: u32,
    category: StatusEffectCategory,
    buff_category: StatusEffectBuffCategory,
    show_type: StatusEffectShowType,
    status_effect_type: StatusEffectType,
    expiration_delay: f32,
    expire_at: Option<DateTime<Utc>>,
    end_tick: u64,
    timestamp: DateTime<Utc>,
    name: String,
}
