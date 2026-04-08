use std::time::{Duration, Instant};

use hashbrown::HashMap;
use tauri::{AppHandle, Emitter, EventTarget};

use crate::models::*;

pub struct UiSender {
    app_handle: AppHandle,
    updated_on: Instant,
    throttle_interval: Duration
}

impl UiSender {
    pub fn new(app_handle: AppHandle) -> Self {
        Self {
            app_handle,
            updated_on: Instant::now(),
            throttle_interval: Duration::from_millis(200)
        }
    }

    pub fn can_send(&mut self) -> bool {
        self.updated_on.elapsed() >= self.throttle_interval
    }

    pub fn set_interval(&mut self, value: Duration) {
        self.throttle_interval = value;
    }

    pub fn try_send(&mut self, payload: UiPayload) {
        // It's safe to use unwrap_unchecked
        // - we do not send events with name descriptor containing : - , / or _
        // - json is generated statically from rust structs
        unsafe {
            match payload {
                UiPayload::None => {},
                UiPayload::InvalidDamage => {
                    self.app_handle.emit_str_filter("invalid-damage", String::default(), Self::main_or_mini).unwrap_unchecked();
                },
                UiPayload::Data { snapshot, party_info } => {
                    let json = serde_json::to_string(&snapshot).unwrap_unchecked();
                    self.app_handle.emit_str_filter("encounter-update", json, Self::main_or_mini).unwrap_unchecked();

                    if let Some(party_info) = party_info {
                        let json = serde_json::to_string(&party_info).unwrap_unchecked();
                        self.app_handle.emit_str_filter("party-update", json, Self::main_or_mini).unwrap_unchecked();
                    }
                },
            }

            self.updated_on = Instant::now();
        }
    }

    fn main_or_mini(target: &EventTarget) -> bool {
        match target {
            EventTarget::Window { label } 
                | EventTarget::Webview { label } 
                | EventTarget::WebviewWindow { label }=> matches!(label.as_str(), "main" | "mini"),
            _ => false,
        }
    }
}

pub fn prepare_ui_payload<'a>(
    encounter: &'a Encounter,
    party_info: Option<&'a [Vec<String>]>,
    damage_valid: bool,
    boss_dead: bool
) -> UiPayload<'a> {
        
    if !damage_valid {
        return UiPayload::InvalidDamage;
    }

    let entities: HashMap<_, _> = encounter.entities.iter()
        .filter(is_entity_valid_for_ui)
        .map(|(k, e)| (k, e.into()))
        .collect();

    if entities.is_empty() {
        return UiPayload::None;
    }

    let mut snapshot = EncounterSnapshot {
        last_combat_packet: encounter.last_combat_packet,
        fight_start: encounter.fight_start,
        local_player: &encounter.local_player,
        entities,
        current_boss_name: "",
        current_boss: None,
        encounter_damage_stats: (&encounter.encounter_damage_stats).into(),
        duration: encounter.duration,
        difficulty: encounter.difficulty.as_deref(),
        boss_only_damage: encounter.boss_only_damage,
        region: encounter.region.as_deref(),
    };

    snapshot.current_boss = build_current_boss(encounter, boss_dead);

    if snapshot.current_boss.is_some() {
        snapshot.current_boss_name = &encounter.current_boss_name;
    }

    UiPayload::Data { snapshot, party_info }
}

fn is_entity_valid_for_ui(item: &(&String, &EncounterEntity)) -> bool {
    let e = item.1;
    ((e.entity_type == EntityType::Player && e.class_id > 0)
        || e.entity_type == EntityType::Esther
        || e.entity_type == EntityType::Boss)
        && e.damage_stats.damage_dealt > 0
}

fn build_current_boss<'a>(
    encounter: &'a Encounter,
    boss_dead: bool,
) -> Option<EncounterCurrentBoss<'a>> {

    let name = &encounter.current_boss_name;
    let boss = encounter.entities.get(name)?;

    if name.is_empty() {
        return None;
    }

    let mut current_hp = boss.current_hp;
    let mut is_dead = boss.is_dead;

    if boss_dead {
        current_hp = 0;
        is_dead = true;
    }

    Some(EncounterCurrentBoss {
        id: boss.id,
        npc_id: boss.npc_id,
        hp_bars: boss.hp_bars,
        name,
        entity_type: boss.entity_type,
        current_hp,
        max_hp: boss.max_hp,
        current_shield: boss.current_shield,
        is_dead,
        skills: &boss.skills,
        damage_stats: &boss.damage_stats,
        skill_stats: &boss.skill_stats,
    })
}