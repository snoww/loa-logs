use bitflags::bitflags;
use hashbrown::{HashMap, HashSet};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, PartialEq, Clone)]
pub enum EntityType {
    #[default]
    UNKNOWN,
    MONSTER,
    BOSS,
    GUARDIAN,
    PLAYER,
    NPC,
}

impl ToString for EntityType {
    fn to_string(&self) -> String {
        match self {
            EntityType::UNKNOWN => "UNKNOWN".to_string(),
            EntityType::MONSTER => "MONSTER".to_string(),
            EntityType::BOSS => "BOSS".to_string(),
            EntityType::GUARDIAN => "GUARDIAN".to_string(),
            EntityType::PLAYER => "PLAYER".to_string(),
            EntityType::NPC => "NPC".to_string(),
        }
    }
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub last_combat_packet: i64,
    pub fight_start: i64,
    pub local_player: String,
    pub entities: HashMap<String, Entity>,
    pub current_boss_name: String,
    pub current_boss: Option<Entity>,
    pub encounter_damage_stats: EncounterDamageStats,
    pub duration: i64,
    pub reset: bool,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct EncounterDamageStats {
    pub total_damage_dealt: i64,
    pub top_damage_dealt: i64,
    pub total_damage_taken: i64,
    pub top_damage_taken: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
    pub most_damage_taken_entity: MostDamageTakenEntity,
    pub buffs: HashMap<i32, StatusEffect>,
    pub debuffs: HashMap<i32, StatusEffect>,
    #[serde(skip)]
    pub unknown_buffs: HashSet<i32>,
}

#[derive(Debug, Serialize, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct MostDamageTakenEntity {
    pub name: String,
    pub damage_taken: i64,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Entity {
    pub last_update: i64,
    pub id: String,
    pub npc_id: i32,
    pub name: String,
    pub entity_type: EntityType,
    pub class_id: i32,
    pub class: String,
    pub gear_score: f64,
    pub current_hp: i64,
    pub max_hp: i64,
    pub is_dead: bool,
    pub skills: HashMap<i32, Skill>,
    pub damage_stats: DamageStats,
    pub skill_stats: SkillStats,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: i32,
    pub name: String,
    pub icon: String,
    pub total_damage: i64,
    pub max_damage: i64,
    pub buffed_by: HashMap<i32, i64>,
    pub debuffed_by: HashMap<i32, i64>,
    pub buffed_by_support: i64,
    pub debuffed_by_support: i64,
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct DamageStats {
    pub damage_dealt: i64,
    pub damage_taken: i64,
    pub buffed_by: HashMap<i32, i64>,
    pub debuffed_by: HashMap<i32, i64>,
    pub buffed_by_support: i64,
    pub debuffed_by_support: i64,
    pub deaths: i64,
    pub death_time: i64,
    pub dps: i64,
    pub dps_intervals: HashMap<i32, i64>,
}

#[derive(Debug, Default, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillStats {
    pub casts: i64,
    pub hits: i64,
    pub crits: i64,
    pub back_attacks: i64,
    pub front_attacks: i64,
    pub counters: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Npc {
    pub id: i32,
    pub name: String,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    #[serde(rename(deserialize = "classid"))]
    pub class_id: i32,
    pub icon: String,
    #[serde(rename(deserialize = "summonids"))]
    pub summon_ids: Option<Vec<i32>>,
    #[serde(rename(deserialize = "summonsourceskill"))]
    pub summon_source_skill: Option<i32>,
    #[serde(rename(deserialize = "sourceskill"))]
    pub source_skill: Option<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    #[serde(rename(deserialize = "sourceskill"))]
    pub source_skill: Option<i32>,
    #[serde(rename(deserialize = "itemname"))]
    pub item_name: Option<String>,
    #[serde(skip, rename(deserialize = "itemdesc"))]
    pub item_desc: Option<String>,
    pub icon: Option<String>,
    #[serde(rename(deserialize = "itemcategory"))]
    pub item_category: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillBuffData {
    pub id: i32,
    pub name: String,
    pub desc: String,
    pub icon: String,
    #[serde(rename(deserialize = "iconshowtype"))]
    pub icon_show_type: String,
    pub duration: i32,
    // buff | debuff
    pub category: String,
    #[serde(rename(deserialize = "type"))]
    pub buff_type: String,
    #[serde(rename(deserialize = "buffcategory"))]
    pub buff_category: String,
    pub target: String,
    #[serde(rename(deserialize = "uniquegroup"))]
    pub unique_group: i32,
    #[serde(rename(deserialize = "overlapflag"))]
    pub overlap_flag: i32,
    #[serde(skip_serializing, rename(deserialize = "passiveoption"))]
    pub passive_option: Vec<PassiveOption>,
    #[serde(rename(deserialize = "sourceskill"))]
    pub source_skill: Option<i32>,
    #[serde(rename(deserialize = "setname"))]
    pub set_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct PassiveOption {
    #[serde(rename(deserialize = "type"))]
    pub option_type: String,
    #[serde(rename(deserialize = "keystat"))]
    pub key_stat: String,
    #[serde(rename(deserialize = "keyindex"))]
    pub key_index: i32,
    pub value: i32,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffect {
    pub target: StatusEffectTarget,
    pub category: String,
    pub buff_category: String,
    pub buff_type: u32,
    pub unique_group: i32,
    pub source: StatusEffectSource,
}

#[derive(Debug, Clone, Serialize, Default, PartialEq)]
pub enum StatusEffectTarget {
    #[default]
    OTHER,
    PARTY,
    SELF,
}

#[derive(Debug, Clone, Serialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct StatusEffectSource {
    pub name: String,
    pub desc: String,
    pub icon: String,
    pub skill: Option<SkillData>,
    pub set_name: Option<String>,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    pub struct StatusEffectBuffTypeFlags: u32 {
        const NONE = 0;
        const DMG = 1;
        const CRIT = 1 << 1;
        const ATKSPEED = 1 << 2;
        const MOVESPEED = 1 << 3;
        const HP = 1 << 4;
        const DEFENSE = 1 << 5;
        const RESOURCE = 1 << 6;
        const COOLDOWN = 1 << 7;
        const STAGGER = 1 << 8;
        const SHIELD = 1 << 9;

        const ANY = 1 << 20;
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectData {
    pub id: i32,
    pub conditions: Vec<CombatEffectCondition>,
    pub actions: Vec<CombatEffectAction>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectCondition {
    #[serde(rename(deserialize = "type"))]
    pub condition_type: String,
    pub actor: String,
    pub arg: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectAction {
    #[serde(rename(deserialize = "type"))]
    pub action_type: String,
    pub actor: String,
}

lazy_static! {
    pub static ref NPC_DATA: HashMap<i32, Npc> = {
        let json_str = include_str!("../../meter-data/Npc.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_DATA: HashMap<i32, SkillData> = {
        let json_str = include_str!("../../meter-data/Skill.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_EFFECT_DATA: HashMap<i32, SkillEffectData> = {
        let json_str = include_str!("../../meter-data/SkillEffect.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref SKILL_BUFF_DATA: HashMap<i32, SkillBuffData> = {
        let json_str = include_str!("../../meter-data/SkillBuff.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref COMBAT_EFFECT_DATA: HashMap<i32, CombatEffectData> = {
        let json_str = include_str!("../../meter-data/CombatEffect.json");
        serde_json::from_str(json_str).unwrap()
    };
    pub static ref STAT_TYPE_MAP: HashMap<&'static str, u32> = {
        let mut map = HashMap::new();
        map.insert("none", 0);
        map.insert("hp", 1);
        map.insert("mp", 2);
        map.insert("str", 3);
        map.insert("agi", 4);
        map.insert("int", 5);
        map.insert("con", 6);
        map.insert("str_x", 7);
        map.insert("agi_x", 8);
        map.insert("int_x", 9);
        map.insert("con_x", 10);
        map.insert("criticalhit", 15);
        map.insert("specialty", 16);
        map.insert("oppression", 17);
        map.insert("rapidity", 18);
        map.insert("endurance", 19);
        map.insert("mastery", 20);
        map.insert("criticalhit_x", 21);
        map.insert("specialty_x", 22);
        map.insert("oppression_x", 23);
        map.insert("rapidity_x", 24);
        map.insert("endurance_x", 25);
        map.insert("mastery_x", 26);
        map.insert("max_hp", 27);
        map.insert("max_mp", 28);
        map.insert("max_hp_x", 29);
        map.insert("max_mp_x", 30);
        map.insert("max_hp_x_x", 31);
        map.insert("max_mp_x_x", 32);
        map.insert("normal_hp_recovery", 33);
        map.insert("combat_hp_recovery", 34);
        map.insert("normal_hp_recovery_rate", 35);
        map.insert("combat_hp_recovery_rate", 36);
        map.insert("normal_mp_recovery", 37);
        map.insert("combat_mp_recovery", 38);
        map.insert("normal_mp_recovery_rate", 39);
        map.insert("combat_mp_recovery_rate", 40);
        map.insert("self_recovery_rate", 41);
        map.insert("drain_hp_dam_rate", 42);
        map.insert("drain_mp_dam_rate", 43);
        map.insert("dam_reflection_rate", 44);
        map.insert("char_attack_dam", 47);
        map.insert("skill_effect_dam_addend", 48);
        map.insert("attack_power_rate", 49);
        map.insert("skill_damage_rate", 50);
        map.insert("attack_power_rate_x", 51);
        map.insert("skill_damage_rate_x", 52);
        map.insert("cooldown_reduction", 53);
        map.insert("paralyzation_point_rate", 54);
        map.insert("def", 55);
        map.insert("res", 56);
        map.insert("def_x", 57);
        map.insert("res_x", 58);
        map.insert("def_x_x", 59);
        map.insert("res_x_x", 60);
        map.insert("def_pen_rate", 67);
        map.insert("res_pen_rate", 68);
        map.insert("physical_inc_rate", 69);
        map.insert("magical_inc_rate", 70);
        map.insert("self_shield_rate", 71);
        map.insert("hit_rate", 72);
        map.insert("dodge_rate", 73);
        map.insert("critical_hit_rate", 74);
        map.insert("critical_res_rate", 75);
        map.insert("critical_dam_rate", 76);
        map.insert("attack_speed", 77);
        map.insert("attack_speed_rate", 78);
        map.insert("move_speed", 79);
        map.insert("move_speed_rate", 80);
        map.insert("prop_move_speed", 81);
        map.insert("prop_move_speed_rate", 82);
        map.insert("vehicle_move_speed", 83);
        map.insert("vehicle_move_speed_rate", 84);
        map.insert("ship_move_speed", 85);
        map.insert("ship_move_speed_rate", 86);
        map.insert("fire_dam_rate", 87);
        map.insert("ice_dam_rate", 88);
        map.insert("electricity_dam_rate", 89);
        map.insert("earth_dam_rate", 91);
        map.insert("dark_dam_rate", 92);
        map.insert("holy_dam_rate", 93);
        map.insert("elements_dam_rate", 94);
        map.insert("fire_res_rate", 95);
        map.insert("ice_res_rate", 96);
        map.insert("electricity_res_rate", 97);
        map.insert("earth_res_rate", 99);
        map.insert("dark_res_rate", 100);
        map.insert("holy_res_rate", 101);
        map.insert("elements_res_rate", 102);
        map.insert("self_cc_time_rate", 105);
        map.insert("enemy_cc_time_rate", 106);
        map.insert("identity_value1", 107);
        map.insert("identity_value2", 108);
        map.insert("identity_value3", 109);
        map.insert("awakening_dam_rate", 110);
        map.insert("item_drop_rate", 111);
        map.insert("gold_rate", 112);
        map.insert("exp_rate", 113);
        map.insert("attack_power_addend", 123);
        map.insert("npc_species_humanoid_dam_rate", 125);
        map.insert("npc_species_devil_dam_rate", 126);
        map.insert("npc_species_substance_dam_rate", 127);
        map.insert("npc_species_undead_dam_rate", 128);
        map.insert("npc_species_plant_dam_rate", 129);
        map.insert("npc_species_insect_dam_rate", 130);
        map.insert("npc_species_spirit_dam_rate", 131);
        map.insert("npc_species_wild_beast_dam_rate", 132);
        map.insert("npc_species_mechanic_dam_rate", 133);
        map.insert("npc_species_ancient_dam_rate", 134);
        map.insert("npc_species_god_dam_rate", 135);
        map.insert("npc_species_archfiend_dam_rate", 136);
        map.insert("vitality", 137);
        map.insert("ship_booter_speed", 138);
        map.insert("ship_wreck_speed_rate", 139);
        map.insert("island_speed_rate", 140);
        map.insert("attack_power_sub_rate_1", 141);
        map.insert("attack_power_sub_rate_2", 142);
        map.insert("physical_inc_sub_rate_1", 143);
        map.insert("physical_inc_sub_rate_2", 144);
        map.insert("magical_inc_sub_rate_1", 145);
        map.insert("magical_inc_sub_rate_2", 146);
        map.insert("skill_damage_sub_rate_1", 147);
        map.insert("skill_damage_sub_rate_2", 148);
        map.insert("resource_recovery_rate", 149);
        map.insert("weapon_dam", 151);
        map
    };
}
