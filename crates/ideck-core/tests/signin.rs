mod common;
use common::FakePlatform;
use ideck_core::{
    process::ProcessInfo,
    signin::{JournalStore, MetadataActivity, SignIn, SignInFiles, State},
    *,
};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::atomic::{AtomicIsize, Ordering},
    time::Duration,
};

struct TestPlatform {
    base: FakePlatform,
    dirs: Directories,
    slot: PathBuf,
    crash: AtomicIsize,
    space: u64,
    fail_copy: bool,
}
impl TestPlatform {
    fn checkpoint(&self) {
        let n = self.crash.fetch_sub(1, Ordering::SeqCst);
        if n == 0 {
            panic!("simulated process termination");
        }
    }
}
impl Platform for TestPlatform {
    fn directories(&self) -> Result<Directories> {
        Ok(self.dirs.clone())
    }
    fn discover(&self, r: &Recipe, refresh: bool) -> Result<InstalledApp> {
        self.base.discover(r, refresh)
    }
    fn exists(&self, path: &str) -> Result<bool> {
        Ok(Path::new(path).exists())
    }
    fn ensure_directory(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path).map_err(|e| error::io(path, e))
    }
    fn launch_lock(&self) -> Result<Box<dyn LaunchLock + '_>> {
        self.base.launch_lock()
    }
    fn launch(&self, plan: &LaunchPlan, strategy: Strategy) -> Result<u32> {
        if plan.profile_id == "main" && !self.slot.exists() {
            fixture_tree(&self.slot, b"fresh fixture account");
        }
        self.base.launch(plan, strategy)
    }
    fn processes(&self) -> Result<Vec<ProcessInfo>> {
        self.base.processes()
    }
    fn request_close(&self, p: &[ProcessInfo]) -> Result<()> {
        self.base.request_close(p)
    }
    fn force_close(&self, p: &[ProcessInfo]) -> Result<()> {
        self.base.force_close(p)
    }
    fn focus(&self, p: &[ProcessInfo]) -> Result<()> {
        self.base.focus(p)
    }
    fn wait(&self, d: Duration) {
        self.base.wait(d);
    }
}
fn bytes(path: &Path) -> u64 {
    let m = fs::symlink_metadata(path).unwrap();
    if m.is_file() {
        return m.len();
    }
    fs::read_dir(path)
        .unwrap()
        .map(|e| bytes(&e.unwrap().path()))
        .sum()
}
fn copy_fixture(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for e in fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.file_type().unwrap().is_dir() {
            copy_fixture(&e.path(), &to.join(e.file_name()));
        } else {
            fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}
impl SignInFiles for TestPlatform {
    fn tree_bytes(&self, p: &Path) -> Result<u64> {
        Ok(bytes(p))
    }
    fn free_bytes(&self, _: &Path) -> Result<u64> {
        Ok(self.space)
    }
    fn rename_data(&self, from: &Path, to: &Path) -> Result<()> {
        self.checkpoint();
        if to.exists() {
            return Err(Error::Invalid("fixture no-overwrite".into()));
        }
        fs::rename(from, to).map_err(|e| error::io(from, e))?;
        self.checkpoint();
        Ok(())
    }
    fn copy_data(&self, from: &Path, to: &Path) -> Result<()> {
        self.checkpoint();
        if from.is_file() {
            fs::create_dir_all(to.parent().unwrap()).unwrap();
            fs::copy(from, to).unwrap();
        } else {
            copy_fixture(from, to);
        }
        if self.fail_copy {
            return Err(Error::Platform("fixture interrupted adoption".into()));
        }
        self.checkpoint();
        Ok(())
    }
    fn sign_in_hint(&self, _: &Path, _: &str) -> Result<bool> {
        Ok(false)
    }
}
fn fixture_tree(root: &Path, value: &[u8]) {
    fs::create_dir_all(root.join("Network")).unwrap();
    fs::write(root.join("Preferences"), value).unwrap();
    fs::write(root.join("Network/fixture.bin"), [0, 1, 2, 255, 128]).unwrap();
    fs::create_dir_all(root.join("nested 日本語")).unwrap();
    fs::write(root.join("nested 日本語/fixture.txt"), value).unwrap();
}
// Hashes are only computed over synthetic fixtures; the production engine never hashes app data.
fn tree_hash(root: &Path) -> Vec<(PathBuf, String)> {
    fn walk(base: &Path, path: &Path, entries: &mut Vec<(PathBuf, String)>) {
        for e in fs::read_dir(path).unwrap() {
            let e = e.unwrap();
            if e.file_type().unwrap().is_dir() {
                walk(base, &e.path(), entries);
            } else {
                entries.push((
                    e.path().strip_prefix(base).unwrap().into(),
                    format!("{:x}", Sha256::digest(fs::read(e.path()).unwrap())),
                ));
            }
        }
    }
    let mut entries = Vec::new();
    walk(root, root, &mut entries);
    entries.sort();
    entries
}
fn fixture() -> (tempfile::TempDir, Engine<TestPlatform>, Profile) {
    let temp = tempfile::tempdir().unwrap();
    let root = temp.path().join("Zoë 日本語 with spaces");
    let local = root.join("Local");
    let roaming = root.join("Roaming");
    fs::create_dir_all(&roaming).unwrap();
    let slot = layout::claude_candidates(&local, &roaming)[0].clone();
    fixture_tree(&slot, b"original main fixture account");
    let dirs = Directories {
        local_app_data: local.to_string_lossy().into_owned(),
        app_data: roaming.to_string_lossy().into_owned(),
        user_profile: root.to_string_lossy().into_owned(),
    };
    let platform = TestPlatform {
        base: FakePlatform::default(),
        dirs,
        slot,
        crash: AtomicIsize::new(-1),
        space: u64::MAX,
        fail_copy: false,
    };
    let store = ProfileStore::new(root.join("Manager"));
    let profile = store
        .create(
            Profile::new(
                "claude-desktop",
                "Fixture profile",
                &root.join("Profiles"),
                ProfileColor::Teal,
            )
            .unwrap(),
        )
        .unwrap();
    fixture_tree(
        &Path::new(&profile.data_dir).join("data"),
        b"previous profile fixture",
    );
    (
        temp,
        Engine::new(platform, store, built_in_recipes().unwrap()).unwrap(),
        profile,
    )
}

#[test]
fn every_durable_transition_can_restore_the_main_fixture_byte_identically() {
    // Includes all three Adopting checkpoints and the terminal states.
    for stop_after in 0..=9 {
        let (_temp, engine, profile) = fixture();
        let original = tree_hash(&engine.platform.slot);
        let runner = SignIn::new(&engine);
        let mut j = runner.prepare(&profile.id, true).unwrap();
        for _ in 0..stop_after {
            if j.state == State::AwaitingSignIn {
                runner.advance(&j.id).unwrap();
                runner.mark_finished(&j.id).unwrap();
                j = JournalStore::new(engine.store.root()).read(&j.id).unwrap();
            } else {
                j = runner.advance(&j.id).unwrap();
            }
        }
        let restarted = SignIn::new(&engine);
        let restored = restarted.restore(&j.id).unwrap();
        assert!(restored.state.terminal(), "checkpoint {stop_after}");
        assert_eq!(
            tree_hash(&engine.platform.slot),
            original,
            "checkpoint {stop_after}"
        );
        assert!(!j.stash().exists());
        assert!(
            JournalStore::new(engine.store.root())
                .active()
                .unwrap()
                .is_empty()
        );
        assert!(
            !engine
                .platform
                .base
                .0
                .lock()
                .unwrap()
                .events
                .contains(&"force-close")
        );
    }
}

#[test]
fn crashes_before_and_after_each_filesystem_operation_restore_original_and_retain_data() {
    for crash_at in 0..14 {
        let (_temp, engine, profile) = fixture();
        let original = tree_hash(&engine.platform.slot);
        let runner = SignIn::new(&engine);
        let j = runner.prepare(&profile.id, true).unwrap();
        engine.platform.crash.store(crash_at, Ordering::SeqCst);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runner.resume(&j.id).unwrap();
            runner.finish(&j.id).unwrap();
        }));
        engine.platform.crash.store(-1, Ordering::SeqCst);
        let recovered = SignIn::new(&engine).restore(&j.id).unwrap();
        assert!(recovered.state.terminal());
        assert_eq!(
            tree_hash(&engine.platform.slot),
            original,
            "operation {crash_at}"
        );
        assert!(Path::new(&profile.data_dir).join("data").exists());
    }
}

