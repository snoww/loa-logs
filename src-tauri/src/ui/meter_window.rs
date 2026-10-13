use anyhow::{Context, Result};
use tauri::{AppHandle, Manager, WebviewWindow, WebviewWindowBuilder};

use crate::constants::{METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL};
use crate::settings::SettingsManager;

pub fn ensure_meter_window(app: &AppHandle, mini: bool) -> Result<WebviewWindow> {
    let label = if mini {
        METER_MINI_WINDOW_LABEL
    } else {
        METER_WINDOW_LABEL
    };
    if let Some(window) = app.get_webview_window(label) {
        return Ok(window);
    }

    let config = app
        .config()
        .app
        .windows
        .iter()
        .find(|config| config.label == label)
        .context("meter window configuration missing")?;
    let settings = app.state::<SettingsManager>().read()?;
    let always_on_top = settings
        .as_ref()
        .is_none_or(|settings| settings.general.always_on_top);

    // Keep the original labels so the window-state plugin restores existing positions.
    // Creation must run in setup or an async command, not a synchronous Windows callback.
    Ok(WebviewWindowBuilder::from_config(app, config)?
        .always_on_top(always_on_top)
        .build()?)
}
