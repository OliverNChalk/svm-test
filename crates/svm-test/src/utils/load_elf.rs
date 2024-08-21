//! Modified from: <https://github.com/buffalojoec/mollusk>

use std::path::PathBuf;

use itertools::Itertools;

pub fn load_program_elf(program_name: &str) -> Vec<u8> {
    let file_name = format!("{program_name}.so");
    let file_path = find_file(&file_name);

    std::fs::read(&file_path)
        .unwrap_or_else(|err| panic!("Failed to read program file; path={file_path:?}; err={err}"))
}

fn find_file(file_name: &str) -> PathBuf {
    let dirs = default_shared_object_dirs();
    for dir in &dirs {
        let candidate = dir.join(file_name);
        if candidate.exists() {
            return candidate;
        }
    }

    panic!(
        "Failed to find file ({file_name}), directories checked:\n- {}",
        dirs.iter().map(|path| path.to_str().unwrap()).join("\n- ")
    )
}

fn default_shared_object_dirs() -> Vec<PathBuf> {
    [
        Some(PathBuf::from("tests/fixtures")),
        std::env::var("BPF_OUT_DIR").map(PathBuf::from).ok(),
        std::env::var("SBF_OUT_DIR").map(PathBuf::from).ok(),
        std::env::current_dir().ok(),
        super::locate_manifest()
            .and_then(|manifest| manifest.parent().map(|dir| dir.join("target/deploy"))),
    ]
    .into_iter()
    .flatten()
    .collect()
}
