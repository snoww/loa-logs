use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub general: GeneralSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct GeneralSettings {
    pub start_loa_on_start: bool,
    pub low_performance_mode: bool,
    #[serde(default = "default_true")]
    pub auto_iface: bool,
    pub port: u16,
    #[serde(default = "default_true")]
    pub always_on_top: bool,
    #[serde(default = "default_true")]
    pub boss_only_damage: bool,
    #[serde(default = "default_true")]
    pub hide_meter_on_start: bool,
    pub hide_logs_on_start: bool,
    pub mini: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}