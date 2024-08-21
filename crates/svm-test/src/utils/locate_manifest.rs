use std::path::PathBuf;

pub(crate) fn locate_manifest() -> Option<PathBuf> {
    let cargo = std::env::var("CARGO").unwrap_or("cargo".to_owned());
    let output = std::process::Command::new(cargo)
        .arg("locate-project")
        .arg("--workspace")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    let parsed: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let root = parsed["root"].as_str().unwrap();

    Some(PathBuf::from(root))
}
