use std::sync::atomic::{AtomicBool, Ordering};

use tauri::{AppHandle, Manager};

use crate::constants::{METER_MINI_WINDOW_LABEL, METER_WINDOW_LABEL};
use crate::models::Encounter;

#[derive(Default)]
pub struct EncounterUpdates {
    mini: AtomicBool,
}

impl EncounterUpdates {
    pub fn set_mini(&self, mini: bool) {
        self.mini.store(mini, Ordering::Relaxed);
    }

    pub fn window_label(&self) -> &'static str {
        if self.mini.load(Ordering::Relaxed) {
            METER_MINI_WINDOW_LABEL
        } else {
            METER_WINDOW_LABEL
        }
    }
}

pub fn send_encounter_update(app: &AppHandle, encounter: Encounter) -> anyhow::Result<()> {
    let label = app.state::<EncounterUpdates>().window_label();
    let Some(window) = app.get_webview_window(label) else {
        return Ok(());
    };

    #[cfg(target_os = "windows")]
    {
        #[derive(serde::Serialize)]
        struct Message<'a> {
            event: &'static str,
            payload: &'a Encounter,
        }

        // ExecuteScript retains snapshot-sized scripts in WebView2 even after GC.
        let message = serde_json::to_string(&Message {
            event: "encounter-update",
            payload: &encounter,
        })?;
        let app = app.clone();
        window.with_webview(move |webview| {
            // A mode change can occur while this delivery waits on the UI thread.
            if app.state::<EncounterUpdates>().window_label() != label {
                return;
            }
            let result = unsafe {
                webview.controller().CoreWebView2().and_then(|webview| {
                    webview.PostWebMessageAsJson(&windows_core::HSTRING::from(message))
                })
            };
            if let Err(error) = result {
                log::warn!("failed to send encounter update: {error}");
            }
        })?;
    }

    #[cfg(not(target_os = "windows"))]
    {
        use tauri::Emitter;
        window.emit("encounter-update", encounter)?;
    }

    Ok(())
}
