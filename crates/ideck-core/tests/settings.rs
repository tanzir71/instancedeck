use ideck_core::settings::{Settings, SettingsStore};
use std::fs;

#[test]
fn settings_defaults_validation_and_backups_preserve_existing_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Zoë 日本語 with spaces");
    let store = SettingsStore::new(&root);
    let mut settings = store.read().unwrap();
    assert!(!settings.onboarding_completed);
    assert!(settings.allow_update_checks);
    settings.profiles_root = "D:\\Profiles Zoë 日本語".into();
    store.write(&settings).unwrap();
    settings.close_to_tray = false;
    store.write(&settings).unwrap();
    assert!(!store.read().unwrap().close_to_tray);
    assert_eq!(
        fs::read_dir(&root)
            .unwrap()
            .filter_map(std::result::Result::ok)
            .filter(|entry| entry
                .file_name()
                .to_string_lossy()
                .starts_with("settings.backup-"))
            .count(),
        1
    );
    let before = fs::read(root.join("settings.json")).unwrap();
    settings.profiles_root = "relative".into();
    assert!(store.write(&settings).is_err());
    assert_eq!(before, fs::read(root.join("settings.json")).unwrap());
    let mut future: serde_json::Value = serde_json::from_slice(&before).unwrap();
    future["schemaVersion"] = 99.into();
    fs::write(
        root.join("settings.json"),
        serde_json::to_vec(&future).unwrap(),
    )
    .unwrap();
    assert!(store.read().is_err());
    assert!(store.write(&Settings::defaults(&root)).is_err());
}
