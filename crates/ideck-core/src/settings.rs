//! Versioned preferences owned by InstanceDeck; no target app data is read.
use crate::{Error, Result, error::io, paths};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Settings {
    pub schema_version: u32,
    pub profiles_root: String,
    pub close_to_tray: bool,
    pub start_with_windows: bool,
    pub switcher_hotkey: String,
    pub allow_update_checks: bool,
    pub update_channel: UpdateChannel,
    pub theme: Theme,
    pub onboarding_completed: bool,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum UpdateChannel {
    Stable,
    Beta,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Theme {
    Light,
    Dark,
}
impl Settings {
    pub fn defaults(state_root: &Path) -> Self {
        Self {
            schema_version: 1,
            profiles_root: state_root.join("Profiles").to_string_lossy().into_owned(),
            close_to_tray: true,
            start_with_windows: false,
            switcher_hotkey: "Ctrl+Alt+Space".into(),
            allow_update_checks: true,
            update_channel: UpdateChannel::Stable,
            theme: Theme::Light,
            onboarding_completed: false,
        }
    }
    pub fn validate(&self) -> Result<()> {
        if self.schema_version != 1
            || self.switcher_hotkey.is_empty()
            || self.switcher_hotkey.len() > 80
            || self.switcher_hotkey.chars().any(char::is_control)
        {
            return Err(Error::Invalid(
                "Unsupported settings version or invalid switcher hotkey".into(),
            ));
        }
        paths::validate_absolute(&self.profiles_root)
    }
}
pub struct SettingsStore {
    root: PathBuf,
}
impl SettingsStore {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }
    pub fn read(&self) -> Result<Settings> {
        let path = self.root.join("settings.json");
        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Ok(Settings::defaults(&self.root));
            }
            Err(error) => return Err(io(&path, error)),
        };
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 1024 * 1024
        {
            return Err(Error::Invalid(
                "Settings must be a regular metadata file smaller than 1 MB".into(),
            ));
        }
        let settings: Settings =
            serde_json::from_str(&fs::read_to_string(&path).map_err(|error| io(&path, error))?)?;
        settings.validate()?;
        Ok(settings)
    }
    pub fn write(&self, settings: &Settings) -> Result<()> {
        settings.validate()?;
        fs::create_dir_all(&self.root).map_err(|error| io(&self.root, error))?;
        let lock_path = self.root.join("settings.lock");
        let lock = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(&lock_path)
            .map_err(|error| io(&lock_path, error))?;
        FileExt::lock_exclusive(&lock).map_err(|error| io(&lock_path, error))?;
        self.read()?; // Refuse to overwrite unknown or invalid existing metadata.
        let id = ulid::Ulid::generate();
        let path = self.root.join("settings.json");
        let staged = self.root.join(format!("settings.pending-{id}.json"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|error| io(&staged, error))?;
        file.write_all((serde_json::to_string_pretty(settings)? + "\n").as_bytes())
            .and_then(|()| file.sync_all())
            .map_err(|error| io(&staged, error))?;
        drop(file);
        if path.try_exists().map_err(|error| io(&path, error))? {
            let backup = self.root.join(format!("settings.backup-{id}.json"));
            fs::copy(&path, &backup).map_err(|error| io(&backup, error))?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&backup)
                .and_then(|file| file.sync_all())
                .map_err(|error| io(&backup, error))?;
        }
        fs::rename(&staged, &path).map_err(|error| io(&path, error))
    }
}
