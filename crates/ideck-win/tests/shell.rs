use ideck_core::ProfileColor;
use ideck_win::icons;

#[test]
fn profile_icon_has_all_sizes_uses_tokens_and_contains_only_our_own_shapes() {
    let bytes = icons::encode_profile_icon(ProfileColor::Teal).unwrap();
    let icon = ico::IconDir::read(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(
        icon.entries().iter().map(|e| e.width()).collect::<Vec<_>>(),
        vec![16, 24, 32, 48, 64, 128, 256]
    );
    let rgb = icons::color_rgb(ProfileColor::Teal).unwrap();
    for entry in icon.entries() {
        let image = entry.decode().unwrap();
        assert_eq!(image.width(), image.height());
        assert!(
            image
                .rgba_data()
                .chunks_exact(4)
                .any(|p| p[..3] == rgb && p[3] == 255)
        );
        assert!(image.rgba_data().chunks_exact(4).any(|p| p[3] == 0));
    }
    assert!(icons::profile_mark(ProfileColor::Teal, 8).is_err());
    assert_ne!(
        icons::encode_profile_icon(ProfileColor::Plum).unwrap(),
        icons::encode_profile_icon(ProfileColor::Teal).unwrap()
    );
}

#[cfg(windows)]
#[test]
fn shortcuts_persist_and_reload_identity_under_unicode_paths_and_preserve_existing_links() {
    use ideck_core::{Profile, shell::IdentityPlan};
    use ideck_win::shell::{ensure_icon, verify_shortcut, write_shortcut};
    use std::fs;
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Shortcut QA Zoë 日本語 with spaces");
    let mut profile = Profile::new(
        "claude-desktop",
        "Client — Zoë 日本語",
        &root.join("Profiles"),
        ProfileColor::Teal,
    )
    .unwrap();
    let icon = ensure_icon(&root.join("Manager"), &profile).unwrap();
    let launcher = root.join("Program Files/InstanceDeck.exe");
    let plan = IdentityPlan::new(
        &profile,
        &launcher.to_string_lossy(),
        &icon.to_string_lossy(),
    )
    .unwrap();
    let shortcut = root.join("Fixture desktop").join(plan.shortcut_filename());
    write_shortcut(&shortcut, &plan).unwrap();
    verify_shortcut(&shortcut, &plan).unwrap();
    let prior = fs::read(&shortcut).unwrap(); // Only our generated shortcut, never a target binary.
    profile.name = "Renamed fixture".into();
    let changed = IdentityPlan::new(&profile, &plan.launcher, &plan.icon).unwrap();
    write_shortcut(&shortcut, &changed).unwrap();
    verify_shortcut(&shortcut, &changed).unwrap();
    let backups: Vec<_> = fs::read_dir(shortcut.parent().unwrap())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p != &shortcut)
        .collect();
    assert_eq!(backups.len(), 1);
    assert_eq!(fs::read(&backups[0]).unwrap(), prior);
    assert!(!launcher.exists()); // Nothing was launched or copied into this fake executable path.
}
