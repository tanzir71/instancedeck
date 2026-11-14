use ideck_core::{
    DesktopOperations, Engine, Error, Platform, Profile, ProfileColor, ProfileStore, Result,
    Settings, SettingsStore, built_in_recipes, layout, paths,
};
use ideck_win::WindowsPlatform;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::{Read, Seek, SeekFrom},
    path::{Path, PathBuf},
    sync::Mutex,
};

pub struct DesktopState<P: Platform = WindowsPlatform> {
    pub engine: Engine<P>,
    pub settings: SettingsStore,
    mutation: Mutex<()>,
}
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    profiles: Vec<Profile>,
    apps: Vec<ideck_core::engine::AppDetection>,
    running: BTreeMap<String, Vec<u32>>,
    running_since: BTreeMap<String, u64>,
    main_running: BTreeMap<String, Vec<u32>>,
    status_errors: BTreeMap<String, String>,
    settings: Settings,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateProfile {
    id: Option<String>,
    app_id: String,
    name: String,
    color: ProfileColor,
    root: String,
    isolate_claude_code: bool,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ProfileAction {
    Launch,
    Close,
    ForceClose,
    Remove,
    Recycle,
    OpenFolder,
}
#[derive(Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MainAction {
    Launch,
    Close,
}
impl DesktopState {
    pub fn new(root: PathBuf) -> Result<Self> {
        let platform = WindowsPlatform::new(&root);
        Self::with_platform(root, platform)
    }
}
impl<P: Platform + DesktopOperations> DesktopState<P> {
    fn with_platform(root: PathBuf, platform: P) -> Result<Self> {
        Ok(Self {
            engine: Engine::new(platform, ProfileStore::new(&root), built_in_recipes()?)?,
            settings: SettingsStore::new(root),
            mutation: Mutex::new(()),
        })
    }
    pub fn snapshot(&self, refresh: bool) -> Result<Snapshot> {
        let profiles = self.engine.store.read()?.profiles;
        let apps = self.engine.apps(refresh);
        let mut running = BTreeMap::new();
        let mut running_since = BTreeMap::new();
        let mut main_running = BTreeMap::new();
        let mut status_errors = BTreeMap::new();
        for profile in &profiles {
            match self.engine.running(profile) {
                Ok(processes) => {
                    if let Some(started) = processes
                        .iter()
                        .map(|process| process.started_at)
                        .filter(|started| *started >= 116_444_736_000_000_000)
                        .min()
                    {
                        running_since.insert(
                            profile.id.clone(),
                            (started - 116_444_736_000_000_000) / 10_000,
                        );
                    }
                    running.insert(
                        profile.id.clone(),
                        processes.iter().map(|process| process.pid).collect(),
                    );
                }
                Err(error) => {
                    status_errors.insert(profile.id.clone(), error.to_string());
                }
            }
        }
        for app in &apps {
            if app.installed.is_some() {
                match self.engine.main_processes(&app.recipe.id) {
                    Ok(processes) => {
                        main_running.insert(
                            app.recipe.id.clone(),
                            processes.iter().map(|process| process.pid).collect(),
                        );
                    }
                    Err(error) => {
                        status_errors.insert(format!("main:{}", app.recipe.id), error.to_string());
                    }
                }
            }
        }
        Ok(Snapshot {
            profiles,
            apps,
            running,
            running_since,
            main_running,
            status_errors,
            settings: self.settings.read()?,
        })
    }
    pub fn create(&self, input: CreateProfile) -> Result<Profile> {
        self.engine.recipe(&input.app_id)?;
        let mut profile = Profile::new(
            &input.app_id,
            &input.name,
            Path::new(&input.root),
            input.color,
        )?;
        profile.options.isolate_claude_code = input.isolate_claude_code;
        if let Some(id) = input.id {
            profile.data_dir = Path::new(&input.root)
                .join(&input.app_id)
                .join(&id)
                .to_string_lossy()
                .into_owned();
            profile.id = id;
        }
        self.engine.store.create(profile)
    }
    pub fn update(&self, profile: Profile) -> Result<()> {
        profile.validate()?;
        self.engine.store.transaction(|state| {
            let saved = state
                .profiles
                .iter_mut()
                .find(|saved| saved.id == profile.id)
                .ok_or_else(|| Error::NotFound("Profile".into()))?;
            // Editable fields only. IPC cannot redirect a saved profile or change account state.
            saved.name = profile.name.trim().into();
            saved.color = profile.color;
            saved.options.isolate_claude_code = profile.options.isolate_claude_code;
            Ok(())
        })
    }
    pub fn action(
        &self,
        id: &str,
        action: ProfileAction,
        confirmation: Option<&str>,
    ) -> Result<()> {
        let _mutation = self.mutation.lock().map_err(|_| {
            Error::Platform("A previous operation failed; restart InstanceDeck".into())
        })?;
        let profile = self.engine.store.read()?.profile(id)?.clone();
        match action {
            ProfileAction::Launch => {
                self.engine.launch(id)?;
            }
            ProfileAction::Close => self.engine.close(id)?,
            ProfileAction::ForceClose => {
                Self::confirm(&profile, confirmation)?;
                self.engine.force_close(id, true)?;
            }
            ProfileAction::Remove => {
                Self::confirm(&profile, confirmation)?;
                self.engine.store.remove(id)?;
            }
            ProfileAction::Recycle => {
                Self::confirm(&profile, confirmation)?;
                let _launch_lock = self.engine.platform.launch_lock()?;
                if !self.engine.running(&profile)?.is_empty() {
                    return Err(Error::StillRunning);
                }
                self.engine.platform.recycle_profile(&profile)?;
                self.engine.store.remove(id)?;
            }
            ProfileAction::OpenFolder => self.engine.platform.open_folder(&profile.data_dir)?,
        }
        Ok(())
    }
    fn confirm(profile: &Profile, confirmation: Option<&str>) -> Result<()> {
        if confirmation != Some(profile.name.as_str()) {
            return Err(Error::Invalid(
                "Type the exact profile name to confirm this action".into(),
            ));
        }
        Ok(())
    }
    pub fn main_action(&self, app_id: &str, action: MainAction) -> Result<()> {
        match action {
            MainAction::Launch => self.engine.launch_main(app_id),
            MainAction::Close => self.engine.close_main(app_id),
        }
    }
    pub fn save_settings(&self, settings: &Settings) -> Result<()> {
        let previous = self.settings.read()?;
        if settings.start_with_windows != previous.start_with_windows
            || settings.switcher_hotkey != previous.switcher_hotkey
            || !matches!(settings.theme, ideck_core::settings::Theme::Light)
        {
            return Err(Error::Invalid(
                "Shell integration and dark theme are not installed yet".into(),
            ));
        }
        self.settings.write(settings)
    }
    pub fn diagnostics(&self, redact: bool) -> Result<Value> {
        let directories = self.engine.platform.directories()?;
        let layouts = layout::claude_candidates(
            Path::new(&directories.local_app_data),
            Path::new(&directories.app_data),
        )
        .iter()
        .map(|path| match layout::probe(path) {
            Ok(probe) => json!({"path": path, "live": probe.live, "relocated": probe.relocated}),
            Err(error) => json!({"path": path, "error": error.to_string()}),
        })
        .collect::<Vec<_>>();
        let snapshot = self.snapshot(false)?;
        let mut report = json!({ "product": "InstanceDeck", "version": env!("CARGO_PKG_VERSION"), "directories": directories, "claudeLayouts": layouts, "profiles": snapshot.profiles.iter().map(|profile| json!({"id": profile.id, "appId": profile.app_id, "runningPids": snapshot.running.get(&profile.id), "accountState": profile.sign_in_state})).collect::<Vec<_>>(), "mainInstances": snapshot.main_running, "statusErrors": snapshot.status_errors, "apps": snapshot.apps, "launchLogs": self.launch_logs()?, "accountHealth": "Manual confirmation required in each official app. No credentials are read." });
        if redact {
            redact_value(&mut report, &directories.user_profile);
        }
        Ok(report)
    }
    fn launch_logs(&self) -> Result<Vec<Value>> {
        let root = self.engine.store.root().join("logs");
        let mut entries = match fs::read_dir(&root) {
            Ok(entries) => entries
                .filter_map(std::result::Result::ok)
                .filter(|entry| {
                    entry
                        .file_name()
                        .to_string_lossy()
                        .starts_with("launches.jsonl")
                })
                .collect::<Vec<_>>(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(ideck_core::error::io(&root, error)),
        };
        entries.sort_by_key(std::fs::DirEntry::file_name);
        let mut rows = Vec::new();
        for entry in entries.iter().rev().take(3).rev() {
            let path = entry.path();
            let metadata =
                fs::symlink_metadata(&path).map_err(|error| ideck_core::error::io(&path, error))?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                continue;
            }
            let mut file =
                fs::File::open(&path).map_err(|error| ideck_core::error::io(&path, error))?;
            let start = metadata.len().saturating_sub(128 * 1024);
            file.seek(SeekFrom::Start(start))
                .map_err(|error| ideck_core::error::io(&path, error))?;
            let mut bytes = Vec::new();
            file.take(128 * 1024)
                .read_to_end(&mut bytes)
                .map_err(|error| ideck_core::error::io(&path, error))?;
            for line in String::from_utf8_lossy(&bytes)
                .lines()
                .skip(usize::from(start > 0))
            {
                if let Ok(value) = serde_json::from_str(line) {
                    rows.push(value);
                }
            }
        }
        Ok(rows
            .into_iter()
            .rev()
            .take(50)
            .collect::<Vec<_>>()
            .into_iter()
            .rev()
            .collect())
    }
}
fn redact_value(value: &mut Value, user_profile: &str) {
    match value {
        Value::String(text) => *text = paths::redact(text, user_profile),
        Value::Array(values) => values
            .iter_mut()
            .for_each(|value| redact_value(value, user_profile)),
        Value::Object(values) => values
            .values_mut()
            .for_each(|value| redact_value(value, user_profile)),
        _ => {}
    }
}

