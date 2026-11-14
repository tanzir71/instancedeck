mod common;
use ideck_core::{
    layout, paths,
    process::{self, ProcessInfo},
    template::{self, Context},
    *,
};
use std::{fs, path::Path};

fn context() -> Context {
    Context {
        profile: r"D:\Profiles with spaces\Zoë\01".into(),
        install_dir: r"C:\Apps\Official".into(),
        exe_dir: r"C:\Apps\Official\app".into(),
        user_profile: r"C:\Users\Zoë Example".into(),
        local_app_data: r"C:\Users\Zoë Example\AppData\Local".into(),
    }
}

#[test]
fn templates_are_exact_nonrecursive_and_unicode_safe() {
    let context = context();
    assert_eq!(
        template::expand("--user-data-dir={profile}", &context).unwrap(),
        format!("--user-data-dir={}", context.profile)
    );
    assert_eq!(
        template::expand(
            "{installDir}|{exeDir}|{userProfile}|{localAppData}",
            &context
        )
        .unwrap(),
        format!(
            "{}|{}|{}|{}",
            context.install_dir, context.exe_dir, context.user_profile, context.local_app_data
        )
    );
    for bad in [
        "{unknown}",
        "{profile",
        "profile}",
        "}x{profile}",
        "\n",
        "\0",
        "\r",
    ] {
        assert!(template::expand(bad, &context).is_err(), "{bad:?}");
    }
    let mut nested = context.clone();
    nested.profile = "{exeDir}".into();
    assert_eq!(template::expand("{profile}", &nested).unwrap(), "{exeDir}");
    nested.profile = "bad\npath".into();
    assert!(template::expand("{profile}", &nested).is_err());
}

#[test]
fn windows_paths_use_component_boundaries_and_normalize_extended_prefixes() {
    for equivalent in [
        r"C:\Users\Zoë Example\Profiles\one",
        r"c:/users/zoë example/Profiles/./one/",
        r"\\?\C:\Users\Zoë Example\Profiles\two\..\one",
    ] {
        assert_eq!(
            paths::normalize_windows(equivalent),
            "c:\\users\\zoë example\\profiles\\one"
        );
    }
    assert_eq!(
        paths::normalize_windows(r"\\?\UNC\server\share\a"),
        r"\\server\share\a"
    );
    assert!(paths::is_within_windows(
        r"D:\profiles\one\data",
        r"D:\Profiles\one"
    ));
    assert!(!paths::is_within_windows(
        r"D:\profiles\one-more\data",
        r"D:\Profiles\one"
    ));
    assert!(!paths::is_within_windows(
        r"D:\profiles\one\..\other",
        r"D:\Profiles\one"
    ));
    for valid in [r"C:\one", r"\\server\share\日本語", "D:/profiles/one"] {
        assert!(paths::validate_absolute(valid).is_ok());
    }
    for invalid in ["relative", "C:relative", "C:\\bad\npath", "", "\0"] {
        assert!(paths::validate_absolute(invalid).is_err());
    }
    assert_eq!(
        paths::redact(
            r"Path C:\Users\Zoë Example\p and C:\Users\Zoë Example\q",
            r"C:\Users\Zoë Example"
        ),
        r"Path %USERPROFILE%\p and %USERPROFILE%\q"
    );
    assert_eq!(paths::redact("plain", ""), "plain");
}

