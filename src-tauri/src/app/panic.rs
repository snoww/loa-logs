use rfd::{MessageButtons, MessageDialog, MessageDialogResult, MessageLevel};
use tauri_plugin_opener::OpenerExt;

use crate::app;

fn show_dialog_on_panic(app: &tauri::AppHandle, panic_message: &str) {
    const LOG_FILENAME: &str = "loa_logs_rCURRENT.log";
    const BUTTON_OPEN: &str = "Show Log File";

    let version = &app.package_info().version;
    let dialog = MessageDialog::new()
        .set_title("An Unexpected Error")
        .set_description(format!(
            r#"
LOA Logs v{version} has {panic_message}

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
        let message = format!("{info}");
        log::error!("{}", message.replace('\n', " "));
        log::logger().flush();

        if !cfg!(debug_assertions) {
            show_dialog_on_panic(&app, &message);
        }
    }));
}
