mod common;
use common::*;
use ideck_core::{process::ProcessInfo, *};

#[test]
fn resolved_plans_isolate_claude_and_optional_codex_cli() {
    let (_temp, engine, mut profile) = fixture();
    let plan = engine.plan(&profile, false).unwrap();
    assert!(plan.env["LOCALAPPDATA"].ends_with("\\localappdata"));
    assert!(plan.env["CLAUDE_CONFIG_DIR"].ends_with("\\claude-code"));
    assert!(plan.args[0].contains("Profiles with spaces\\Zoë"));
    assert_eq!(plan.directories.len(), 4);
    assert_eq!(engine.recipes().len(), 2);
    profile.options.isolate_claude_code = false;
    assert!(
        !engine
            .plan(&profile, false)
            .unwrap()
            .env
            .contains_key("CLAUDE_CONFIG_DIR")
    );
    profile.app_id = "codex".into();
    let plan = engine.plan(&profile, false).unwrap();
    assert!(plan.env["CODEX_HOME"].ends_with("\\codex-home"));
    assert!(plan.env["CODEX_ELECTRON_USER_DATA_PATH"].ends_with("\\electron"));
    assert!(!plan.env.contains_key("CODEX_CLI_PATH"));
    engine
        .platform
        .0
        .lock()
        .unwrap()
        .existing_paths
        .insert(r"C:\Apps\official\app\resources\codex.exe".into());
    assert!(
        engine
            .plan(&profile, false)
            .unwrap()
            .env
            .contains_key("CODEX_CLI_PATH")
    );
    profile.app_id = "missing".into();
    assert!(engine.plan(&profile, false).is_err());
}

#[test]
fn engine_validates_catalog_and_prevents_escaping_profile_paths() {
    let temp = tempfile::tempdir().unwrap();
    let recipes = built_in_recipes().unwrap();
    assert!(
        Engine::new(
            FakePlatform::default(),
            ProfileStore::new(temp.path()),
            vec![recipes[0].clone(), recipes[0].clone()]
        )
        .is_err()
    );
    let mut bad = recipes[0].clone();
    bad.id = "bad/id".into();
    assert!(
        Engine::new(
            FakePlatform::default(),
            ProfileStore::new(temp.path()),
            vec![bad]
        )
        .is_err()
    );
    let mut escaped = recipes[0].clone();
    escaped
        .launch
        .env
        .insert("DATA".into(), "{profile}\\..\\outside".into());
    let engine = Engine::new(
        FakePlatform::default(),
        ProfileStore::new(temp.path()),
        vec![escaped],
    )
    .unwrap();
    assert!(engine.plan(&profile("claude-desktop"), false).is_err());
    assert!(engine.launch("missing").is_err());
    assert!(engine.close("missing").is_err());
}

#[test]
fn discovery_is_reported_honestly_without_launching() {
    let (_temp, engine, _) = fixture();
    assert!(engine.apps(false).iter().all(|app| app.installed.is_some()));
    engine.platform.0.lock().unwrap().missing_app = true;
    assert!(
        engine
            .apps(true)
            .iter()
            .all(|app| app.installed.is_none() && app.error.is_some())
    );
    assert_eq!(engine.platform.0.lock().unwrap().refreshes, 2);
}

#[test]
fn launch_confirms_a_matching_process_and_focuses_existing_instances() {
    let (_temp, engine, profile) = fixture();
    let result = engine.launch(&profile.id).unwrap();
    assert!(!result.already_running);
    assert_eq!(result.strategy, Some(Strategy::Direct));
    assert_eq!(result.pid, 77);
    assert!(
        engine
            .store
            .read()
            .unwrap()
            .profile(&profile.id)
            .unwrap()
            .last_launched_at
            .is_some()
    );
    let again = engine.launch(&profile.id).unwrap();
    assert!(again.already_running);
    assert_eq!(again.strategy, None);
    let state = engine.platform.0.lock().unwrap();
    assert_eq!(state.attempts, vec![Strategy::Direct]);
    assert_eq!(state.events.last(), Some(&"focus"));
    assert_eq!(state.directories.len(), 4);
}

#[test]
fn launch_re_resolves_updates_once_then_uses_the_package_fallback() {
    let (_temp, engine, profile) = fixture();
    engine.platform.0.lock().unwrap().fail_direct = true;
    assert_eq!(
        engine.launch(&profile.id).unwrap().strategy,
        Some(Strategy::PackageContext)
    );
    let state = engine.platform.0.lock().unwrap();
    assert_eq!(
        state.attempts,
        vec![Strategy::Direct, Strategy::PackageContext]
    );
    assert_eq!(state.refreshes, 1);
    drop(state);
    let (_temp2, updated, profile2) = fixture();
    {
        let mut state = updated.platform.0.lock().unwrap();
        state.fail_direct = true;
        state.update_exe = true;
    }
    assert_eq!(
        updated.launch(&profile2.id).unwrap().strategy,
        Some(Strategy::Direct)
    );
    assert_eq!(
        updated.platform.0.lock().unwrap().attempts,
        vec![Strategy::Direct, Strategy::Direct]
    );
}