#[test]
fn recipes_roundtrip_generate_schema_and_reject_unsafe_shapes() {
    let recipes = built_in_recipes().unwrap();
    assert_eq!(recipes.len(), 2);
    let schema: serde_json::Value = serde_json::from_str(&recipe_schema().unwrap()).unwrap();
    assert!(schema["properties"]["recipeVersion"].is_object());
    for recipe in &recipes {
        assert_eq!(
            Recipe::from_json(&serde_json::to_string(recipe).unwrap())
                .unwrap()
                .id,
            recipe.id
        );
    }
    let baseline = serde_json::to_value(&recipes[0]).unwrap();
    for (pointer, value) in [
        ("/id", serde_json::json!("../escape")),
        ("/recipeVersion", serde_json::json!(0)),
        ("/name", serde_json::json!(" ")),
        ("/seo/slug", serde_json::json!("unsafe/slug")),
        ("/discovery", serde_json::json!([])),
        ("/launch/strategies", serde_json::json!(["package-context"])),
        (
            "/launch/strategies",
            serde_json::json!(["direct", "direct"]),
        ),
        ("/processMatch/exeName", serde_json::json!("path\\app.exe")),
        ("/processMatch/exeName", serde_json::json!("app.dll")),
        ("/processMatch/arg", serde_json::json!("--no-profile")),
        ("/launch/args", serde_json::json!(["--x={unknown}"])),
        ("/launch/env", serde_json::json!({"BAD=KEY":"x"})),
        ("/launch/env", serde_json::json!({"1KEY":"x"})),
        ("/launch/env", serde_json::json!({"Key":"x", "KEY":"y"})),
        ("/launch/env", serde_json::json!({"":"x"})),
        ("/discovery/0/exe", serde_json::json!("..\\escape.exe")),
        ("/discovery/0/exe", serde_json::json!("C:\\app.exe")),
        ("/discovery/0/packageFamily", serde_json::json!("")),
    ] {
        let mut invalid = baseline.clone();
        *invalid.pointer_mut(pointer).unwrap() = value;
        assert!(
            Recipe::from_json(&invalid.to_string()).is_err(),
            "{pointer}"
        );
    }
    let mut unknown = baseline;
    unknown["secret"] = serde_json::json!(true);
    assert!(Recipe::from_json(&unknown.to_string()).is_err());
    assert!(!recipe::valid_id("-edge"));
    assert!(!recipe::valid_id("edge-"));
    assert!(!recipe::valid_id(""));
    assert_eq!(Strategy::Direct.to_string(), "direct");
    assert_eq!(Strategy::PackageContext.to_string(), "package-context");
}

#[test]
fn registry_and_known_paths_are_validated_without_reading_apps() {
    let mut recipe = built_in_recipes().unwrap().remove(0);
    recipe.discovery = vec![
        Discovery::Registry {
            display_name: "Fixture".into(),
            exe: "app.exe".into(),
        },
        Discovery::KnownPath {
            path: "{localAppData}\\Fixture\\app.exe".into(),
            version_prefix: None,
        },
    ];
    recipe.validate().unwrap();
    recipe.discovery = vec![Discovery::Registry {
        display_name: " ".into(),
        exe: "app.exe".into(),
    }];
    assert!(recipe.validate().is_err());
    recipe.discovery = vec![Discovery::Registry {
        display_name: "Fixture".into(),
        exe: "../app.exe".into(),
    }];
    assert!(recipe.validate().is_err());
    recipe.discovery = vec![Discovery::KnownPath {
        path: "{unsupported}\\app.exe".into(),
        version_prefix: None,
    }];
    assert!(recipe.validate().is_err());
}

#[test]
fn profile_store_retains_backups_and_data_across_rename_and_remove() {
    let temp = tempfile::tempdir().unwrap();
    let store = ProfileStore::new(temp.path());
    assert!(store.read().unwrap().profiles.is_empty());
    let profile = store.create(common::profile("claude-desktop")).unwrap();
    let original_path = profile.data_dir.clone();
    // A sentinel represents user data; remove-from-deck may not touch it.
    let sentinel = temp.path().join("user-data-kept");
    fs::write(&sentinel, "fixture").unwrap();
    store.rename(&profile.id, "Renamed — client").unwrap();
    assert_eq!(
        store.read().unwrap().profile(&profile.id).unwrap().data_dir,
        original_path
    );
    store.mark_launched(&profile.id).unwrap();
    assert!(
        store
            .read()
            .unwrap()
            .profile(&profile.id)
            .unwrap()
            .last_launched_at
            .is_some()
    );
    assert_eq!(store.remove(&profile.id).unwrap().id, profile.id);
    assert!(sentinel.exists());
    assert!(store.read().unwrap().profiles.is_empty());
    let backups = fs::read_dir(temp.path())
        .unwrap()
        .flatten()
        .filter(|entry| {
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("profiles.backup-")
        })
        .count();
    assert_eq!(backups, 3);
    assert!(store.remove("missing").is_err());
    assert!(store.rename("missing", "Name").is_err());
    assert!(store.mark_launched("missing").is_err());
    assert_eq!(store.root(), temp.path());
}

