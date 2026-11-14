#![cfg(windows)]
use ideck_core::{DesktopOperations, Profile, ProfileColor};
use ideck_win::WindowsPlatform;
use std::{fs, path::Path};
use tempfile::TempDir;

#[test]
fn recycle_moves_only_a_manager_shaped_fixture_directory() {
    let scratch = TempDir::new().unwrap();
    let root = scratch.path().join("Profiles Zoë 日本語 with spaces");
    let profile = Profile::new("fixture", "Disposable fixture", &root, ProfileColor::Teal).unwrap();
    let path = Path::new(&profile.data_dir);
    fs::create_dir_all(path).unwrap();
    fs::write(
        path.join("fixture.txt"),
        b"InstanceDeck recycle-only test fixture",
    )
    .unwrap();
    let platform = WindowsPlatform::new(scratch.path().join("manager"));
    platform.recycle_profile(&profile).unwrap();
    assert!(!path.exists());
    assert!(root.join("fixture").is_dir());
    // This test's fixture remains recoverable in the Windows Recycle Bin.
}

#[test]
fn recycle_rejects_an_arbitrary_target_and_preserves_it() {
    let scratch = TempDir::new().unwrap();
    let mut profile =
        Profile::new("fixture", "Fixture", scratch.path(), ProfileColor::Teal).unwrap();
    let unrelated = scratch.path().join("keep this folder 日本語");
    fs::create_dir_all(&unrelated).unwrap();
    profile.data_dir = unrelated.to_string_lossy().into_owned();
    let platform = WindowsPlatform::new(scratch.path().join("manager"));
    assert!(platform.recycle_profile(&profile).is_err());
    assert!(unrelated.is_dir());
}