#[test]
fn launch_failures_and_tracking_timeouts_are_bounded_and_never_kill() {
    let (_temp, engine, profile) = fixture();
    {
        let mut state = engine.platform.0.lock().unwrap();
        state.fail_direct = true;
        state.fail_package = true;
    }
    let error = engine.launch(&profile.id).unwrap_err().to_string();
    assert!(error.contains("direct, package-context"));
    {
        let mut state = engine.platform.0.lock().unwrap();
        state.fail_direct = false;
        state.fail_package = false;
        state.no_process = true;
    }
    assert!(
        engine
            .launch(&profile.id)
            .unwrap_err()
            .to_string()
            .contains("20 seconds")
    );
    let state = engine.platform.0.lock().unwrap();
    assert_eq!(state.waits, 78);
    assert!(!state.events.contains(&"force-close"));
}

#[test]
fn graceful_close_preserves_other_profiles_and_never_forces_by_default() {
    let (_temp, engine, profile) = fixture();
    engine.launch(&profile.id).unwrap();
    let unrelated = ProcessInfo {
        pid: 99,
        parent_pid: None,
        executable: r"C:\Apps\official\app\claude.exe".into(),
        args: vec!["--user-data-dir=D:\\another\\data".into()],
        started_at: 101,
    };
    engine.platform.0.lock().unwrap().processes.push(unrelated);
    engine.close(&profile.id).unwrap();
    let state = engine.platform.0.lock().unwrap();
    assert_eq!(state.processes.len(), 1);
    assert_eq!(state.processes[0].pid, 99);
    assert!(!state.events.contains(&"force-close"));
    drop(state);
    engine.close(&profile.id).unwrap();
}

#[test]
fn force_requires_confirmation_after_eight_seconds_of_graceful_close() {
    let (_temp, engine, profile) = fixture();
    engine.launch(&profile.id).unwrap();
    engine.platform.0.lock().unwrap().stubborn = true;
    assert!(matches!(
        engine.close(&profile.id),
        Err(Error::StillRunning)
    ));
    assert_eq!(engine.platform.0.lock().unwrap().waits, 16);
    assert!(engine.force_close(&profile.id, false).is_err());
    engine.force_close(&profile.id, true).unwrap();
    let state = engine.platform.0.lock().unwrap();
    assert_eq!(state.waits, 32);
    assert_eq!(state.events.last(), Some(&"force-close"));
    assert_eq!(state.events[state.events.len() - 2], "graceful-close");
    assert!(state.processes.is_empty());
    drop(state);
    engine.force_close(&profile.id, true).unwrap();
}

#[test]
fn reused_or_unprovable_process_identity_does_not_grant_close_authority() {
    let (_temp, engine, profile) = fixture();
    engine.launch(&profile.id).unwrap();
    {
        let mut state = engine.platform.0.lock().unwrap();
        state.stubborn = true;
        state.reused_pid_on_close = true;
    }
    engine.force_close(&profile.id, true).unwrap();
    assert!(
        !engine
            .platform
            .0
            .lock()
            .unwrap()
            .events
            .contains(&"force-close")
    );
    engine.platform.0.lock().unwrap().processes[0].started_at = 0;
    assert!(engine.close(&profile.id).is_err());
}

#[test]
fn main_instance_tracking_excludes_every_managed_descendant() {
    let (_temp, engine, profile) = fixture();
    engine.launch(&profile.id).unwrap();
    let executable = app(&engine.recipes()[0], false).executable;
    {
        let mut state = engine.platform.0.lock().unwrap();
        state.processes.extend([
            ProcessInfo {
                pid: 78,
                parent_pid: Some(77),
                executable: executable.clone(),
                args: vec![],
                started_at: 101,
            },
            ProcessInfo {
                pid: 90,
                parent_pid: None,
                executable: executable.clone(),
                args: vec![],
                started_at: 102,
            },
            ProcessInfo {
                pid: 91,
                parent_pid: Some(90),
                executable,
                args: vec!["--type=renderer".into()],
                started_at: 103,
            },
        ]);
    }
    let main = engine.main_processes("claude-desktop").unwrap();
    assert_eq!(main.iter().map(|p| p.pid).collect::<Vec<_>>(), vec![90, 91]);
    engine.close_main("claude-desktop").unwrap();
    assert_eq!(engine.running(&profile).unwrap().len(), 2);
}

#[test]
fn force_close_does_not_report_success_until_the_profile_has_exited() {
    let (_temp, engine, profile) = fixture();
    engine.launch(&profile.id).unwrap();
    {
        let mut state = engine.platform.0.lock().unwrap();
        state.stubborn = true;
        state.fail_force = true;
    }
    assert!(matches!(
        engine.force_close(&profile.id, true),
        Err(Error::StillRunning)
    ));
    assert_eq!(engine.platform.0.lock().unwrap().waits, 32);
}
