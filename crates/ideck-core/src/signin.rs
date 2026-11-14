//! Journaled directory operations. Only manager-owned journals are opened here;
//! target app data is handled through opaque platform operations.
use crate::{Engine, Error, Platform, Profile, Result, SignInState, error::io, layout};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

pub trait SignInFiles: Send + Sync {
    /// Metadata only; reject links/reparse points throughout the tree.
    fn tree_bytes(&self, path: &Path) -> Result<u64>;
    fn free_bytes(&self, path: &Path) -> Result<u64>;
    /// Same-volume, no-replace rename, with bounded handle-release retries.
    fn rename_data(&self, from: &Path, to: &Path) -> Result<()>;
    /// Opaque OS copy. Never open, parse or log target file contents. Retain source.
    fn copy_data(&self, from: &Path, to: &Path) -> Result<()>;
    fn sign_in_hint(&self, slot: &Path, session: &str) -> Result<bool>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub enum State {
    Preflight,
    Stashed,
    AwaitingSignIn,
    Captured,
    Adopting,
    Restoring,
    Done,
    Aborted,
}
impl State {
    pub fn terminal(self) -> bool {
        matches!(self, Self::Done | Self::Aborted)
    }
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
enum Adoption {
    #[default]
    Preserve,
    Copy,
    Activate,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Journal {
    schema_version: u32,
    pub id: String,
    pub profile_id: String,
    pub state: State,
    pub slot: PathBuf,
    pub target: PathBuf,
    pub main_was_running: bool,
    close_main_confirmed: bool,
    pub updated_at: String,
    adoption: Adoption,
    restore_only: bool,
}
impl Journal {
    fn beside_slot(&self, kind: &str) -> PathBuf {
        self.slot.with_file_name(format!(
            "{}.ideck-{kind}-{}",
            self.slot.file_name().unwrap_or_default().to_string_lossy(),
            self.id
        ))
    }
    pub fn stash(&self) -> PathBuf {
        self.beside_slot("stash")
    }
    pub fn capture(&self) -> PathBuf {
        self.beside_slot("capture")
    }
    fn cancelled(&self) -> PathBuf {
        self.beside_slot("cancelled")
    }
    fn incoming(&self) -> PathBuf {
        self.target
            .with_file_name(format!("data.ideck-incoming-{}", self.id))
    }
    fn previous(&self) -> PathBuf {
        self.target
            .with_file_name(format!("data.ideck-before-{}", self.id))
    }
}

#[derive(Clone, Debug)]
pub struct JournalStore {
    root: PathBuf,
}
impl JournalStore {
    pub fn new(state_root: &Path) -> Self {
        Self {
            root: state_root.join("journal"),
        }
    }
    fn path(&self, id: &str) -> Result<PathBuf> {
        if ulid::Ulid::from_string(id).is_err() {
            return Err(Error::Invalid("Invalid sign-in journal ID".into()));
        }
        Ok(self.root.join(format!("signin-{id}.json")))
    }
    pub fn read(&self, id: &str) -> Result<Journal> {
        let path = self.path(id)?;
        let metadata = fs::symlink_metadata(&path).map_err(|e| io(&path, e))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 64 * 1024 {
            return Err(Error::Invalid(
                "Sign-in journal is not a small regular manager file".into(),
            ));
        }
        let journal: Journal = serde_json::from_slice(&fs::read(&path).map_err(|e| io(&path, e))?)?;
        if journal.schema_version != 1 || journal.id != id {
            return Err(Error::Invalid(
                "Unsupported or mismatched sign-in journal".into(),
            ));
        }
        Ok(journal)
    }
    pub fn active(&self) -> Result<Vec<Journal>> {
        let entries = match fs::read_dir(&self.root) {
            Ok(entries) => entries,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(e) => return Err(io(&self.root, e)),
        };
        let mut journals = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|e| io(&self.root, e))?;
            let name = entry.file_name();
            let name = name.to_string_lossy();
            if let Some(id) = name
                .strip_prefix("signin-")
                .and_then(|v| v.strip_suffix(".json"))
            {
                let journal = self.read(id)?;
                if !journal.state.terminal() {
                    journals.push(journal);
                }
            }
        }
        journals.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(journals)
    }
    fn save(&self, journal: &mut Journal) -> Result<()> {
        fs::create_dir_all(&self.root).map_err(|e| io(&self.root, e))?;
        journal.updated_at = crate::profile::timestamp();
        let path = self.path(&journal.id)?;
        // Retain each committed state. A pending file is never treated as committed.
        let nonce = ulid::Ulid::generate();
        let pending = self.root.join(format!("pending-{nonce}.json"));
        let mut file = fs::OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&pending)
            .map_err(|e| io(&pending, e))?;
        file.write_all(&serde_json::to_vec_pretty(journal)?)
            .and_then(|_| file.sync_all())
            .map_err(|e| io(&pending, e))?;
        drop(file);
        if path.try_exists().map_err(|e| io(&path, e))? {
            let backup = self
                .root
                .join(format!("history-{}-{nonce}.json", journal.id));
            fs::copy(&path, &backup).map_err(|e| io(&backup, e))?;
            fs::OpenOptions::new()
                .write(true)
                .open(&backup)
                .and_then(|f| f.sync_all())
                .map_err(|e| io(&backup, e))?;
        }
        fs::rename(&pending, &path).map_err(|e| io(&path, e))
    }
}

