use tauri_plugin_window_state::StateFlags;

pub const DB_VERSION: i32 = 5;
pub const SETTINGS_FILE_NAME: &str = "settings.json";
pub const DATABASE_FILE_NAME: &str = "encounters.db";
pub const METER_WINDOW_LABEL: &str = "main";
pub const LOGS_WINDOW_LABEL: &str = "logs";
pub const WINDOW_STATE_FLAGS: StateFlags = StateFlags::from_bits_truncate(
    StateFlags::FULLSCREEN.bits()
        | StateFlags::MAXIMIZED.bits()
        | StateFlags::POSITION.bits()
        | StateFlags::SIZE.bits()
        | StateFlags::VISIBLE.bits(),
);
