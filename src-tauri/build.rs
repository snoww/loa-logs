use std::fs;

fn main() {
    if cfg!(debug_assertions) {
        println!("DEV BUILD");
        tauri_build::build();
    } else {
        let mut windows = tauri_build::WindowsAttributes::new();

        windows = windows.app_manifest(fs::read_to_string("windows-app.manifest").unwrap());

        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run build script");
    };
}
