use std::path::PathBuf;
use std::{env, fs};

fn main() {
    embed_ip_ranges();

    if cfg!(debug_assertions) {
        println!("DEV BUILD");
        tauri_build::build();
    } else {
        let mut windows = tauri_build::WindowsAttributes::new();
        windows = windows.app_manifest(include_str!("app.manifest"));

        tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
            .expect("failed to run build script");
    };
}

/// Filter the AWS ip-ranges.json down to the regions we use and write it to
/// OUT_DIR so the runtime can `include_str!` an immutable copy.
fn embed_ip_ranges() {
    const SOURCE: &str = "build-data/ip-ranges.json";
    const KEEP: &[&str] = &["us-east-1", "us-west-2", "eu-central-1"];

    println!("cargo:rerun-if-changed={SOURCE}");

    let raw = fs::read_to_string(SOURCE).expect("failed to read ip-ranges.json");
    let parsed: serde_json::Value = serde_json::from_str(&raw).expect("invalid ip-ranges.json");

    let prefixes = parsed
        .get("prefixes")
        .and_then(|v| v.as_array())
        .expect("ip-ranges.json missing prefixes array");

    let filtered: Vec<&serde_json::Value> = prefixes
        .iter()
        .filter(|p| {
            p.get("region")
                .and_then(|r| r.as_str())
                .is_some_and(|r| KEEP.contains(&r))
        })
        .collect();

    let out = serde_json::json!({ "prefixes": filtered });
    let dest = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("ip-ranges.min.json");
    fs::write(&dest, serde_json::to_vec(&out).unwrap())
        .expect("failed to write filtered ip-ranges");
}