#[test]
fn resume_after_every_operation_completes_adoption_and_restores_the_main_fixture() {
    for crash_at in 0..14 {
        let (_temp, engine, profile) = fixture();
        let original = tree_hash(&engine.platform.slot);
        let runner = SignIn::new(&engine);
        let j = runner.prepare(&profile.id, true).unwrap();
        engine.platform.crash.store(crash_at, Ordering::SeqCst);
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            runner.resume(&j.id).unwrap();
            runner.finish(&j.id).unwrap();
        }));
        engine.platform.crash.store(-1, Ordering::SeqCst);
        let recovered = SignIn::new(&engine).resume(&j.id).unwrap();
        let complete = if recovered.state == State::AwaitingSignIn {
            SignIn::new(&engine).finish(&j.id).unwrap()
        } else {
            recovered
        };
        assert_eq!(complete.state, State::Done, "operation {crash_at}");
        assert_eq!(
            tree_hash(&engine.platform.slot),
            original,
            "operation {crash_at}"
        );
        assert_eq!(tree_hash(&j.target), tree_hash(&j.capture()));
    }
}

#[test]
fn successful_capture_restores_main_retains_previous_data_and_requires_manual_account_confirmation()
{
    let (_temp, engine, profile) = fixture();
    engine.launch_main("claude-desktop").unwrap();
    let original = tree_hash(&engine.platform.slot);
    let runner = SignIn::new(&engine);
    assert!(runner.prepare(&profile.id, false).is_err());
    let j = runner.prepare(&profile.id, true).unwrap();
    assert!(runner.prepare(&profile.id, true).is_err());
    assert!(engine.launch(&profile.id).is_err());
    assert!(engine.launch_main("claude-desktop").is_err());
    assert!(engine.close_main("claude-desktop").is_err());
    assert!(engine.store.remove(&profile.id).is_err());
    runner.resume(&j.id).unwrap();
    let fresh = tree_hash(&engine.platform.slot);
    let complete = runner.finish(&j.id).unwrap();
    assert_eq!(complete.state, State::Done);
    assert_eq!(tree_hash(&engine.platform.slot), original);
    assert_eq!(tree_hash(&complete.target), fresh);
    assert_eq!(tree_hash(&j.capture()), fresh);
    assert!(!engine.main_processes("claude-desktop").unwrap().is_empty());
    assert_eq!(
        engine
            .store
            .read()
            .unwrap()
            .profile(&profile.id)
            .unwrap()
            .sign_in_state,
        SignInState::Unknown
    );
}

