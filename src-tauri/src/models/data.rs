use crate::models::utils::{
    int_or_string_as_option_string, int_or_string_as_string, null_or_default,
};
use hashbrown::HashMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillData {
    pub id: i32,
    pub name: Option<String>,
    #[serde(rename = "type", default)]
    #[serde(deserialize_with = "int_or_string_as_string")]
    pub skill_type: String,
    pub desc: Option<String>,
    pub class_id: u32,
    pub icon: Option<String>,
    #[serde(default, deserialize_with = "int_or_string_as_option_string")]
    pub identity_category: Option<String>,
    #[serde(default)]
    pub directional_mask: i32,
    #[serde(alias = "groups")]
    pub groups: Option<Vec<u32>>,
    pub summon_source_skills: Option<Vec<u32>>,
    pub source_skills: Option<Vec<u32>>,
    #[serde(default)]
    pub is_hyper_awakening: bool,
    #[serde(default)]
    pub levels: HashMap<u32, SkillLevelData>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillLevelData {
    #[serde(default)]
    pub mana_cost: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillEffectData {
    pub id: i32,
    pub comment: String,
    #[serde(skip)]
    pub stagger: i32,
    pub source_skills: Option<Vec<u32>>,
    pub directional_mask: Option<i32>,
    pub item_name: Option<String>,
    pub item_desc: Option<String>,
    pub item_type: Option<String>,
    pub icon: Option<String>,
    pub values: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SkillBuffData {
    pub id: i32,
    pub name: Option<String>,
    pub desc: Option<String>,
    pub icon: Option<String>,
    pub icon_show_type: Option<String>,
    pub duration: i32,
    // buff | debuff
    pub category: String,
    #[serde(rename(deserialize = "type"))]
    #[serde(deserialize_with = "int_or_string_as_string")]
    pub buff_type: String,
    pub status_effect_values: Option<Vec<i32>>,
    #[serde(deserialize_with = "int_or_string_as_option_string")]
    pub buff_category: Option<String>,
    pub target: String,
    pub unique_group: u32,
    #[serde(rename(deserialize = "overlap"))]
    pub overlap_flag: i32,
    pub per_level_data: HashMap<String, PerLevelData>,
    pub source_skills: Option<Vec<u32>>,
    pub set_name: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PerLevelData {
    pub passive_options: Vec<PassiveOption>,
    pub status_effect_values: Option<Vec<i32>>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct PassiveOption {
    #[serde(rename(deserialize = "type"))]
    pub option_type: String,
    pub key_stat: String,
    pub key_index: i32,
    pub value: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectData {
    pub effects: Vec<CombatEffectDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct CombatEffectDetail {
    pub ratio: i32,
    pub cooldown: i32,
    pub conditions: Vec<CombatEffectCondition>,
    pub actions: Vec<CombatEffectAction>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectCondition {
    #[serde(alias = "type")]
    pub condition_type: String,
    pub actor_type: String,
    pub arg: i32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct CombatEffectAction {
    pub action_type: String,
    pub actor_type: String,
    pub args: Vec<i32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase", default)]
pub struct Npc {
    pub id: Option<i32>,
    pub name: Option<String>,
    pub grade: String,
    #[serde(rename = "type")]
    pub npc_type: Option<String>,
    pub hp_bars: u32,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct Esther {
    pub name: String,
    pub icon: String,
    pub skills: Vec<i32>,
    #[serde(alias = "npcs")]
    pub npc_ids: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalResourceAddon {
    #[serde(rename = "type", default, deserialize_with = "null_or_default")]
    pub addon_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub stat_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub key_index: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub key_value: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAbilityLevelData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub values: Vec<i64>,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addons: Vec<ExternalResourceAddon>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAbilityData {
    pub id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub feature_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: HashMap<u32, ExternalAbilityLevelData>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalItemData {
    pub id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub category: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub tier: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub grade: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub balance_level: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub level_option_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub item_amplification_base_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub static_option_ids: Vec<u32>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ExternalItemOptionLevel {
    #[serde(alias = "weapon_power_", default, deserialize_with = "null_or_default")]
    pub weapon_power: i64,
    #[serde(
        alias = "physical_defense_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub physical_defense: i64,
    #[serde(
        alias = "magic_defense_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub magic_defense: i64,
    #[serde(alias = "strength_", default, deserialize_with = "null_or_default")]
    pub strength: i64,
    #[serde(alias = "dexterity_", default, deserialize_with = "null_or_default")]
    pub dexterity: i64,
    #[serde(alias = "intelligence_", default, deserialize_with = "null_or_default")]
    pub intelligence: i64,
    #[serde(alias = "vitality_", default, deserialize_with = "null_or_default")]
    pub vitality: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalItemLevelOptionData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: HashMap<u32, ExternalItemOptionLevel>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ExternalItemAmplificationLevel {
    #[serde(
        alias = "balance_level_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub balance_level: i64,
    #[serde(
        alias = "stage_bonus_stat_rate_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub stage_bonus_stat_rate: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalItemAmplificationBaseData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: HashMap<u32, ExternalItemAmplificationLevel>,
}

#[derive(Debug, Default, Deserialize, Clone)]
pub struct ExternalItemGradeStaticOptionData {
    #[serde(alias = "weapon_power_", default, deserialize_with = "null_or_default")]
    pub weapon_power: i64,
    #[serde(
        alias = "physical_defense_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub physical_defense: i64,
    #[serde(
        alias = "magic_defense_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub magic_defense: i64,
    #[serde(alias = "strength_", default, deserialize_with = "null_or_default")]
    pub strength: i64,
    #[serde(alias = "dexterity_", default, deserialize_with = "null_or_default")]
    pub dexterity: i64,
    #[serde(alias = "intelligence_", default, deserialize_with = "null_or_default")]
    pub intelligence: i64,
    #[serde(alias = "vitality_", default, deserialize_with = "null_or_default")]
    pub vitality: i64,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addons: Vec<ExternalResourceAddon>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAddonContainer {
    #[serde(default, deserialize_with = "null_or_default")]
    pub addons: Vec<ExternalResourceAddon>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkPassiveData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: HashMap<u32, ExternalAbilityLevelData>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkPassiveKarmaData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub ranks: HashMap<u32, ExternalAddonContainer>,
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: HashMap<u32, ExternalAddonContainer>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCardBookLevel {
    #[serde(
        alias = "required_card_count_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub required_card_count: u32,
    #[serde(
        alias = "required_awakening_level_sum_",
        default,
        deserialize_with = "null_or_default"
    )]
    pub required_awakening_level_sum: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub damage_attr: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addons: Vec<ExternalResourceAddon>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalCardBookData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub card_ids: Vec<u32>,
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: Vec<ExternalCardBookLevel>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridCoreOptionSlot {
    #[serde(default, deserialize_with = "null_or_default")]
    pub option_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub required_points: u32,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridCoreData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub options: Vec<ExternalArkGridCoreOptionSlot>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridCoreOptionData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub addons: Vec<ExternalResourceAddon>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridGemLevel {
    #[serde(default, deserialize_with = "null_or_default")]
    pub level: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addon_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub stat_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addon_index: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addon_value: i64,
    #[serde(default, deserialize_with = "null_or_default")]
    pub addon_value_override: i64,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridOptionGroup {
    #[serde(default, deserialize_with = "null_or_default")]
    pub option_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub levels: Vec<ExternalArkGridGemLevel>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridGemData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub option_groups: Vec<ExternalArkGridOptionGroup>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalArkGridData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub cores: HashMap<u32, ExternalArkGridCoreData>,
    #[serde(rename = "coreOptions", default, deserialize_with = "null_or_default")]
    pub core_options: HashMap<u32, ExternalArkGridCoreOptionData>,
    #[serde(default, deserialize_with = "null_or_default")]
    pub gems: HashMap<u32, ExternalArkGridGemData>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalAddonSkillFeature {
    pub id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub desc: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub skill_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub skill_group_id: u32,
    #[serde(rename = "type", default, deserialize_with = "null_or_default")]
    pub feature_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub target: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub parameter_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub parameters: Vec<i64>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSkillFeatureTripodEntry {
    #[serde(rename = "type", default, deserialize_with = "null_or_default")]
    pub feature_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub level: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub target_mode_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub passive_option_value: i64,
    #[serde(default, deserialize_with = "null_or_default")]
    pub parameter_type: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub parameters: Vec<i64>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSkillFeatureTripod {
    #[serde(default, deserialize_with = "null_or_default")]
    pub key: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub desc: String,
    #[serde(default, deserialize_with = "null_or_default")]
    pub entries: Vec<ExternalSkillFeatureTripodEntry>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalSkillFeatureData {
    #[serde(default, deserialize_with = "null_or_default")]
    pub skill_id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub tripods: HashMap<u32, ExternalSkillFeatureTripod>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ExternalItemClassOptionData {
    pub id: u32,
    #[serde(default, deserialize_with = "null_or_default")]
    pub class_options: HashMap<u32, ExternalResourceAddon>,
}
