use anyhow::{Context, Result, anyhow};
use hashbrown::{HashMap, HashSet};
use ipnet::Ipv4Net;
use serde::de::DeserializeOwned;
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::{fs, ops::Deref, path::Path, sync::OnceLock};

use crate::models::*;

pub static COMBAT_EFFECT_DATA: OnceLockWrapper<HashMap<i32, CombatEffectData>> =
    OnceLockWrapper::new();
pub static ENGRAVING_DATA: OnceLockWrapper<HashMap<u32, EngravingData>> = OnceLockWrapper::new();
pub static SKILL_BUFF_DATA: OnceLockWrapper<HashMap<u32, SkillBuffData>> = OnceLockWrapper::new();
pub static SKILL_DATA: OnceLockWrapper<HashMap<u32, SkillData>> = OnceLockWrapper::new();
pub static SKILL_EFFECT_DATA: OnceLockWrapper<HashMap<u32, SkillEffectData>> =
    OnceLockWrapper::new();
pub static SUPPORT_AP_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static SUPPORT_IDENTITY_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static RDPS_ADDITIONAL_IDENTITY_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static STAT_TYPE_MAP: OnceLockWrapper<HashMap<String, u32>> = OnceLockWrapper::new();
pub static STAT_TYPE_NAME_MAP: OnceLockWrapper<HashMap<u32, String>> = OnceLockWrapper::new();
pub static ESTHER_DATA: OnceLockWrapper<Vec<Esther>> = OnceLockWrapper::new();
pub static NPC_DATA: OnceLockWrapper<HashMap<u32, Npc>> = OnceLockWrapper::new();
pub static GEM_SKILL_MAP: OnceLockWrapper<HashMap<u32, Vec<u32>>> = OnceLockWrapper::new();
pub static RAID_MAP: OnceLockWrapper<HashMap<String, String>> = OnceLockWrapper::new();
pub static IP_RANGES: OnceLockWrapper<Vec<IpRangeEntry>> = OnceLockWrapper::new();
pub static SUPPORT_MARKING_GROUP: OnceLockWrapper<HashSet<u32>> = OnceLockWrapper::new();
pub static EXTERNAL_ABILITY_DATA: OnceLockWrapper<HashMap<u32, ExternalAbilityData>> =
    OnceLockWrapper::new();
pub static EXTERNAL_ITEM_DATA: OnceLockWrapper<HashMap<u32, ExternalItemData>> =
    OnceLockWrapper::new();
pub static EXTERNAL_ITEM_LEVEL_OPTION_DATA: OnceLockWrapper<
    HashMap<u32, ExternalItemLevelOptionData>,
> = OnceLockWrapper::new();
pub static EXTERNAL_ITEM_AMPLIFICATION_BASE_DATA: OnceLockWrapper<
    HashMap<u32, ExternalItemAmplificationBaseData>,
> = OnceLockWrapper::new();
pub static EXTERNAL_ITEM_GRADE_STATIC_OPTION_DATA: OnceLockWrapper<
    HashMap<u32, ExternalItemGradeStaticOptionData>,
> = OnceLockWrapper::new();
pub static EXTERNAL_ARK_PASSIVE_DATA: OnceLockWrapper<HashMap<u32, ExternalArkPassiveData>> =
    OnceLockWrapper::new();
pub static EXTERNAL_ARK_PASSIVE_KARMA_DATA: OnceLockWrapper<
    HashMap<u32, ExternalArkPassiveKarmaData>,
> = OnceLockWrapper::new();
pub static EXTERNAL_CARD_BOOK_DATA: OnceLockWrapper<HashMap<u32, ExternalCardBookData>> =
    OnceLockWrapper::new();
pub static EXTERNAL_ARK_GRID_DATA: OnceLockWrapper<ExternalArkGridData> = OnceLockWrapper::new();
pub static EXTERNAL_ARK_GRID_GEM_LEVELS_BY_OPTION_ID: OnceLockWrapper<
    HashMap<u32, Vec<ExternalArkGridGemLevel>>,