#[test]
fn partial_adoption_failure_restores_main_and_previous_profile_without_pruning_partial_or_capture()
{
    let (_temp, mut engine, profile) = fixture();
    engine.platform.fail_copy = true;
    let original = tree_hash(&engine.platform.slot);
    let previous = tree_hash(&Path::new(&profile.data_dir).join("data"));
    let runner = SignIn::new(&engine);
    let j = runner.prepare(&profile.id, true).unwrap();
    runner.resume(&j.id).unwrap();
    assert!(runner.finish(&j.id).is_err());
    assert_eq!(tree_hash(&engine.platform.slot), original);
    assert_eq!(tree_hash(&j.target), previous);
    assert!(j.capture().exists());
    assert_eq!(
        JournalStore::new(engine.store.root())
            .read(&j.id)
            .unwrap()
            .state,
        State::Aborted
    );
}

#[test]
fn preflight_and_recovery_fail_closed_on_unknown_layout_low_space_and_tampered_paths() {
    let (_temp, mut engine, profile) = fixture();
    engine.platform.space = 0;
    assert!(SignIn::new(&engine).prepare(&profile.id, true).is_err());
    engine.platform.space = u64::MAX;
    fs::write(
        engine.platform.slot.join(".user-data-left-fixture"),
        b"not read",
    )
    .unwrap();
    assert!(SignIn::new(&engine).prepare(&profile.id, true).is_err());
    // Rename the marker away, never remove it, even in this fixture.
    fs::rename(
        engine.platform.slot.join(".user-data-left-fixture"),
        engine.platform.slot.join("retained-marker-fixture"),
    )
    .unwrap();
    let j = SignIn::new(&engine).prepare(&profile.id, true).unwrap();
    let path = engine
        .store
        .root()
        .join("journal")
        .join(format!("signin-{}.json", j.id));
    let mut value: serde_json::Value = serde_json::from_slice(&fs::read(&path).unwrap()).unwrap();
    value["target"] = serde_json::json!(engine.platform.slot);
    fs::write(&path, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(SignIn::new(&engine).restore(&j.id).is_err());
    assert!(engine.platform.slot.exists());
}

#[test]
fn metadata_hint_requires_a_burst_and_three_seconds_of_quiet_and_never_claims_identity() {
    let mut activity = MetadataActivity::default();
    assert!(!activity.ready(10_000));
    activity.record(100);
    assert!(!activity.ready(10_000));
    activity.record(200);
    assert!(!activity.ready(3_199));
    assert!(activity.ready(3_200));
    activity.record(4_000);
    assert!(!activity.ready(4_001));
}

#[test]
fn changed_default_layout_keeps_recovery_pending_after_restoring_the_original() {
    let (_temp, engine, profile) = fixture();
    let original = tree_hash(&engine.platform.slot);
    let runner = SignIn::new(&engine);
    let j = runner.prepare(&profile.id, true).unwrap();
    runner.resume(&j.id).unwrap();
    let dirs = engine.platform.directories().unwrap();
    let unexpected =
        layout::claude_candidates(Path::new(&dirs.local_app_data), Path::new(&dirs.app_data))[1]
            .clone();
    fixture_tree(&unexpected, b"unexpected alternate fixture slot");
    assert!(runner.restore(&j.id).is_err());
    assert_eq!(tree_hash(&engine.platform.slot), original);
    assert!(unexpected.exists());
    assert_eq!(
        JournalStore::new(engine.store.root())
            .read(&j.id)
            .unwrap()
            .state,
        State::Restoring
    );
    assert!(engine.launch_main("claude-desktop").is_err());
}

#[test]
fn mcp_copy_transfers_only_the_named_config_and_keeps_a_retained_destination_backup() {
    let (_temp, engine, source) = fixture();
    let root = Path::new(&source.data_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap();
    let target = engine
        .store
        .create(Profile::new("claude-desktop", "MCP target", root, ProfileColor::Plum).unwrap())
        .unwrap();
    let from_dir = Path::new(&source.data_dir).join("data");
    let to_dir = Path::new(&target.data_dir).join("data");
    fs::create_dir_all(&to_dir).unwrap();
    fs::write(
        from_dir.join("claude_desktop_config.json"),
        b"synthetic MCP configuration",
    )
    .unwrap();
    fs::write(
        from_dir.join("unrelated-login-fixture.bin"),
        b"must not copy",
    )
    .unwrap();
    fs::write(
        to_dir.join("claude_desktop_config.json"),
        b"previous synthetic MCP",
    )
    .unwrap();
    signin::copy_mcp(&engine, &source.id, &target.id).unwrap();
    assert_eq!(
        fs::read(to_dir.join("claude_desktop_config.json")).unwrap(),
        b"synthetic MCP configuration"
    );
    assert!(!to_dir.join("unrelated-login-fixture.bin").exists());
    assert_eq!(fs::read_dir(&to_dir).unwrap().count(), 2);
    assert!(signin::copy_mcp(&engine, &source.id, &source.id).is_err());
}
