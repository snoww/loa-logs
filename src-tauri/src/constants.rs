use tauri_plugin_window_state::StateFlags;

pub const WINDOW_MS: i64 = 5_000;
pub const WINDOW_S: i64 = 5;
pub const METER_WINDOW_LABEL: &str = "main";
pub const METER_MINI_WINDOW_LABEL: &str = "mini";
pub const LOGS_WINDOW_LABEL: &str = "logs";
pub const DATABASE_PATH: &str = "encounters.db";
pub const SETTINGS_PATH: &str = "settings.json";
pub const LOCAL_PLAYERS_PATH: &str = "local_players.json";
pub const REGION_PATH: &str = "current_region";
pub const STEAM_GAME_URL: &str = "steam://rungameid/1599340";
pub const GAME_EXE_NAME: &str = "LOSTARK.exe";
pub const WINDOW_STATE_FLAGS: StateFlags = StateFlags::from_bits_truncate(
    StateFlags::FULLSCREEN.bits()
        | StateFlags::MAXIMIZED.bits()
        | StateFlags::POSITION.bits()
        | StateFlags::SIZE.bits()
        | StateFlags::VISIBLE.bits(),
);