#[test]
fn profile_validation_migrations_and_failed_transactions_preserve_metadata() {
    let temp = tempfile::tempdir().unwrap();
    let store = ProfileStore::new(temp.path());
    let profile = common::profile("codex");
    let mut old = serde_json::to_value(ProfileState {
        schema_version: 0,
        profiles: vec![profile.clone()],
    })
    .unwrap();
    old["profiles"][0]
        .as_object_mut()
        .unwrap()
        .remove("options");
    fs::write(temp.path().join("profiles.json"), old.to_string()).unwrap();
    let migrated = store.read().unwrap();
    assert_eq!(migrated.schema_version, 1);
    assert!(migrated.profiles[0].options.isolate_claude_code);
    store.rename(&profile.id, "Migrated").unwrap();
    let before = fs::read(temp.path().join("profiles.json")).unwrap();
    assert!(store.rename(&profile.id, "\n").is_err());
    assert_eq!(fs::read(temp.path().join("profiles.json")).unwrap(), before);
    assert!(store.create(profile.clone()).is_err());
    assert_eq!(fs::read(temp.path().join("profiles.json")).unwrap(), before);
    old["schemaVersion"] = serde_json::json!(999);
    assert!(ProfileState::from_json(&old.to_string()).is_err());
    for (field, value) in [
        ("id", "not-ulid"),
        ("appId", "../app"),
        ("name", ""),
        ("badge", "vendor-logo"),
        ("dataDir", "relative"),
    ] {
        let mut bad = profile.clone();
        match field {
            "id" => bad.id = value.into(),
            "appId" => bad.app_id = value.into(),
            "name" => bad.name = value.into(),
            "badge" => bad.badge = value.into(),
            _ => bad.data_dir = value.into(),
        }
        assert!(bad.validate().is_err());
    }
    assert!(
        Profile::new(
            "codex",
            &"x".repeat(121),
            Path::new(r"D:\Profiles"),
            ProfileColor::Sky
        )
        .is_err()
    );
    let mut duplicate = profile.clone();
    duplicate.id = ulid::Ulid::generate().to_string();
    assert!(
        ProfileState::from_json(
            &serde_json::to_string(&ProfileState {
                schema_version: 1,
                profiles: vec![profile.clone(), duplicate]
            })
            .unwrap()
        )
        .is_err()
    );
    fs::write(temp.path().join("profiles.json"), "bad json").unwrap();
    assert!(store.read().is_err());
    let blocked = tempfile::NamedTempFile::new().unwrap();
    assert!(ProfileStore::new(blocked.path()).create(profile).is_err());
}

#[test]
fn layout_probe_uses_only_names_and_refuses_relocation_and_ambiguity() {
    let temp = tempfile::tempdir().unwrap();
    let a = temp.path().join("Zoë with spaces");
    let b = temp.path().join("日本語");
    assert!(!layout::probe(&a).unwrap().live);
    fs::create_dir_all(&a).unwrap();
    fs::write(a.join("Network"), "fixture contents are irrelevant").unwrap();
    assert!(layout::probe(&a).unwrap().live);
    assert_eq!(layout::resolve(&[a.clone(), b.clone()]).unwrap(), a);
    fs::write(a.join(".user-data-left-old"), "").unwrap();
    let relocated = layout::probe(&a).unwrap();
    assert!(relocated.relocated);
    assert!(!relocated.live);
    assert!(layout::resolve(std::slice::from_ref(&a)).is_err());
    fs::create_dir_all(&b).unwrap();
    fs::write(b.join("Preferences"), "").unwrap();
    assert_eq!(layout::resolve(&[a.clone(), b.clone()]).unwrap(), b);
    fs::rename(a.join(".user-data-left-old"), a.join("marker-preserved")).unwrap();
    assert!(layout::resolve(&[a.clone(), b.clone()]).is_err());
    assert!(layout::probe(&b.join("Preferences")).is_err());
    assert_eq!(
        layout::claude_candidates(Path::new("local"), Path::new("roaming")).len(),
        3
    );
}

