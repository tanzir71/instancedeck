#![cfg(windows)]
use ideck_core::signin::SignInFiles;
use ideck_win::WindowsPlatform;
use std::{
    fs,
    time::{Duration, Instant},
};

#[test]
fn opaque_windows_transfer_and_no_replace_rename_preserve_synthetic_unicode_fixtures() {
    let temp = tempfile::tempdir().unwrap();
    let source = temp.path().join("Zoë 日本語 fixture source");
    let target = temp.path().join("Other drive layout/profile/data");
    fs::create_dir_all(source.join("Network")).unwrap();
    fs::write(source.join("Preferences"), b"only synthetic fixture bytes").unwrap();
    fs::write(source.join("Network/fixture.bin"), [0, 255, 128]).unwrap();
    let platform = WindowsPlatform::new(temp.path().join("manager"));
    assert!(platform.tree_bytes(&source).unwrap() > 0);
    assert!(platform.free_bytes(&source).unwrap() > 0);
    platform.copy_data(&source, &target).unwrap();
    assert!(source.exists());
    assert_eq!(
        fs::read(target.join("Network/fixture.bin")).unwrap(),
        [0, 255, 128]
    );
    assert!(platform.copy_data(&source, &target).is_err());
    let renamed = target.with_file_name("retained fixture");
    platform.rename_data(&target, &renamed).unwrap();
    assert!(platform.rename_data(&source, &renamed).is_err());
    assert!(source.exists());
    assert!(renamed.exists());
}

#[test]
fn windows_metadata_watcher_enables_only_a_quiet_hint_and_resets_for_a_new_journal() {
    let temp = tempfile::tempdir().unwrap();
    let slot = temp.path().join("Fresh fixture Zoë 日本語");
    fs::create_dir_all(slot.join("Network")).unwrap();
    let platform = WindowsPlatform::new(temp.path().join("manager"));
    assert!(
        !platform
            .sign_in_hint(&slot, "first synthetic journal")
            .unwrap()
    );
    fs::write(slot.join("config.json"), b"synthetic fixture only").unwrap();
    std::thread::sleep(Duration::from_millis(100));
    fs::write(slot.join("Network/Cookies-wal"), b"synthetic fixture only").unwrap();
    let started = Instant::now();
    while !platform
        .sign_in_hint(&slot, "first synthetic journal")
        .unwrap()
    {
        assert!(started.elapsed() < Duration::from_secs(7));
        std::thread::sleep(Duration::from_millis(100));
    }
    assert!(started.elapsed() >= Duration::from_secs(3));
    assert!(
        !platform
            .sign_in_hint(&slot, "second synthetic journal")
            .unwrap()
    );
}
