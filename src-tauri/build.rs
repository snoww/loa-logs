use std::{env, fs::{read_dir, DirEntry, File}, path::PathBuf};

use tar::Builder;

/// Returns all entries in `dir` sorted lexicographically by filename.
fn sorted_dir_entries(dir: &str) -> Vec<DirEntry> {
    let mut entries: Vec<_> = read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .collect();

    entries.sort_by_key(|e| e.file_name());
    entries
}

/// Compresses migrations into a `.tar` archive inside the target dir.
/// Ensures migrations are added in lexicographic order for deterministic builds.
fn compress_migrations() {
    let target_dir = get_target_dir();
    let output_archive_path = target_dir.join("migrations.tar");

    let migrations = File::create(output_archive_path).unwrap();
    let mut builder = Builder::new(migrations);

    for entry in sorted_dir_entries("migrations") {
        let path = entry.path().canonicalize().unwrap();
        let mut file = File::open(&path).unwrap();

        println!("cargo:rerun-if-changed={}", path.display());
        builder
            .append_file(
                PathBuf::from(path.file_name().unwrap()),
                &mut file,
            )
            .unwrap();
    }
}

/// Returns the `target` directory for the current build.
///
/// In a Tauri build script, `OUT_DIR` points to a deep subfolder
/// like `target/debug/build/<crate>/out`. This function climbs
/// up the folder hierarchy to reach `target/debug` or `target/release`,
/// which is typically where build artifacts should be placed.
pub fn get_target_dir() -> PathBuf {
    let out_dir = PathBuf::from(env::var("OUT_DIR").expect("OUT_DIR must be set"));

    // climb up three levels: `.../build/<crate>/out` → `target/debug` or `target/release`
    out_dir
        .parent().expect("OUT_DIR parent missing")
        .parent().expect("OUT_DIR grandparent missing")
        .parent().expect("OUT_DIR great-grandparent missing")
        .to_path_buf()
}

fn main() {

    compress_migrations();

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
