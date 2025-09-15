use std::ops::Deref;

use tauri::{AppHandle, Manager, WebviewWindow};
use tauri_plugin_window_state::WindowExt;

use crate::constants::{LOGS_WINDOW_LABEL, METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL, WINDOW_STATE_FLAGS};

pub trait AppHandleExtensions {
    fn get_meter_window(&self) -> Option<MeterWindow>;
    fn get_mini_window(&self) -> Option<OtherWindow>;
    fn get_logs_window(&self) -> Option<OtherWindow>;
    fn get_window(&self, is_mini: bool) -> Box<dyn WindowExtensions>;
}

pub trait WindowExtensions {
    fn restore_default_state(&self);
    fn restore_and_focus(&self);
}

impl AppHandleExtensions for AppHandle {
    fn get_logs_window(&self) -> Option<OtherWindow> {
        self.get_webview_window(LOGS_WINDOW_LABEL).map(OtherWindow::new)
    }
    
    fn get_meter_window(&self) -> Option<MeterWindow> {
        self.get_webview_window(METER_WINDOW_LABEL).map(MeterWindow::new)
    }
    
    fn get_mini_window(&self) -> Option<OtherWindow> {
        self.get_webview_window(METER_MINI_WINDOW_LABEL).map(OtherWindow::new)
    }
    
    fn get_window(&self, is_mini: bool) -> Box<dyn WindowExtensions> {
        let window = if is_mini {
            Box::new(self.get_mini_window().unwrap()) as Box<dyn WindowExtensions>
        } else {
            Box::new(self.get_meter_window().unwrap()) as Box<dyn WindowExtensions>
        };

        window
    }
}

impl AppHandleExtensions for &AppHandle {
    fn get_logs_window(&self) -> Option<OtherWindow> {
        self.get_webview_window(LOGS_WINDOW_LABEL).map(OtherWindow::new)
    }
    
    fn get_meter_window(&self) -> Option<MeterWindow> {
        self.get_webview_window(METER_WINDOW_LABEL).map(MeterWindow::new)
    }
    
    fn get_mini_window(&self) -> Option<OtherWindow> {
        self.get_webview_window(METER_MINI_WINDOW_LABEL).map(OtherWindow::new)
    }

    fn get_window(&self, is_mini: bool) -> Box<dyn WindowExtensions> {
        let window = if is_mini {
            Box::new(self.get_mini_window().unwrap()) as Box<dyn WindowExtensions>
        } else {
            Box::new(self.get_meter_window().unwrap()) as Box<dyn WindowExtensions>
        };

        window
    }
}

pub struct MeterWindow(WebviewWindow);

impl WindowExtensions for MeterWindow {
    fn restore_and_focus(&self) {
        unsafe {
            self.0.show().unwrap_unchecked();
            self.0.unminimize().unwrap_unchecked();
            self.0.set_focus().unwrap_unchecked();
            self.0.set_ignore_cursor_events(false).unwrap_unchecked();
        }
    }
    
    fn restore_default_state(&self) {
        unsafe { self.0.restore_state(WINDOW_STATE_FLAGS).unwrap_unchecked() }
    }
}

impl MeterWindow {
    pub fn new(window: WebviewWindow) -> Self {
        Self(window)
    }
}

impl Deref for MeterWindow {
    type Target = WebviewWindow;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct OtherWindow(WebviewWindow);

impl WindowExtensions for OtherWindow {
    fn restore_and_focus(&self) {
        unsafe {
            self.0.show().unwrap_unchecked();
            self.0.unminimize().unwrap_unchecked();
            self.0.set_focus().unwrap_unchecked();
        }
    }
    
    fn restore_default_state(&self) {
        unsafe { self.0.restore_state(WINDOW_STATE_FLAGS).unwrap_unchecked() }
    }
}

impl OtherWindow {
    pub fn new(window: WebviewWindow) -> Self {
        Self(window)
    }
}

impl Deref for OtherWindow {
    type Target = WebviewWindow;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}