/// All callers (UI, CLI and tray) use the same cross-process launch lock.
pub fn guard_launch(state_root: &Path, app_id: &str) -> Result<()> {
    if app_id == "claude-desktop" && !JournalStore::new(state_root).active()?.is_empty() {
        return Err(Error::Invalid(
            "Claude sign-in recovery is pending. Resume it or restore your main Claude first."
                .into(),
        ));
    }
    Ok(())
}

/// Copy exactly the explicit MCP configuration file, preserving any prior copy.
pub fn copy_mcp<P: Platform + SignInFiles>(engine: &Engine<P>, from: &str, to: &str) -> Result<()> {
    let _lock = engine.platform.launch_lock()?;
    guard_launch(engine.store.root(), "claude-desktop")?;
    let state = engine.store.read()?;
    let source = state.profile(from)?;
    let target = state.profile(to)?;
    if from == to || source.app_id != "claude-desktop" || target.app_id != "claude-desktop" {
        return Err(Error::Invalid(
            "Choose two different Claude profiles".into(),
        ));
    }
    if !engine.running(target)?.is_empty() {
        return Err(Error::StillRunning);
    }
    let source_dir = Path::new(&source.data_dir).join("data");
    let target_dir = Path::new(&target.data_dir).join("data");
    if layout::probe(&source_dir)?.relocated || layout::probe(&target_dir)?.relocated {
        return Err(Error::Invalid(
            "A Claude profile has a relocation marker; MCP copy was blocked".into(),
        ));
    }
    let source_file = source_dir.join("claude_desktop_config.json");
    let target_file = target_dir.join("claude_desktop_config.json");
    if !fs::symlink_metadata(&source_file)
        .map_err(|e| io(&source_file, e))?
        .is_file()
    {
        return Err(Error::Invalid(
            "The MCP source must be the regular configuration file".into(),
        ));
    }
    engine.platform.tree_bytes(&source_file)?;
    let backup = target_dir.join(format!(
        "claude_desktop_config.ideck-backup-{}.json",
        ulid::Ulid::generate()
    ));
    let had_target = target_file.try_exists().map_err(|e| io(&target_file, e))?;
    if had_target
        && !fs::symlink_metadata(&target_file)
            .map_err(|e| io(&target_file, e))?
            .is_file()
    {
        return Err(Error::Invalid(
            "The MCP destination is not a regular configuration file".into(),
        ));
    }
    if had_target {
        engine.platform.rename_data(&target_file, &backup)?;
    }
    if let Err(error) = engine.platform.copy_data(&source_file, &target_file) {
        if target_file.try_exists().map_err(|e| io(&target_file, e))? {
            engine.platform.rename_data(
                &target_file,
                &target_dir.join(format!(
                    "claude_desktop_config.ideck-incomplete-{}.json",
                    ulid::Ulid::generate()
                )),
            )?;
        }
        if had_target {
            engine.platform.rename_data(&backup, &target_file)?;
        }
        return Err(error);
    }
    Ok(())
}

