#![allow(dead_code)]

use tauri::{LogicalPosition, LogicalSize, Position, Size};
use tauri_plugin_window_state::StateFlags;
use window_vibrancy::Color;

pub const WINDOW_MS: i64 = 5_000;
pub const WINDOW_S: i64 = 5;
pub const DB_VERSION: i32 = 5;
pub const METER_WINDOW_LABEL: &str = "main";
pub const METER_MINI_WINDOW_LABEL: &str = "mini";
pub const LOGS_WINDOW_LABEL: &str = "logs";
pub const DATABASE_PATH: &str = "encounters.db";
pub const SETTINGS_PATH: &str = "settings.json";
pub const DEFAULT_SETTINGS_PATH: &str = "settings.template.json";
pub const LOCAL_PLAYERS_PATH: &str = "local_players.json";
pub const REGION_PATH: &str = "current_region";
pub const STEAM_GAME_URL: &str = "steam://rungameid/1599340";
pub const GAME_EXE_NAME: &str = "LOSTARK.exe";
pub const TASK_NAME: &str = "LOA_Logs_Auto_Start";
pub const DEFAULT_BLUR: Color = (10, 10, 10, 50);
pub const DEFAULT_PORT: u16 = 6040;
pub const WINDOW_POSITION: Position = Position::Logical(LogicalPosition {
    x: 100.0,
    y: 100.0,
});
pub const DEFAULT_MINI_METER_WINDOW_SIZE: Size = Size::Logical(LogicalSize {
    width: 1280.0,
    height: 200.0,
});
pub const DEFAULT_METER_WINDOW_SIZE: Size = Size::Logical(LogicalSize {
    width: 500.0,
    height: 350.0,
});
pub const WINDOW_STATE_FLAGS: StateFlags = StateFlags::from_bits_truncate(
    StateFlags::FULLSCREEN.bits()
        | StateFlags::MAXIMIZED.bits()
        | StateFlags::POSITION.bits()
        | StateFlags::SIZE.bits()
        | StateFlags::VISIBLE.bits(),
);