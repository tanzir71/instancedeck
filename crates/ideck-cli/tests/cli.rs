#![cfg(windows)]
use serde_json::Value;
use std::{path::Path, process::Command};

fn invoke(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ideck"))
        .arg("--state-dir")
        .arg(root)
        .args(args)
        .output()
        .unwrap()
}
fn successful(root: &Path, args: &[&str]) -> Value {
    let result = invoke(root, args);
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
    serde_json::from_slice(&result.stdout).unwrap()
}

#[test]
fn cli_create_rename_list_and_remove_keep_profile_data() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Zoë 日本語 manager with spaces");
    let profile = successful(
        &root,
        &[
            "profiles",
            "create",
            "claude-desktop",
            "QA — 日本語",
            "--color",
            "iris",
        ],
    );
    let id = profile["id"].as_str().unwrap();
    let data = Path::new(profile["dataDir"].as_str().unwrap());
    std::fs::create_dir_all(data).unwrap();
    std::fs::write(data.join("keep-user-data.fixture"), "test fixture only").unwrap();
    let renamed = successful(&root, &["profiles", "rename", id, "A new name"]);
    assert_eq!(renamed["name"], "A new name");
    assert_eq!(renamed["dataDir"], profile["dataDir"]);
    let list = successful(&root, &["profiles", "list"]);
    assert_eq!(list["profiles"].as_array().unwrap().len(), 1);
    let removed = successful(&root, &["profiles", "remove", id]);
    assert_eq!(removed["removedFromDeck"], id);
    assert!(data.join("keep-user-data.fixture").exists());
    assert!(
        successful(&root, &["profiles", "list"])["profiles"]
            .as_array()
            .unwrap()
            .is_empty()
    );
}

#[test]
fn cli_errors_fail_without_mutating_metadata_or_launching() {
    let temp = tempfile::tempdir().unwrap();
    for args in [
        vec!["profiles", "create", "unknown-app", "Name"],
        vec![
            "profiles",
            "create",
            "codex",
            "Name",
            "--color",
            "vendor-logo",
        ],
        vec!["launch", "missing"],
        vec!["close", "missing"],
        vec!["plan", "missing"],
    ] {
        assert!(!invoke(temp.path(), &args).status.success());
    }
    assert!(!temp.path().join("profiles.json").exists());
}

#[test]
fn doctor_redacts_user_paths_and_exposes_only_structured_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let report = successful(temp.path(), &["doctor"]);
    let text = serde_json::to_string(&report).unwrap();
    assert_eq!(report["directories"]["userProfile"], "%USERPROFILE%");
    assert!(report["apps"].is_array());
    assert!(report["mainInstances"].is_array());
    assert!(!text.contains("auth.json"));
    assert!(!text.contains("Cookies"));
    assert!(report["profiles"].as_array().unwrap().is_empty());
}
