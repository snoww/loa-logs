use crate::data::*;
use hashbrown::{HashMap, HashSet};
use std::sync::Once;

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
        SKILL_DATA.set(read("Skill.json")).unwrap();
        SKILL_EFFECT_DATA.set(read("SkillEffect.json")).unwrap();
        COMBAT_EFFECT_DATA.set(read("CombatEffect.json")).unwrap();
        EXTERNAL_ABILITY_DATA.set(read("Ability.json")).unwrap();
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
