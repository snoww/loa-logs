use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri_plugin_opener::open_path;

use crate::app;

pub fn show_dialog_on_panic(panic_info: &std::panic::PanicHookInfo) {
    let open_button = "Open Logs Folder";

    let dialog = MessageDialog::new()
        .set_title("An Unexpected Error")
        .set_description(format!(r#"
LOA Logs has {panic_info}

There's a log file named "loa_logs_rCURRENT.log" next to executale.

If the issue persists, report it to the developers in Discord.
        "#))
        .set_level(MessageLevel::Error)
        .set_buttons(MessageButtons::OkCustom(open_button.to_string()));

    let result = dialog.show();
    if cfg!(target_os = "windows") && result == MessageDialogResult::Custom(open_button.to_string())
        || cfg!(target_os = "linux") && result == MessageDialogResult::Ok
    {
        let _ = open_path(app::path::log_dir(), None::<&str>);
    }
}

pub fn set_hook() {
    std::panic::set_hook(Box::new(|info| {
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
            app::panic::show_dialog_on_panic(info);
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
