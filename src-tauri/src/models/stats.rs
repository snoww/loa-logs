use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use strum_macros::AsRefStr;

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsArgs {
    pub date_from: DateTime<Utc>,
    pub date_to: DateTime<Utc>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsResult {
    pub items: Vec<RaidStats>
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidMetric {
    pub kind: RaidType,
    pub played_as_support: bool,
    pub dps: i64,
    pub support_ap: f32,
    pub support_brand: f32,
    pub support_identity: f32,
    pub support_hyper: f32
}

#[derive(Debug, Hash, Serialize, Deserialize, PartialEq, Eq, Clone, Copy, AsRefStr)]
#[serde(rename_all = "camelCase")]
pub enum RaidType {
    Unknown,
    #[strum(serialize = "Behemoth")]
    BehemothG1,
    #[strum(serialize = "Behemoth")]
    BehemothG2,
    #[strum(serialize = "Narkiel")]
    EchidnaG1,
    #[strum(serialize = "Echidna")]
    EchidnaG2,
    #[strum(serialize = "Akkan")]
    Act1G1,
    #[strum(serialize = "Aegir")]
    Act1G2,
    #[strum(serialize = "Mordum")]
    Act3G3,
    #[strum(serialize = "Naitreya")]
    Act3G2,
    #[strum(serialize = "Thaemine & Infernas")]
    Act3G1,
    #[strum(serialize = "Narok")]
    Act2G1,
    #[strum(serialize = "Brelshaza")]
    Act2G2,
    #[strum(serialize = "Tarkal")]
    StrikeG1,
    Drextalas,
    Skolakia,
    Argeos,
}

impl RaidType {
    pub fn get_bosses(&self) -> &'static [&'static str] {
        match self {
            Self::Act3G3 => &["Mordum, the Abyssal Punisher", "Flash of Punishment"],
            Self::Act3G2 => &["Blossoming Fear, Naitreya"],
            Self::Act3G1 => &["Infernas"],
            Self::Act2G2 => &["Phantom Manifester Brelshaza"],
            Self::Act2G1 => &["Narok the Butcher"],
            Self::Act1G2 => &["Aegir, the Oppressor"],
            Self::Act1G1 => &["Akkan, Lord of Death"],
            Self::BehemothG1 => &["Behemoth, the Storm Commander"],
            Self::BehemothG2 => &["Behemoth, Cruel Storm Slayer"],
            Self::EchidnaG1 => &["Red Doom Narkiel"],
            Self::EchidnaG2 => &["Covetous Master Echidna"],
            Self::StrikeG1 => &["Flame of Darkness, Tarkal"],
            Self::Skolakia => &["Skolakia"],
            Self::Drextalas => &["Drextalas"],
            Self::Argeos => &["Argeos"],
            Self::Unknown => &[],
        }
    }

    pub fn order(&self) -> u32 {
        match self {
            Self::BehemothG2 => 5,
            Self::EchidnaG2 => 4,
            Self::Act1G2 => 3,
            Self::Act2G2 => 2,
            Self::Act3G3 => 1,
            Self::StrikeG1 => 0,
            _ if self.is_guardian_raid() => 999,
            _ => 6
        }
    }

    pub fn is_final_gate(&self) -> bool {
        match self {
            Self::BehemothG2 => true,
            Self::EchidnaG2 => true,
            Self::Act1G2 => true,
            Self::Act3G3 => true,
            Self::Act2G2 => true,
            Self::StrikeG1 => true,
            Self::Argeos => true,
            Self::Skolakia => true,
            Self::Drextalas => true,
            _ => false
        }
    }

    pub fn is_guardian_raid(&self) -> bool {
        match self {
            Self::Argeos => true,
            Self::Skolakia => true,
            Self::Drextalas => true,
            _ => false
        }
    }
}

impl From<String> for RaidType {
    fn from(value: String) -> Self {
        match value.as_str() {
            "Mordum, the Abyssal Punisher" => Self::Act3G3,
            "Flash of Punishment" => Self::Act3G3,
            "Blossoming Fear, Naitreya" => Self::Act3G2,
            "Infernas" => Self::Act3G1,
            "Phantom Manifester Brelshaza" => Self::Act2G2,
            "Narok the Butcher" => Self::Act2G1,
            "Aegir, the Oppressor" => Self::Act1G2,
            "Akkan, Lord of Death" => Self::Act1G1,
            "Behemoth, the Storm Commander" => Self::BehemothG1,
            "Behemoth, Cruel Storm Slayer" => Self::BehemothG2,
            "Red Doom Narkiel" => Self::EchidnaG1,
            "Covetous Master Echidna" => Self::EchidnaG2,
            "Flame of Darkness, Tarkal" => Self::StrikeG1,
            "Skolakia" => Self::Skolakia,
            "Drextalas" => Self::Drextalas,
            "Argeos" => Self::Argeos,
            _ => Self::Unknown
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct RaidStats {
    pub name: String,
    pub order: u32,
    pub raid_type: RaidType,
    pub count: u32,
    pub dps: Option<Unit>,
    pub uptimes: Option<(String, String, String, String)>,
    pub instances: Vec<RaidMetric>,
    pub is_final_gate: bool,
    pub is_guardian_raid: bool
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Unit {
    pub formatted: String,
    pub raw: i64
}