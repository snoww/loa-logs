use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri_plugin_opener::OpenerExt;

use crate::app;

pub fn show_dialog_on_panic(app: &tauri::AppHandle, panic_info: &std::panic::PanicHookInfo) {
    const LOG_FILENAME: &str = "loa_logs_rCURRENT.log";
    const BUTTON_OPEN: &str = "Reveal Log File";

    let version = &app.package_info().version;
    let dialog = MessageDialog::new()
        .set_title("An Unexpected Error")
        .set_description(format!(
            r#"
LOA Logs v{version} has {panic_info}

There's a log file named "{LOG_FILENAME}" next to the executable.

If the issue persists, report it to the developers on Discord.
        "#
        ))
        .set_level(MessageLevel::Error)
        .set_buttons(MessageButtons::OkCustom(BUTTON_OPEN.to_string()));

    let result = dialog.show();
    if cfg!(target_os = "windows") && result == MessageDialogResult::Custom(BUTTON_OPEN.to_string())
        || cfg!(target_os = "linux") && result == MessageDialogResult::Ok
    {
        let log_path = app::path::log_dir().join(LOG_FILENAME);
        let _ = app.opener().reveal_item_in_dir(log_path);
    }
}

pub fn set_hook(app: &tauri::AppHandle) {
    let app = app.clone();
    
    std::panic::set_hook(Box::new(move |info| {
        let message = if let Some(location) = info.location()
            && let Some(payload) = payload_as_str(info)
        {
            format!("panicked at {location}: {payload}")
        } else {
            format!("panicked: {info:?}")
        };
        log::error!("{message}");
        log::logger().flush();

        if !cfg!(debug_assertions) {
            app::panic::show_dialog_on_panic(&app, info);
        }
    }));
}

/// replace with `PanicHookInfo::payload_as_str()` when stabilized
/// in 1.91.0 https://github.com/rust-lang/rust/pull/144861
fn payload_as_str<'a>(info: &'a std::panic::PanicHookInfo) -> Option<&'a str> {
    if let Some(s) = info.payload().downcast_ref::<&str>() {
        Some(s)
    } else if let Some(s) = info.payload().downcast_ref::<String>() {
        Some(s)
    } else {
        None
    }
}
