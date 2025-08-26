use anyhow::Result;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
pub use linux::AutoLaunchManager;
#[cfg(target_os = "windows")]
pub use windows::AutoLaunchManager;

/// This is copied from the auto start plugin because one day it may support Task Scheduler
/// https://github.com/tauri-apps/tauri-plugin-autostart/blob/v1/src/lib.rs#L44
pub trait AutoLaunch {
    fn is_enabled(&self) -> Result<bool>;
    fn enable(&self) -> Result<()>;
    fn disable(&self) -> Result<()>;
}