pub struct SignIn<'a, P: Platform + SignInFiles> {
    engine: &'a Engine<P>,
    journals: JournalStore,
}
impl<'a, P: Platform + SignInFiles> SignIn<'a, P> {
    pub fn new(engine: &'a Engine<P>) -> Self {
        Self {
            engine,
            journals: JournalStore::new(engine.store.root()),
        }
    }
    fn validate(&self, journal: &Journal) -> Result<Profile> {
        let profile = self
            .engine
            .store
            .read()?
            .profile(&journal.profile_id)?
            .clone();
        let dirs = self.engine.platform.directories()?;
        if profile.app_id != "claude-desktop"
            || journal.target != Path::new(&profile.data_dir).join("data")
            || !layout::claude_candidates(
                Path::new(&dirs.local_app_data),
                Path::new(&dirs.app_data),
            )
            .contains(&journal.slot)
        {
            return Err(Error::Invalid("Journal paths do not match the saved profile and known Claude slots. No data was moved.".into()));
        }
        Ok(profile)
    }
    fn set_state(&self, journal: &mut Journal, state: State) -> Result<()> {
        journal.state = state;
        self.journals.save(journal)
    }
    pub fn prepare(&self, id: &str, close_main_confirmed: bool) -> Result<Journal> {
        let _lock = self.engine.platform.launch_lock()?;
        if !self.journals.active()?.is_empty() {
            return Err(Error::Invalid(
                "Resolve the existing Claude sign-in journal first".into(),
            ));
        }
        let profile = self.engine.store.read()?.profile(id)?.clone();
        if profile.app_id != "claude-desktop" {
            return Err(Error::Invalid(
                "Snapshot sign-in is only for Claude Desktop".into(),
            ));
        }
        if !self.engine.running(&profile)?.is_empty() {
            return Err(Error::StillRunning);
        }
        let dirs = self.engine.platform.directories()?;
        let slot = layout::resolve(&layout::claude_candidates(
            Path::new(&dirs.local_app_data),
            Path::new(&dirs.app_data),
        ))?;
        let size = self.engine.platform.tree_bytes(&slot)?;
        if self.engine.platform.free_bytes(&slot)? < size.saturating_mul(2) {
            return Err(Error::Invalid("Sign-in needs at least twice the main slot's size in free disk space. No data was moved.".into()));
        }
        let main_was_running = !self.engine.main_processes("claude-desktop")?.is_empty();
        if main_was_running && !close_main_confirmed {
            return Err(Error::Invalid(
                "Confirm graceful close of your main Claude before starting".into(),
            ));
        }
        let mut journal = Journal {
            schema_version: 1,
            id: ulid::Ulid::generate().to_string(),
            profile_id: id.into(),
            state: State::Preflight,
            slot,
            target: Path::new(&profile.data_dir).join("data"),
            main_was_running,
            close_main_confirmed,
            updated_at: String::new(),
            adoption: Adoption::Preserve,
            restore_only: false,
        };
        self.journals.save(&mut journal)?;
        Ok(journal)
    }
    /// One durable step, intentionally exposed for the crash/restart harness.
    pub fn advance(&self, id: &str) -> Result<Journal> {
        let _lock = self.engine.platform.launch_lock()?;
        let mut j = self.journals.read(id)?;
        let profile = self.validate(&j)?;
        let files = &self.engine.platform;
        match j.state {
            State::Preflight => {
                if !self.engine.main_processes("claude-desktop")?.is_empty()
                    && !j.close_main_confirmed
                {
                    return Err(Error::Invalid(
                        "Main Claude close has not been confirmed".into(),
                    ));
                }
                self.engine.close_main_for_sign_in()?;
                self.set_state(&mut j, State::Stashed)?;
            }
            State::Stashed => {
                if !j.stash().try_exists().map_err(|e| io(j.stash(), e))? {
                    if !self.engine.main_processes("claude-desktop")?.is_empty()
                        && !j.close_main_confirmed
                    {
                        return Err(Error::Invalid(
                            "Confirm close of the newly opened main Claude before continuing"
                                .into(),
                        ));
                    }
                    self.engine.close_main_for_sign_in()?;
                    files.tree_bytes(&j.slot)?;
                    files.rename_data(&j.slot, &j.stash())?;
                } else if j.slot.try_exists().map_err(|e| io(&j.slot, e))? {
                    return Err(Error::Invalid("Both the stash and default slot exist before fresh launch. Restore the main slot first.".into()));
                }
                self.set_state(&mut j, State::AwaitingSignIn)?;
            }
            State::AwaitingSignIn => {
                if !j.stash().try_exists().map_err(|e| io(j.stash(), e))? {
                    return Err(Error::Invalid(
                        "The original Claude stash is missing. No fresh app was launched.".into(),
                    ));
                }
                self.engine.launch_main_for_sign_in()?;
            }
            State::Captured => {
                self.engine.close_main_for_sign_in()?;
                if !j.capture().try_exists().map_err(|e| io(j.capture(), e))? {
                    if !layout::probe(&j.slot)?.live {
                        return Err(Error::Invalid(
                            "The fresh Claude slot is not a recognized live layout".into(),
                        ));
                    }
                    files.tree_bytes(&j.slot)?;
                    files.rename_data(&j.slot, &j.capture())?;
                }
                self.set_state(&mut j, State::Adopting)?;
            }
            State::Adopting => {
                if !self.engine.running(&profile)?.is_empty() {
                    return Err(Error::StillRunning);
                }
                match j.adoption {
                    Adoption::Preserve => {
                        if j.target.try_exists().map_err(|e| io(&j.target, e))? {
                            if j.previous().try_exists().map_err(|e| io(j.previous(), e))? {
                                return Err(Error::Invalid("Both current and previous profile data exist; no directory was overwritten".into()));
                            }
                            files.rename_data(&j.target, &j.previous())?;
                        }
                        j.adoption = Adoption::Copy;
                        self.journals.save(&mut j)?;
                    }
                    Adoption::Copy => {
                        if j.incoming().try_exists().map_err(|e| io(j.incoming(), e))? {
                            let retained = j.incoming().with_file_name(format!(
                                "data.ideck-incomplete-{}",
                                ulid::Ulid::generate()
                            ));
                            files.rename_data(&j.incoming(), &retained)?;
                        }
                        files.copy_data(&j.capture(), &j.incoming())?;
                        j.adoption = Adoption::Activate;
                        self.journals.save(&mut j)?;
                    }
                    Adoption::Activate => {
                        let incoming =
                            j.incoming().try_exists().map_err(|e| io(j.incoming(), e))?;
                        let target = j.target.try_exists().map_err(|e| io(&j.target, e))?;
                        match (incoming, target) {
                            (true, false) => files.rename_data(&j.incoming(), &j.target)?,
                            (false, true) => {} // Rename completed before an interruption.
                            _ => return Err(Error::Invalid(
                                "Profile activation paths are ambiguous; nothing was overwritten"
                                    .into(),
                            )),
                        }
                        self.set_state(&mut j, State::Restoring)?;
                    }
                }
            }
            State::Restoring => self.restore_locked(&mut j)?,
            State::Done | State::Aborted => {}
        }
        Ok(j)
    }
    pub fn resume(&self, id: &str) -> Result<Journal> {
        for _ in 0..12 {
            let current = self.journals.read(id)?;
            if current.state.terminal() {
                return Ok(current);
            }
            if current.state == State::AwaitingSignIn {
                return self.advance(id);
            }
            let result = self.advance(id);
            match result {
                Ok(j) if j.state == State::AwaitingSignIn => return self.advance(id),
                Ok(j) if j.state.terminal() => return Ok(j),
                Ok(_) => {}
                Err(error) => {
                    // An adoption failure always attempts restoration; failure retains the journal.
                    if matches!(current.state, State::Captured | State::Adopting) {
                        let _ = self.restore(id);
                    }
                    return Err(error);
                }
            }
        }
        Err(Error::Invalid(
            "Sign-in recovery exceeded its bounded step count".into(),
        ))
    }
    pub fn finish(&self, id: &str) -> Result<Journal> {
        self.mark_finished(id)?;
        self.resume(id)
    }
    pub fn mark_finished(&self, id: &str) -> Result<()> {
        {
            let _lock = self.engine.platform.launch_lock()?;
            let mut j = self.journals.read(id)?;
            self.validate(&j)?;
            if j.state != State::AwaitingSignIn {
                return Err(Error::Invalid(
                    "This journal is not awaiting sign-in".into(),
                ));
            }
            self.set_state(&mut j, State::Captured)?;
        }
        Ok(())
    }
    pub fn restore(&self, id: &str) -> Result<Journal> {
        let _lock = self.engine.platform.launch_lock()?;
        let mut j = self.journals.read(id)?;
        self.validate(&j)?;
        if j.state.terminal() {
            return Ok(j);
        }
        j.restore_only = true;
        self.set_state(&mut j, State::Restoring)?;
        self.restore_locked(&mut j)?;
        Ok(j)
    }
    fn restore_locked(&self, j: &mut Journal) -> Result<()> {
        let files = &self.engine.platform;
        if j.stash().try_exists().map_err(|e| io(j.stash(), e))? {
            self.engine.close_main_for_sign_in()?;
            if j.slot.try_exists().map_err(|e| io(&j.slot, e))? {
                files.rename_data(&j.slot, &j.cancelled())?;
            }
            files.rename_data(&j.stash(), &j.slot)?;
        }
        if !j.slot.try_exists().map_err(|e| io(&j.slot, e))? {
            return Err(Error::Invalid(
                "The main Claude slot is still missing. Recovery remains pending.".into(),
            ));
        }
        if j.restore_only
            && !j.target.try_exists().map_err(|e| io(&j.target, e))?
            && j.previous().try_exists().map_err(|e| io(j.previous(), e))?
        {
            files.rename_data(&j.previous(), &j.target)?;
        }
        let dirs = self.engine.platform.directories()?;
        if layout::resolve(&layout::claude_candidates(
            Path::new(&dirs.local_app_data),
            Path::new(&dirs.app_data),
        ))? != j.slot
        {
            return Err(Error::Invalid("Claude's active layout changed during sign-in. The original is restored; review the layout before relaunching.".into()));
        }
        if !j.restore_only || j.adoption != Adoption::Preserve {
            self.engine.store.transaction(|state| {
                let profile = state
                    .profiles
                    .iter_mut()
                    .find(|p| p.id == j.profile_id)
                    .ok_or_else(|| Error::NotFound("Profile".into()))?;
                profile.sign_in_state = SignInState::Unknown; // Only the user confirms the account.
                Ok(())
            })?;
        }
        self.set_state(
            j,
            if j.restore_only {
                State::Aborted
            } else {
                State::Done
            },
        )?;
        if j.main_was_running {
            self.engine.launch_main_for_sign_in()?;
        }
        Ok(())
    }
}

/// Metadata event heuristic, never account verification.
#[derive(Default)]
pub struct MetadataActivity {
    writes: usize,
    last_write_ms: u64,
}
impl MetadataActivity {
    pub fn record(&mut self, now_ms: u64) {
        self.writes += 1;
        self.last_write_ms = now_ms;
    }
    pub fn ready(&self, now_ms: u64) -> bool {
        self.writes >= 2 && now_ms.saturating_sub(self.last_write_ms) >= 3_000
    }
}
