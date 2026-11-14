mod common;
use common::*;
use ideck_core::{custom::CustomApp, history::Kind, *};
use std::{collections::BTreeMap, path::Path};

fn custom(environment_only: bool) -> CustomApp {
    CustomApp {
        id: "custom-editor".into(),
        name: "Editor 日本語".into(),
        executable: r"C:\Official apps\Editor.exe".into(),
        args: if environment_only {
            vec![]
        } else {
            vec!["--data={profile}".into()]
        },
        env: if environment_only {
            BTreeMap::from([("APP_HOME".into(), "{profile}\\home".into())])
        } else {
            BTreeMap::new()
        },
        match_argument: if environment_only {
            ""
        } else {
            "--data={profile}"
        }
        .into(),
    }
}
#[test]
fn all_generic_recipes_resolve_unicode_data_paths_and_match_paired_arguments() {
    let root = tempfile::tempdir().unwrap();
    let engine = Engine::new(
        FakePlatform::default(),
        ProfileStore::new(root.path()),
        built_in_recipes().unwrap(),
    )
    .unwrap();
    for recipe in engine.recipes().into_iter().skip(2) {
        assert_eq!(recipe.status, RecipeStatus::Community);
        assert!(recipe.verified_on.is_empty());
        let profile = Profile::new(
            &recipe.id,
            "Zoë 日本語",
            Path::new(r"D:\Profiles Zoë 日本語 with spaces"),
            ProfileColor::Iris,
        )
        .unwrap();
        engine.store.create(profile.clone()).unwrap();
        let plan = engine.plan(&profile, false).unwrap();
        assert!(plan.args.iter().any(|a| a.contains(&profile.data_dir)));
        let p = process::ProcessInfo {
            pid: 20,
            parent_pid: None,
            executable: plan.app.executable.clone(),
            args: plan.args.clone(),
            started_at: 100,
        };
        assert!(process::matches_profile(
            &p,
            &plan.app,
            &recipe.process_match,
            &plan.match_argument
        ));
        assert!(!process::matches_main(&p, &plan.app, &recipe.process_match));
    }
}
#[test]
fn custom_recipes_persist_without_reading_app_files_and_environment_receipts_reject_pid_reuse() {
    let root = tempfile::tempdir().unwrap();
    let engine = Engine::new(
        FakePlatform::default(),
        ProfileStore::new(root.path()),
        built_in_recipes().unwrap(),
    )
    .unwrap();
    let input = custom(true);
    engine.save_custom(&input).unwrap();
    assert!(engine.save_custom(&input).is_err());
    assert_eq!(custom::load(root.path()).unwrap().len(), 1);
    let profile = Profile::new(
        &input.id,
        "Work",
        Path::new(r"D:\Profiles Zoë 日本語"),
        ProfileColor::Iris,
    )
    .unwrap();
    engine.store.create(profile.clone()).unwrap();
    engine.launch(&profile.id).unwrap();
    assert_eq!(engine.running(&profile).unwrap().len(), 1);
    engine.platform.0.lock().unwrap().processes[0].started_at += 1;
    assert!(engine.running(&profile).unwrap().is_empty());
    assert!(engine.launch_main(&input.id).is_err());
    let mut invalid = custom(false);
    invalid
        .env
        .insert("API_TOKEN".into(), "not-a-secret-fixture".into());
    assert!(invalid.recipe().is_err());
    invalid = custom(false);
    invalid.match_argument = "--wrong={profile}".into();
    assert!(invalid.recipe().is_err());
}
#[test]
fn metadata_import_preserves_existing_ids_resets_local_claims_and_rejects_overlaps() {
    let saved = Profile::new(
        "codex",
        "Saved",
        Path::new(r"D:\Profiles"),
        ProfileColor::Teal,
    )
    .unwrap();
    let mut imported = Profile::new(
        "codex",
        "Imported",
        Path::new(r"D:\Profiles"),
        ProfileColor::Iris,
    )
    .unwrap();
    imported.sign_in_state = SignInState::SignedIn;
    imported.options.shared_codex_history = true;
    let state = ProfileState {
        schema_version: 1,
        profiles: vec![saved.clone()],
    };
    let input = ProfileState {
        schema_version: 1,
        profiles: vec![saved.clone(), imported.clone()],
    };
    let merged = state
        .merge_import(&serde_json::to_string(&input).unwrap())
        .unwrap();
    assert_eq!(merged.profiles.len(), 2);
    assert_eq!(merged.profiles[1].sign_in_state, SignInState::Unknown);
    assert!(!merged.profiles[1].options.shared_codex_history);
    imported.data_dir = format!("{}\\codex\\{}", saved.data_dir, imported.id);
    let bad = ProfileState {
        schema_version: 1,
        profiles: vec![imported],
    };
    assert!(
        state
            .merge_import(&serde_json::to_string(&bad).unwrap())
            .is_err()
    );
    assert!(
        state
            .merge_import(r#"{"schemaVersion":22,"profiles":[]}"#)
            .is_err()
    );
}
#[test]
fn interrupted_sessions_sharing_can_resume_or_restore_without_merging_or_pruning() {
    for fail_at in [1, 2] {
        for restore in [false, true] {
            let root = tempfile::tempdir().unwrap();
            let engine = Engine::new(
                FakePlatform::default(),
                ProfileStore::new(root.path()),
                built_in_recipes().unwrap(),
            )
            .unwrap();
            let profile = Profile::new(
                "codex",
                "Work",
                Path::new(r"D:\Profiles Zoë 日本語"),
                ProfileColor::Iris,
            )
            .unwrap();
            engine.store.create(profile.clone()).unwrap();
            let sessions = Path::new(&profile.data_dir)
                .join("codex-home")
                .join("sessions");
            {
                let mut state = engine.platform.0.lock().unwrap();
                state
                    .history
                    .insert(sessions.to_string_lossy().into_owned(), Kind::Directory);
                state.fail_history_at = Some(fail_at);
            }
            assert!(history::configure(&engine, &profile.id, true).is_err());
            assert!(history::guard(&engine, &profile).is_err());
            assert!(engine.store.remove(&profile.id).is_err());
            engine.platform.0.lock().unwrap().fail_history_at = None;
            history::configure(&engine, &profile.id, !restore).unwrap();
            let saved = engine
                .store
                .read()
                .unwrap()
                .profile(&profile.id)
                .unwrap()
                .clone();
            assert_eq!(saved.options.shared_codex_history, !restore);
            if !restore {
                history::configure(&engine, &profile.id, false).unwrap();
            }
            assert_eq!(
                engine.platform.history_kind(&sessions).unwrap(),
                Kind::Directory
            );
            assert!(
                engine
                    .platform
                    .0
                    .lock()
                    .unwrap()
                    .history
                    .values()
                    .any(|v| matches!(v, Kind::Directory))
            );
            assert!(
                history::guard(
                    &engine,
                    &engine
                        .store
                        .read()
                        .unwrap()
                        .profile(&profile.id)
                        .unwrap()
                        .clone()
                )
                .is_ok()
            );
        }
    }
}

#[test]
fn interrupted_private_history_restoration_keeps_both_targets_and_can_finish() {
    for operation in [1, 2] {
        let root = tempfile::tempdir().unwrap();
        let engine = Engine::new(
            FakePlatform::default(),
            ProfileStore::new(root.path()),
            built_in_recipes().unwrap(),
        )
        .unwrap();
        let profile = Profile::new(
            "codex",
            "Work",
            Path::new(r"D:\Profiles Zoë 日本語"),
            ProfileColor::Iris,
        )
        .unwrap();
        engine.store.create(profile.clone()).unwrap();
        let sessions = Path::new(&profile.data_dir)
            .join("codex-home")
            .join("sessions");
        engine
            .platform
            .0
            .lock()
            .unwrap()
            .history
            .insert(sessions.to_string_lossy().into_owned(), Kind::Directory);
        history::configure(&engine, &profile.id, true).unwrap();
        {
            let mut state = engine.platform.0.lock().unwrap();
            state.fail_history_at = Some(state.history_ops + operation);
        }
        assert!(history::configure(&engine, &profile.id, false).is_err());
        assert!(engine.store.remove(&profile.id).is_err());
        engine.platform.0.lock().unwrap().fail_history_at = None;
        history::configure(&engine, &profile.id, false).unwrap();
        assert_eq!(
            engine.platform.history_kind(&sessions).unwrap(),
            Kind::Directory
        );
        assert!(
            engine
                .platform
                .0
                .lock()
                .unwrap()
                .history
                .values()
                .any(|v| matches!(v, Kind::Junction(_)))
        );
        assert!(
            !engine
                .store
                .read()
                .unwrap()
                .profile(&profile.id)
                .unwrap()
                .options
                .shared_codex_history
        );
    }
}