#[test]
fn process_fixtures_require_the_exact_profile_and_official_install_tree() {
    let recipe = built_in_recipes().unwrap().remove(0);
    let app = common::app(&recipe, false);
    let argument = r"--user-data-dir=D:\Profiles with spaces\Zoë\one\data";
    let mut process = ProcessInfo {
        pid: 10,
        parent_pid: None,
        executable: app.executable.clone(),
        args: vec![argument.into()],
        started_at: 1,
    };
    assert!(process::matches_profile(
        &process,
        &app,
        &recipe.process_match,
        argument
    ));
    process.args = vec![
        "--USER-DATA-DIR".into(),
        "\"d:/profiles with spaces/Zoë/one/data\"".into(),
    ];
    assert!(process::matches_profile(
        &process,
        &app,
        &recipe.process_match,
        argument
    ));
    process.args = vec![format!("{argument}-other")];
    assert!(!process::matches_profile(
        &process,
        &app,
        &recipe.process_match,
        argument
    ));
    process.args = vec![argument.into()];
    process.executable = r"C:\Apps\official-fake\app\claude.exe".into();
    assert!(!process::matches_profile(
        &process,
        &app,
        &recipe.process_match,
        argument
    ));
    process.executable = app.executable.clone();
    assert!(!process::matches_main(
        &process,
        &app,
        &recipe.process_match
    ));
    process.args.clear();
    assert!(process::matches_main(&process, &app, &recipe.process_match));
    process.args.push("--type=renderer".into());
    assert!(!process::matches_main(
        &process,
        &app,
        &recipe.process_match
    ));
    process.args = vec![r"D:\Profiles\bare".into()];
    assert!(process::matches_profile(
        &process,
        &app,
        &recipe.process_match,
        r"d:/profiles/bare"
    ));
    let child = ProcessInfo {
        pid: 11,
        parent_pid: Some(10),
        executable: r"C:\OtherTool\tool.exe".into(),
        args: vec![],
        started_at: 2,
    };
    let grandchild = ProcessInfo {
        pid: 12,
        parent_pid: Some(11),
        executable: r"C:\OtherTool\tool.exe".into(),
        args: vec![],
        started_at: 3,
    };
    let unrelated = ProcessInfo {
        pid: 13,
        parent_pid: None,
        executable: app.executable,
        args: vec![],
        started_at: 4,
    };
    assert_eq!(
        process::tree(&[grandchild, unrelated, child, process], &[10]).len(),
        3
    );
}

#[test]
fn process_trees_reject_reused_or_missing_parent_identities() {
    let root = ProcessInfo {
        pid: 10,
        parent_pid: None,
        executable: "root.exe".into(),
        args: vec![],
        started_at: 100,
    };
    let old_orphan = ProcessInfo {
        pid: 11,
        parent_pid: Some(10),
        executable: "old.exe".into(),
        args: vec![],
        started_at: 20,
    };
    let child = ProcessInfo {
        pid: 12,
        parent_pid: Some(10),
        executable: "child.exe".into(),
        args: vec![],
        started_at: 101,
    };
    assert_eq!(
        process::tree(&[root, old_orphan, child.clone()], &[10])
            .iter()
            .map(|p| p.pid)
            .collect::<Vec<_>>(),
        vec![10, 12]
    );
    assert!(process::tree(&[child], &[10]).is_empty());
}
