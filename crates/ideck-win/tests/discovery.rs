use ideck_win::discovery::versioned_executable;

#[test]
fn legacy_discovery_chooses_the_highest_numeric_version_without_reading_binaries() {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Zoë 日本語 with spaces");
    assert!(
        versioned_executable(&root.join("app.exe"), "app-")
            .unwrap()
            .is_none()
    );
    for version in [
        "app-1.9.0",
        "app-1.10.0",
        "app-0.99.0",
        "app-incomplete",
        "app-2.0.0",
    ] {
        let directory = root.join(version);
        std::fs::create_dir_all(&directory).unwrap();
        if version != "app-2.0.0" {
            std::fs::write(
                directory.join("app.exe"),
                "not a real binary; metadata fixture only",
            )
            .unwrap();
        }
    }
    let (path, version) = versioned_executable(&root.join("app.exe"), "app-")
        .unwrap()
        .unwrap();
    assert_eq!(version, "1.10.0");
    assert_eq!(path, root.join("app-1.10.0/app.exe"));
}
