use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub general: GeneralSettings,
    #[serde(default)]
    pub local_api: LocalApiSettings,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

/// Settings for the read-only local HTTP API that lets a browser (e.g. neria.dev)
/// read sanitized meter clears from this machine. Disabled by default and bound to
/// 127.0.0.1 only. See `crate::local_api`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct LocalApiSettings {
    pub enabled: bool,
    pub port: u16,
    pub token: String,
    pub allowed_origins: Vec<String>,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

impl Default for LocalApiSettings {
    fn default() -> Self {
        Self {
            enabled: false,
            port: default_local_api_port(),
            token: String::new(),
            allowed_origins: default_allowed_origins(),
            extra: Map::new(),
        }
    }
}

fn default_local_api_port() -> u16 {
    6041
}

fn default_allowed_origins() -> Vec<String> {
    vec![
        "https://neria.dev".to_string(),
        "https://dev-neria.niome.dev".to_string(),
        "http://localhost:5173".to_string(),
        "http://127.0.0.1:5173".to_string(),
    ]
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
    pub beta_channel: bool,
    pub exitlag_compat: bool,
    #[serde(flatten)]
    pub extra: Map<String, Value>,
}

fn default_true() -> bool {
    true
}