> = OnceLockWrapper::new();
pub static EXTERNAL_ADDON_SKILL_FEATURE_DATA: OnceLockWrapper<
    HashMap<u32, ExternalAddonSkillFeature>,
> = OnceLockWrapper::new();
pub static EXTERNAL_ITEM_CLASS_OPTION_DATA: OnceLockWrapper<
    HashMap<u32, ExternalItemClassOptionData>,
> = OnceLockWrapper::new();
pub static EXTERNAL_SKILL_FEATURE_DATA: OnceLockWrapper<HashMap<u32, ExternalSkillFeatureData>> =
    OnceLockWrapper::new();

pub struct OnceLockWrapper<T>(OnceLock<T>);

impl<T> OnceLockWrapper<T> {
    pub const fn new() -> Self {
        Self(OnceLock::new())
    }

    pub fn set(&self, value: T) -> Result<()> {
        self.0
            .set(value)
            .map_err(|_| anyhow!("OnceLockWrapper already initialized"))
    }
}

impl<T> Deref for OnceLockWrapper<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.0.get().expect("OnceLockWrapper not initialized")
    }
}

pub struct AssetPreloader;

fn load<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let s = fs::read_to_string(path).with_context(|| anyhow!("Missing file at: {path:?}"))?;
    serde_json::from_str::<T>(&s).with_context(|| anyhow!("Error parsing JSON in {path:?}"))
}

fn load_meter_data<T: DeserializeOwned>(resource_dir: &Path, file_name: &str) -> Result<T> {
    load(&resource_dir.join("meter-data").join(file_name))
}

fn load_extra_meter_data<T: DeserializeOwned>(
    resource_dir: &Path,
    file_name: &str,
    dump_file_name: &str,
) -> Result<T> {
    let bundled = resource_dir.join("meter-data").join(file_name);
    if bundled.exists() {
        return load(&bundled);
    }

    let dump_path = Path::new(r"I:\Engine\honing\dump\External\Resources").join(dump_file_name);
    if dump_path.exists() {
        return load(&dump_path);
    }

    Err(anyhow!(
        "Missing extra meter-data file {:?}. Checked {:?} and dump fallback {:?}",
        file_name,
        bundled,
        dump_path
    ))
}

fn legacy_stat_alias(name: &str) -> Option<&'static str> {
    match name {
        "Crit" => Some("criticalhit"),
        "Specialization" => Some("specialty"),
        "Domination" => Some("oppression"),
        "Swiftness" => Some("rapidity"),
        "Expertise" => Some("mastery"),
        "CRITICALHIT_X" => Some("criticalhit_x"),
        "SPECIALTY_X" => Some("specialty_x"),
        "OPPRESSION_X" => Some("oppression_x"),
        "RAPIDITY_X" => Some("rapidity_x"),
        "MASTERY_X" => Some("mastery_x"),
        "OutgoingDamage" => Some("outgoing_damage"),
        "OutgoingDamage0" => Some("outgoing_damage0"),
        "OutgoingDamage1" => Some("outgoing_damage1"),
        "OutgoingDamage2" => Some("outgoing_damage2"),
        "SupportGauge" => Some("support_gauge"),
        "SupportGauge0" => Some("support_gauge0"),
        "SupportGauge1" => Some("support_gauge1"),
        "SupportGauge2" => Some("support_gauge2"),
        "AllyDamageEnhancementEffect" => Some("ally_damage_enhancement_effect"),
        "AllyAtkPowerEnhancementEffect" => Some("ally_atk_power_enhancement_effect"),
        "ShieldForPartyMembers" => Some("shield_for_party_members"),
        "RecoveryForPartyMembers" => Some("recovery_for_party_members"),
        "ArkPassivePoint" => Some("ark_passive_point"),
        _ => None,
    }
}

