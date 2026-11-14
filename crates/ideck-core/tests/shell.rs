use ideck_core::{shell::*, *};
use std::path::Path;

#[test]
fn identity_waits_for_a_stable_nonzero_window_and_rejects_reused_process_identity() {
    let mut windows = StableWindows::default();
    let candidate = WindowCandidate {
        handle: 42,
        pid: 77,
        process_started_at: 100,
    };
    assert!(windows.observe(0, &[candidate]).is_empty());
    assert!(windows.observe(749, &[candidate]).is_empty());
    assert_eq!(windows.observe(750, &[candidate]), vec![candidate]);
    let reused = WindowCandidate {
        process_started_at: 101,
        ..candidate
    };
    assert!(windows.observe(800, &[reused]).is_empty());
    assert_eq!(windows.observe(1550, &[reused]), vec![reused]);
    windows.observe(1600, &[]);
    assert!(windows.observe(1700, &[reused]).is_empty());
    assert!(
        windows
            .observe(
                3000,
                &[
                    WindowCandidate {
                        handle: 0,
                        ..reused
                    },
                    WindowCandidate {
                        process_started_at: 0,
                        ..reused
                    }
                ]
            )
            .is_empty()
    );
}

#[test]
fn shortcut_identity_survives_renames_and_unicode_without_target_binary_paths() {
    let mut profile = Profile::new(
        "claude-desktop",
        "Client — Zoë 日本語 / QA",
        Path::new(r"D:\Profiles with spaces"),
        ProfileColor::Teal,
    )
    .unwrap();
    let plan = IdentityPlan::new(
        &profile,
        r"C:\Program Files\InstanceDeck\InstanceDeck.exe",
        r"C:\Users\Zoë\icons\fixture.ico",
    )
    .unwrap();
    assert_eq!(
        plan.app_user_model_id,
        format!("InstanceDeck.claude-desktop.{}", profile.id)
    );
    assert_eq!(plan.args, vec!["launch", profile.id.as_str()]);
    assert!(!plan.shortcut_filename().contains('/'));
    assert!(
        plan.shortcut_filename()
            .ends_with(&format!("{}.lnk", profile.id))
    );
    profile.name = "Renamed".into();
    let renamed = IdentityPlan::new(&profile, &plan.launcher, &plan.icon).unwrap();
    assert_eq!(renamed.app_user_model_id, plan.app_user_model_id);
    assert_eq!(renamed.args, plan.args);
    assert!(IdentityPlan::new(&profile, "relative.exe", &plan.icon).is_err());
}

#[test]
fn global_hotkeys_normalize_modifiers_and_reject_ambiguous_or_conflicting_bindings() {
    assert_eq!(normalize_hotkey("alt + control + a").unwrap(), "Ctrl+Alt+A");
    assert_eq!(
        normalize_hotkey("win+shift+F24").unwrap(),
        "Shift+Super+F24"
    );
    assert_eq!(normalize_hotkey("Ctrl+`").unwrap(), "Ctrl+Backquote");
    for bad in [
        "",
        "A",
        "Shift+A",
        "Ctrl",
        "Ctrl+Ctrl+A",
        "Ctrl+Alt+F25",
        "Ctrl+A+B",
        "Ctrl+Enter",
        "Ctrl+Alt+",
        "Ctrl+日本語",
        "Ctrl+\nA",
    ] {
        assert!(normalize_hotkey(bad).is_err(), "{bad}");
    }
    let settings = Settings::defaults(Path::new(r"C:\Manager"));
    let mut profile = Profile::new(
        "codex",
        "Fixture",
        Path::new(r"D:\Profiles"),
        ProfileColor::Teal,
    )
    .unwrap();
    profile.hotkey = Some("alt+ctrl+space".into());
    assert!(validate_hotkeys(&settings, &[profile.clone()]).is_err());
    profile.hotkey = Some("Ctrl+Alt+1".into());
    validate_hotkeys(&settings, &[profile.clone()]).unwrap();
    let mut other = profile.clone();
    other.id = ulid::Ulid::generate().to_string();
    assert!(validate_hotkeys(&settings, &[profile, other]).is_err());
}
