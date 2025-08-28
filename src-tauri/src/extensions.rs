use tauri::{AppHandle, Manager, Window};

use crate::constants::LOGS_WINDOW_LABEL;

pub trait AppHandleExtensions {
    fn get_logs_window(&self) -> Option<Window>;
}

impl AppHandleExtensions for AppHandle {
    fn get_logs_window(&self) -> Option<Window> {
        self.get_window(LOGS_WINDOW_LABEL)
    }
}