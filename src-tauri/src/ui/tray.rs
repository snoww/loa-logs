use strum::EnumProperty;
use strum_macros::{AsRefStr, EnumString};
use tauri::{
    AppHandle, Runtime,
    menu::{Menu, MenuBuilder},
    tray::TrayIconBuilder,
};

use crate::ui::{on_menu_event, on_tray_icon_event};

#[derive(Debug, EnumString, EnumProperty, AsRefStr)]
#[strum(serialize_all = "kebab_case")]
pub enum TrayCommand {
    #[strum(props(label = "Show Logs"))]
    ShowLogs,

    #[strum(props(label = "Show Meter"))]
    ShowMeter,

    #[strum(props(label = "Hide Meter"))]
    Hide,

    #[strum(props(label = "Start Lost Ark"))]
    StartLoa,

    #[strum(props(label = "Reset Window"))]
    Reset,

    #[strum(props(label = "Quit"))]
    Quit,
}

pub struct LoaMenuBuilder<'a, R: Runtime>(MenuBuilder<'a, R, AppHandle<R>>);

impl<'a, R: Runtime> LoaMenuBuilder<'a, R> {
    pub fn new(app: &'a AppHandle<R>) -> Self {
        Self(MenuBuilder::new(app))
    }

    pub fn command(mut self, cmd: TrayCommand) -> Self {
        self.0 = self.0.text(cmd.as_ref(), cmd.get_str("label").unwrap());
        self
    }

    pub fn separator(mut self) -> Self {
        self.0 = self.0.separator();
        self
    }

    pub fn build(self) -> tauri::Result<Menu<R>> {
        self.0.build()
    }
}

pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    let menu = LoaMenuBuilder::new(app)
        .command(TrayCommand::ShowLogs)
        .separator()
        .command(TrayCommand::ShowMeter)
        .command(TrayCommand::Hide)
        .separator()
        .command(TrayCommand::StartLoa)
        .separator()
        .command(TrayCommand::Reset)
        .separator()
        .command(TrayCommand::Quit)
        .build()?;

    let tray = TrayIconBuilder::new()
        .icon(tauri::include_image!("icons/icon.png"))
        .menu(&menu)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_tray_icon_event)
        .show_menu_on_left_click(false);

    tray.build(app).map(|_| ())
}