#[cfg(test)]
#[path = "../../../../crates/ideck-core/tests/common/mod.rs"]
mod fake;

#[cfg(test)]
mod tests {
    use super::fake::FakePlatform;
    use super::*;
    impl DesktopOperations for FakePlatform {
        fn open_folder(&self, _: &str) -> Result<()> {
            self.0.lock().unwrap().events.push("open-folder");
            Ok(())
        }
        fn recycle_profile(&self, profile: &Profile) -> Result<()> {
            let mut state = self.0.lock().unwrap();
            if state.fail_recycle {
                return Err(Error::Platform("Fixture Recycle Bin failure".into()));
            }
            state.recycled.push(profile.data_dir.clone());
            Ok(())
        }
    }
    #[test]
    fn editing_and_removing_metadata_cannot_redirect_or_delete_profile_data() {
        let temp = tempfile::tempdir().unwrap();
        let state =
            DesktopState::with_platform(temp.path().join("manager"), FakePlatform::default())
                .unwrap();
        let profile = state
            .create(CreateProfile {
                id: None,
                app_id: "codex".into(),
                name: "P2 fixture Zoë 日本語".into(),
                color: ProfileColor::Teal,
                root: temp
                    .path()
                    .join("Profiles with spaces")
                    .to_string_lossy()
                    .into_owned(),
                isolate_claude_code: true,
            })
            .unwrap();
        fs::create_dir_all(&profile.data_dir).unwrap();
        let sentinel = Path::new(&profile.data_dir).join("fixture.txt");
        fs::write(&sentinel, "retained").unwrap();
        let mut changed = profile.clone();
        changed.name = "Renamed".into();
        changed.data_dir = r"D:\\Unrelated".into();
        changed.sign_in_state = ideck_core::SignInState::SignedIn;
        state.update(changed).unwrap();
        let saved = state
            .engine
            .store
            .read()
            .unwrap()
            .profile(&profile.id)
            .unwrap()
            .clone();
        assert_eq!(saved.data_dir, profile.data_dir);
        assert_eq!(saved.sign_in_state, ideck_core::SignInState::NeedsSignIn);
        assert!(
            state
                .create(CreateProfile {
                    id: Some("../unrelated".into()),
                    app_id: "codex".into(),
                    name: "Invalid id".into(),
                    color: ProfileColor::Teal,
                    root: temp.path().join("Profiles").to_string_lossy().into_owned(),
                    isolate_claude_code: true
                })
                .is_err()
        );
        assert_eq!(state.engine.store.read().unwrap().profiles.len(), 1);
        assert!(
            state
                .action(&profile.id, ProfileAction::Remove, Some("wrong"))
                .is_err()
        );
        assert!(
            state
                .action(&profile.id, ProfileAction::Recycle, None)
                .is_err()
        );
        assert!(
            state
                .action(&profile.id, ProfileAction::ForceClose, None)
                .is_err()
        );
        state
            .action(&profile.id, ProfileAction::Remove, Some("Renamed"))
            .unwrap();
        assert!(state.engine.store.read().unwrap().profiles.is_empty());
        assert_eq!(fs::read_to_string(sentinel).unwrap(), "retained");
    }
    #[test]
    fn diagnostic_redaction_recurses_and_settings_reject_uninstalled_features() {
        let temp = tempfile::tempdir().unwrap();
        let state = DesktopState::with_platform(temp.path().to_path_buf(), FakePlatform::default())
            .unwrap();
        let mut settings = state.settings.read().unwrap();
        settings.start_with_windows = true;
        assert!(state.save_settings(&settings).is_err());
        let mut report = json!({"nested": [{"path": "C:\\Users\\Zoë\\AppData\\Local"}]});
        redact_value(&mut report, r"C:\Users\Zoë");
        assert!(!report.to_string().contains("Zoë"));
    }
    #[test]
    fn recycle_requires_confirmation_and_stopped_state_and_keeps_metadata_on_failure() {
        let temp = tempfile::tempdir().unwrap();
        let state =
            DesktopState::with_platform(temp.path().join("manager"), FakePlatform::default())
                .unwrap();
        let profile = state.engine.store.create(fake::profile("codex")).unwrap();
        assert!(
            state
                .action(&profile.id, ProfileAction::Recycle, Some("wrong"))
                .is_err()
        );
        assert!(state.engine.platform.0.lock().unwrap().recycled.is_empty());
        state
            .action(&profile.id, ProfileAction::Launch, None)
            .unwrap();
        state.engine.platform.0.lock().unwrap().processes[0].started_at = 116_444_736_420_000_000;
        let snapshot = state.snapshot(false).unwrap();
        assert_eq!(snapshot.running_since.get(&profile.id), Some(&42_000));
        assert!(matches!(
            snapshot.profiles[0].sign_in_state,
            ideck_core::SignInState::NeedsSignIn
        ));
        assert!(matches!(
            state.action(&profile.id, ProfileAction::Recycle, Some(&profile.name)),
            Err(Error::StillRunning)
        ));
        assert!(state.engine.platform.0.lock().unwrap().recycled.is_empty());
        state
            .action(&profile.id, ProfileAction::Close, None)
            .unwrap();
        state.engine.platform.0.lock().unwrap().fail_recycle = true;
        assert!(
            state
                .action(&profile.id, ProfileAction::Recycle, Some(&profile.name))
                .is_err()
        );
        assert_eq!(state.engine.store.read().unwrap().profiles.len(), 1);
        state.engine.platform.0.lock().unwrap().fail_recycle = false;
        state
            .action(&profile.id, ProfileAction::Recycle, Some(&profile.name))
            .unwrap();
        assert!(state.engine.store.read().unwrap().profiles.is_empty());
        assert_eq!(
            state.engine.platform.0.lock().unwrap().recycled,
            vec![profile.data_dir]
        );
    }
}
