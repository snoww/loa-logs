use std::path::Path;

use anyhow::Result;

/// Ensures that the required WinDivert files are available in the current directory.
///
/// ### Windows-specific
/// This function is only relevant on Windows platforms. It is a workaround for cases where
/// the SYS (`WinDivert64.sys`) driver files are locked by an already running kernel driver.
pub fn load_windivert(current_dir: &Path) -> Result<()> {
    #[cfg(all(target_os = "windows"))]
    {
        let windivert_dll_path = current_dir.join("WinDivert.dll");

        if !windivert_dll_path.exists() {
            use std::fs;

            let bytes: &'static [u8] = include_bytes!("../WinDivert.dll");
            fs::write(windivert_dll_path, bytes)?;
        }

        let windivert_driver_path = current_dir.join("WinDivert64.sys");

        if !windivert_driver_path.exists() {
            use std::fs;

            let bytes: &'static [u8] = include_bytes!("../WinDivert64.sys");
            fs::write(windivert_driver_path, bytes)?;
        }
    }

    Ok(())
}