fn build_stat_type_map(mut canonical: HashMap<String, u32>) -> HashMap<String, u32> {
    let existing = canonical
        .iter()
        .map(|(name, value)| (name.clone(), *value))
        .collect::<Vec<_>>();
    for (name, value) in existing {
        canonical.entry(name.to_ascii_lowercase()).or_insert(value);
        if let Some(alias) = legacy_stat_alias(&name) {
            canonical.entry(alias.to_string()).or_insert(value);
        }
    }
    canonical
}

fn build_stat_type_name_map(canonical: &HashMap<String, u32>) -> HashMap<u32, String> {
    let mut names = HashMap::new();
    for (name, id) in canonical {
        let preferred_name = legacy_stat_alias(name)
            .map(str::to_string)
            .unwrap_or_else(|| name.to_ascii_lowercase());
        names.entry(*id).or_insert(preferred_name);
    }
    names
}

pub fn stat_type_name_from_id(stat_id: u32) -> Option<String> {
    STAT_TYPE_NAME_MAP.get(&stat_id).cloned()
}

impl AssetPreloader {
    pub fn new(resource_dir: &Path) -> Result<Self> {
        COMBAT_EFFECT_DATA.set(load_meter_data(resource_dir, "CombatEffect.json")?)?;
        ENGRAVING_DATA.set(load_meter_data(resource_dir, "Ability.json")?)?;
        SKILL_BUFF_DATA.set(load_meter_data(resource_dir, "SkillBuff.json")?)?;
        SKILL_DATA.set(load_meter_data(resource_dir, "Skill.json")?)?;
        SKILL_EFFECT_DATA.set(load_meter_data(resource_dir, "SkillEffect.json")?)?;
        let stat_type_data: HashMap<String, u32> = load_meter_data(resource_dir, "StatType.json")?;
        STAT_TYPE_NAME_MAP.set(build_stat_type_name_map(&stat_type_data))?;
        STAT_TYPE_MAP.set(build_stat_type_map(stat_type_data))?;
        ESTHER_DATA.set(load_meter_data(resource_dir, "Esther.json")?)?;
        NPC_DATA.set(load_meter_data(resource_dir, "Npc.json")?)?;
        GEM_SKILL_MAP.set({
            let raw: HashMap<String, (String, String, Vec<u32>)> =
                load_meter_data(resource_dir, "GemSkillGroup.json")?;
            raw.into_iter()
                .filter_map(|(key, entry)| key.parse::<u32>().ok().map(|id| (id, entry.2)))
                .collect()
        })?;
        RAID_MAP.set({
            let encounters: HashMap<String, HashMap<String, Vec<String>>> =
                load_meter_data(resource_dir, "encounters.json")?;
            encounters
                .values()
                .flat_map(|raid| raid.iter())
                .flat_map(|(gate, bosses)| {
                    bosses.iter().map(move |boss| (boss.clone(), gate.clone()))
                })
                .collect()
        })?;
        SUPPORT_AP_GROUP.set(HashSet::from([
            101204, // bard
            101105, // paladin
            314004, // artist
            480030, // valkyrie
        ]))?;
        SUPPORT_MARKING_GROUP.set(HashSet::from([
            210230, // shared support brand unique group
        ]))?;
        SUPPORT_IDENTITY_GROUP.set(HashSet::from([
            211400, // bard serenade of courage
            368000, // paladin holy aura
            310501, // artist moonfall
            480018, // valkyrie release light
        ]))?;
        RDPS_ADDITIONAL_IDENTITY_GROUP.set(HashSet::from([
            214020, // bard major chord
            360102, // paladin holy aura group
            480024, // valkyrie wings of freedom
        ]))?;
        IP_RANGES.set({
            let raw: AwsIpRanges = load_meter_data(resource_dir, "ip-ranges.json")?;
            raw.prefixes
                .into_iter()
                .filter(|p| {
                    p.region == "us-east-1" || p.region == "us-west-2" || p.region == "eu-central-1"
                })
                .filter_map(|p| {
                    Ipv4Net::from_str(&p.ip_prefix)
                        .ok()
                        .map(|net| IpRangeEntry {
                            net,
                            region: p.region,
                        })
                })
                .collect()
        })?;

        EXTERNAL_ABILITY_DATA.set(load_extra_meter_data(
            resource_dir,
            "Ability.json",
            "Ability_EnumStrings.json",
        )?)?;
        EXTERNAL_ITEM_DATA.set(load_extra_meter_data(
            resource_dir,
            "Item.json",
            "Item_EnumStrings.json",
        )?)?;
        EXTERNAL_ITEM_LEVEL_OPTION_DATA.set(load_extra_meter_data(
            resource_dir,
            "ItemLevelOption.json",
            "ItemLevelOption_EnumStrings.json",
        )?)?;
        EXTERNAL_ITEM_AMPLIFICATION_BASE_DATA.set(load_extra_meter_data(
            resource_dir,
            "ItemAmplificationBase.json",
            "ItemAmplificationBase_EnumStrings.json",
        )?)?;
        EXTERNAL_ITEM_GRADE_STATIC_OPTION_DATA.set(load_extra_meter_data(
            resource_dir,
            "ItemGradeStaticOption.json",
            "ItemGradeStaticOption_EnumStrings.json",
        )?)?;
        EXTERNAL_ARK_PASSIVE_DATA.set(load_extra_meter_data(
            resource_dir,
            "ArkPassive.json",
            "ArkPassive_EnumStrings.json",
        )?)?;
        EXTERNAL_ARK_PASSIVE_KARMA_DATA.set(load_extra_meter_data(
            resource_dir,
            "ArkPassiveKarma.json",
            "ArkPassiveKarma_EnumStrings.json",
        )?)?;
        EXTERNAL_CARD_BOOK_DATA.set(load_extra_meter_data(
            resource_dir,
            "CardBook.json",
            "CardBook_EnumStrings.json",
        )?)?;
        EXTERNAL_ARK_GRID_DATA.set(load_extra_meter_data(
            resource_dir,
            "ArkGrid.json",
            "ArkGrid_EnumStrings.json",
        )?)?;
        EXTERNAL_ADDON_SKILL_FEATURE_DATA.set(load_extra_meter_data(
            resource_dir,
            "AddonSkillFeature.json",
            "AddonSkillFeature_EnumStrings.json",
        )?)?;
        EXTERNAL_ITEM_CLASS_OPTION_DATA.set(load_extra_meter_data(
            resource_dir,
            "ItemClassOption.json",
            "ItemClassOption_EnumStrings.json",
        )?)?;
        EXTERNAL_SKILL_FEATURE_DATA.set(load_extra_meter_data(
            resource_dir,
            "SkillFeature.json",
            "SkillFeature_EnumStrings.json",
        )?)?;
        EXTERNAL_ARK_GRID_GEM_LEVELS_BY_OPTION_ID.set({
            let mut levels_by_option_id: HashMap<u32, Vec<ExternalArkGridGemLevel>> =
                HashMap::new();
            for gem in EXTERNAL_ARK_GRID_DATA.gems.values() {
                for option_group in &gem.option_groups {
                    levels_by_option_id
                        .entry(option_group.option_id)
                        .or_default()
                        .extend(option_group.levels.clone());
                }
            }
            levels_by_option_id
        })?;

        Ok(Self)
    }
}

#[derive(Debug, serde::Deserialize)]
struct AwsIpRanges {
    prefixes: Vec<AwsPrefix>,
}

#[derive(Debug, serde::Deserialize)]
struct AwsPrefix {
    ip_prefix: String,
    region: String,
}

pub struct IpRangeEntry {
    pub net: Ipv4Net,
    pub region: String,
}

pub fn get_region_from_ip(ip: Ipv4Addr) -> Option<String> {
    for entry in IP_RANGES.iter() {
        if entry.net.contains(&ip) {
            return match entry.region.as_str() {
                "us-east-1" | "us-west-2" => Some("NA".to_string()),
                "eu-central-1" => Some("EUC".to_string()),
                _ => None,
            };
        }
    }
    None
}
