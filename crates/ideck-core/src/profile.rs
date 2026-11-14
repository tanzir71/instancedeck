//! Reads only InstanceDeck's own profiles.json; profile data is never inspected.
use crate::{Error, Result, error::io, paths, recipe::valid_id};
use fs2::FileExt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub enum ProfileColor {
    Terracotta,
    Teal,
    Iris,
    Ochre,
    Sky,
    Plum,
    Moss,
    #[default]
    Graphite,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema, Default)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub enum SignInState {
    SignedIn,
    #[default]
    Unknown,
    NeedsSignIn,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub struct ProfileOptions {
    #[serde(default = "default_true")]
    pub isolate_claude_code: bool,
    #[serde(default)]
    pub shared_codex_history: bool,
}
fn default_true() -> bool {
    true
}
impl Default for ProfileOptions {
    fn default() -> Self {
        Self {
            isolate_claude_code: true,
            shared_codex_history: false,
        }
    }
}
fn default_badge() -> String {
    "monogram".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub struct Profile {
    pub id: String,
    pub app_id: String,
    pub name: String,
    #[serde(default)]
    pub color: ProfileColor,
    #[serde(default = "default_badge")]
    pub badge: String,
    pub data_dir: String,
    #[serde(default)]
    pub options: ProfileOptions,
    #[serde(default)]
    pub hotkey: Option<String>,
    pub created_at: String,
    #[serde(default)]
    pub last_launched_at: Option<String>,
    #[serde(default)]
    pub sign_in_state: SignInState,
}

pub fn timestamp() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("UTC timestamp is representable")
}

impl Profile {
    pub fn new(app_id: &str, name: &str, root: &Path, color: ProfileColor) -> Result<Self> {
        let id = ulid::Ulid::generate().to_string();
        let profile = Self {
            data_dir: root.join(app_id).join(&id).to_string_lossy().into_owned(),
            id,
            app_id: app_id.into(),
            name: name.trim().into(),
            color,
            badge: default_badge(),
            options: ProfileOptions::default(),
            hotkey: None,
            created_at: timestamp(),
            last_launched_at: None,
            sign_in_state: SignInState::NeedsSignIn,
        };
        profile.validate()?;
        Ok(profile)
    }
    pub fn validate(&self) -> Result<()> {
        if ulid::Ulid::from_string(&self.id).is_err()
            || !valid_id(&self.app_id)
            || self.name.trim().is_empty()
            || self.name.chars().count() > 120
            || self.name.chars().any(char::is_control)
        {
            return Err(Error::Invalid(
                "Profile requires a ULID, safe app id and a name of 1–120 characters".into(),
            ));
        }
        paths::validate_absolute(&self.data_dir)?;
        if self.badge != "monogram" {
            return Err(Error::Invalid(
                "Only neutral monogram badges are currently supported".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProfileState {
    pub schema_version: u32,
    pub profiles: Vec<Profile>,
}
impl Default for ProfileState {
    fn default() -> Self {
        Self {
            schema_version: 1,
            profiles: Vec::new(),
        }
    }
}

impl ProfileState {
    pub fn from_json(input: &str) -> Result<Self> {
        let mut state: Self = serde_json::from_str(input)?;
        match state.schema_version {
            0 => state.schema_version = 1, // v0: same identity/path fields, optional visual/options fields.
            1 => {}
            version => {
                return Err(Error::Invalid(format!(
                    "Unsupported profiles schema {version}; the file was not changed"
                )));
            }
        }
        let mut ids = BTreeSet::new();
        let mut dirs = BTreeSet::new();
        for profile in &state.profiles {
            profile.validate()?;
            if !ids.insert(&profile.id) || !dirs.insert(paths::normalize_windows(&profile.data_dir))
            {
                return Err(Error::Invalid(
                    "Duplicate profile id or data directory".into(),
                ));
            }
        }
        Ok(state)
    }
    pub fn profile(&self, id: &str) -> Result<&Profile> {
        self.profiles
            .iter()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::NotFound("Profile".into()))
    }
}

#[derive(Clone, Debug)]
pub struct ProfileStore {
    root: PathBuf,
}
impl ProfileStore {
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            root: state_dir.into(),
        }
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn read(&self) -> Result<ProfileState> {
        let file = self.root.join("profiles.json");
        match fs::symlink_metadata(&file) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                Ok(ProfileState::default())
            }
            Err(error) => Err(io(&file, error)),
            Ok(metadata) if metadata.file_type().is_symlink() => Err(Error::Invalid(
                "Profile metadata cannot be a symbolic link".into(),
            )),
            Ok(metadata) if metadata.len() > 32 * 1024 * 1024 => Err(Error::Invalid(
                "profiles.json exceeds the 32 MB safety limit".into(),
            )),
            Ok(_) => ProfileState::from_json(&fs::read_to_string(&file).map_err(|e| io(&file, e))?),
        }
    }
    pub fn transaction<T>(&self, change: impl FnOnce(&mut ProfileState) -> Result<T>) -> Result<T> {
        fs::create_dir_all(&self.root).map_err(|e| io(&self.root, e))?;
        let lock_path = self.root.join("profiles.lock");
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(&lock_path)
            .map_err(|e| io(&lock_path, e))?;
        FileExt::lock_exclusive(&lock).map_err(|e| io(&lock_path, e))?;
        let mut state = self.read()?;
        let result = change(&mut state)?;
        // Validate the resulting document before any write; never silently drop fields.
        let json = serde_json::to_string_pretty(&state)? + "\n";
        ProfileState::from_json(&json)?;
        self.write(&json)?;
        Ok(result)
    }
    fn write(&self, json: &str) -> Result<()> {
        use std::io::Write;
        let id = ulid::Ulid::generate();
        let path = self.root.join("profiles.json");
        let staged = self.root.join(format!("profiles.pending-{id}.json"));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&staged)
            .map_err(|e| io(&staged, e))?;
        file.write_all(json.as_bytes())
            .and_then(|_| file.sync_all())
            .map_err(|e| io(&staged, e))?;
        drop(file);
        if path.try_exists().map_err(|e| io(&path, e))? {
            let backup = self.root.join(format!("profiles.backup-{id}.json"));
            // This is the manager's own metadata, never an app's profile or credential data.
            fs::copy(&path, &backup).map_err(|e| io(&backup, e))?;
            OpenOptions::new()
                .read(true)
                .write(true)
                .open(&backup)
                .and_then(|f| f.sync_all())
                .map_err(|e| io(&backup, e))?;
        }
        // std::fs::rename atomically replaces a file on Windows (MoveFileExW).
        fs::rename(&staged, &path).map_err(|e| io(&path, e))?;
        Ok(())
    }
    pub fn create(&self, profile: Profile) -> Result<Profile> {
        self.transaction(|state| {
            state.profiles.push(profile.clone());
            Ok(profile)
        })
    }
    pub fn rename(&self, id: &str, name: &str) -> Result<()> {
        self.transaction(|state| {
            let profile = state
                .profiles
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| Error::NotFound("Profile".into()))?;
            profile.name = name.trim().into();
            Ok(())
        })
    }
    /// Removing metadata never touches the profile's data folder.
    pub fn remove(&self, id: &str) -> Result<Profile> {
        self.transaction(|state| {
            let position = state
                .profiles
                .iter()
                .position(|p| p.id == id)
                .ok_or_else(|| Error::NotFound("Profile".into()))?;
            Ok(state.profiles.remove(position))
        })
    }
    pub fn mark_launched(&self, id: &str) -> Result<()> {
        self.transaction(|state| {
            state
                .profiles
                .iter_mut()
                .find(|p| p.id == id)
                .ok_or_else(|| Error::NotFound("Profile".into()))?
                .last_launched_at = Some(timestamp());
            Ok(())
        })
    }
}
