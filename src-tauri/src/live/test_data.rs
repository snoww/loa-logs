use crate::data::*;
use hashbrown::{HashMap, HashSet};
use std::sync::Once;

pub const POINT_ABILITY_ID: u32 = 4_000_000_003;
pub const GEM_ITEM_ID: u32 = 4_000_000_004;
pub const POINT_ABILITY_SKILL_ID: u32 = 4_000_000_005;
pub const NEXT_ABILITY_ID: u32 = 4_000_000_006;
pub const SECOND_NEXT_ABILITY_ID: u32 = 4_000_000_008;
pub const NEXT_SKILL_ID: u32 = 7_000_001;
pub const NEXT_MARKER_ID: u32 = 7_000_003;

pub fn initialize() {
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        fn read<T: serde::de::DeserializeOwned>(name: &str) -> T {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("meter-data")
                .join(name);
            serde_json::from_slice(&std::fs::read(path).unwrap()).unwrap()
        }
        NPC_WINDOW_DATA.set(read("NpcWindows.json")).unwrap();
        SKILL_BUFF_DATA.set(read("SkillBuff.json")).unwrap();
        let mut skills: HashMap<u32, crate::models::SkillData> = read("Skill.json");
        skills.insert(
            NEXT_SKILL_ID,
            crate::models::SkillData {
                id: NEXT_SKILL_ID as i32,
                groups: Some(vec![7_000_002]),
                ..Default::default()
            },
        );
        SKILL_DATA.set(skills).unwrap();
        SKILL_EFFECT_DATA.set(read("SkillEffect.json")).unwrap();
        EXTERNAL_SKILL_FEATURE_DATA
            .set(read("SkillFeature.json"))
            .unwrap();
        COMBAT_EFFECT_DATA.set(read("CombatEffect.json")).unwrap();
        let mut abilities: HashMap<u32, crate::models::ExternalAbilityData> = read("Ability.json");
        abilities.insert(
            POINT_ABILITY_ID,
            crate::models::ExternalAbilityData {
                id: POINT_ABILITY_ID,
                levels: [(1, 125), (2, 250), (3, 375)]
                    .into_iter()
                    .map(|(level, value)| {
                        (
                            level,
                            crate::models::ExternalAbilityLevelData {
                                addons: vec![crate::models::ExternalResourceAddon {
                                    addon_type: "skill_status_effect_stat_multiplier".to_string(),
                                    key_index: POINT_ABILITY_SKILL_ID,
                                    key_value: value,
                                    ..Default::default()
                                }],
                                ..Default::default()
                            },
                        )
                    })
                    .collect(),
                ..Default::default()
            },
        );
        for (ability_id, damage_rate, critical_rate) in [
            (NEXT_ABILITY_ID, 2000, 500),
            (SECOND_NEXT_ABILITY_ID, 1000, 300),
        ] {
            abilities.insert(
                ability_id,
                crate::models::ExternalAbilityData {
                    feature_type: "ag_alpha_next_skill_enhance".to_string(),
                    levels: HashMap::from([(
                        1,
                        crate::models::ExternalAbilityLevelData {
                            values: vec![
                                1,
                                i64::from(NEXT_SKILL_ID),
                                0,
                                damage_rate,
                                critical_rate,
                                i64::from(NEXT_MARKER_ID),
                                0,
                                1,
                                0,
                                0,
                                0,
                            ],
                            ..Default::default()
                        },
                    )]),
                    ..Default::default()
                },
            );
        }
        EXTERNAL_ABILITY_DATA.set(abilities).unwrap();
        EXTERNAL_ITEM_DATA
            .set(HashMap::from([(
                GEM_ITEM_ID,
                crate::models::ExternalItemData {
                    id: GEM_ITEM_ID,
                    tier: 4,
                    ..Default::default()
                },
            )]))
            .unwrap();
        STAT_TYPE_NAME_MAP.set(HashMap::new()).unwrap();
        STAT_TYPE_MAP.set(read("StatType.json")).unwrap();
        SUPPORT_AP_GROUP.set(HashSet::new()).unwrap();
        SUPPORT_IDENTITY_GROUP
            .set(HashSet::from([211400, 368000, 310501, 480018]))
            .unwrap();
        SUPPORT_MARKING_GROUP.set(HashSet::from([210230])).unwrap();
        RDPS_ADDITIONAL_IDENTITY_GROUP
            .set(HashSet::from([214020, 360102, 480024]))
            .unwrap();
    });
}
