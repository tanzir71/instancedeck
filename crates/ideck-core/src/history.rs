//! Opt-in sessions-only sharing. No session contents are read or merged.
use crate::{Engine, Error, Platform, Profile, Result, error::io, paths};
use serde::{Deserialize, Serialize};
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    Missing,
    Directory,
    Junction(PathBuf),
}
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Journal {
    schema_version: u32,
    profile_id: String,
    data_dir: String,
    generation: String,
    desired: bool,
    complete: bool,
}
fn journal_path(root: &Path, id: &str) -> PathBuf {
    root.join("history-journal").join(format!("{id}.json"))
}
fn read(root: &Path, profile: &Profile) -> Result<Option<Journal>> {
    let path = journal_path(root, &profile.id);
    let metadata = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(io(&path, e)),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 16 * 1024 {
        return Err(Error::Invalid("Invalid shared-history journal".into()));
    }
    let journal: Journal =
        serde_json::from_str(&fs::read_to_string(&path).map_err(|e| io(&path, e))?)?;
    if journal.schema_version != 1
        || journal.profile_id != profile.id
        || journal.data_dir != profile.data_dir
        || ulid::Ulid::from_string(&journal.generation).is_err()
    {
        return Err(Error::Invalid(
            "Shared-history journal does not match the saved profile".into(),
        ));
    }
    Ok(Some(journal))
}
fn write(root: &Path, journal: &Journal) -> Result<()> {
    let path = journal_path(root, &journal.profile_id);
    let parent = path.parent().expect("journal parent");
    fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
    let staged = parent.join(format!("pending-{}.json", ulid::Ulid::generate()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staged)
        .map_err(|e| io(&staged, e))?;
    file.write_all(serde_json::to_string_pretty(journal)?.as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| io(&staged, e))?;
    drop(file);
    if path.try_exists().map_err(|e| io(&path, e))? {
        let backup = parent.join(format!(
            "retained-{}-{}.json",
            journal.profile_id,
            ulid::Ulid::generate()
        ));
        fs::copy(&path, &backup).map_err(|e| io(&backup, e))?;
    }
    fs::rename(&staged, &path).map_err(|e| io(&path, e))
}
fn paths_for(root: &Path, profile: &Profile, journal: &Journal) -> (PathBuf, PathBuf, PathBuf) {
    let home = Path::new(&profile.data_dir).join("codex-home");
    (
        home.join("sessions"),
        home.join(format!("sessions.ideck-private-{}", journal.generation)),
        root.join("shared-codex-sessions"),
    )
}
pub fn guard<P: Platform>(engine: &Engine<P>, profile: &Profile) -> Result<()> {
    if profile.app_id != "codex" {
        return Ok(());
    }
    guard_pending(engine.store.root(), profile)?;
    engine.platform.check_history(profile, engine.store.root())
}
pub fn guard_pending(root: &Path, profile: &Profile) -> Result<()> {
    if read(root, profile)?.is_some_and(|j| !j.complete) {
        return Err(Error::Invalid("Shared-history change is incomplete. Resume sharing or restore private history in profile details before launching.".into()));
    }
    Ok(())
}
pub fn configure<P: Platform>(engine: &Engine<P>, id: &str, enabled: bool) -> Result<()> {
    let _lock = engine.platform.launch_lock()?;
    let profile = engine.store.read()?.profile(id)?.clone();
    if profile.app_id != "codex" {
        return Err(Error::Invalid("Only Codex sessions can be shared".into()));
    }
    if !engine.running(&profile)?.is_empty() {
        return Err(Error::StillRunning);
    }
    let expected = format!("codex\\{}", profile.id);
    if !paths::normalize_windows(&profile.data_dir)
        .ends_with(&format!("\\{}", expected.to_lowercase()))
    {
        return Err(Error::Invalid(
            "Shared history requires the saved codex/profile-id folder".into(),
        ));
    }
    let previous = read(engine.store.root(), &profile)?;
    let mut journal = match previous {
        Some(j) if !j.complete || j.desired => j,
        _ => Journal {
            schema_version: 1,
            profile_id: profile.id.clone(),
            data_dir: profile.data_dir.clone(),
            generation: ulid::Ulid::generate().to_string(),
            desired: enabled,
            complete: false,
        },
    };
    journal.desired = enabled;
    journal.complete = false;
    write(engine.store.root(), &journal)?; // durable intent precedes any app-directory change
    let (sessions, private, shared) = paths_for(engine.store.root(), &profile, &journal);
    let current = engine.platform.history_kind(&sessions)?;
    let backup = engine.platform.history_kind(&private)?;
    if enabled {
        match engine.platform.history_kind(&shared)? {
            Kind::Missing => engine
                .platform
                .ensure_directory(&shared.to_string_lossy())?,
            Kind::Directory => {}
            Kind::Junction(_) => {
                return Err(Error::Invalid(
                    "The shared sessions target cannot be a junction".into(),
                ));
            }
        }
        match current {
            Kind::Directory if backup==Kind::Missing=>engine.platform.move_history(&sessions,&private)?,
            Kind::Directory=>return Err(Error::Invalid("Both private backup and current sessions exist. Restore private history to preserve both before retrying.".into())),
            Kind::Junction(ref target) if paths::normalize_windows(&target.to_string_lossy())==paths::normalize_windows(&shared.to_string_lossy())=>{},
            Kind::Junction(_)=>return Err(Error::Invalid("Sessions points to an unexpected target; nothing was moved".into())),
            Kind::Missing=>{},
        }
        if engine.platform.history_kind(&sessions)? == Kind::Missing {
            engine
                .platform
                .ensure_directory(&sessions.parent().expect("home").to_string_lossy())?;
            engine
                .platform
                .create_history_junction(&shared, &sessions)?;
        }
    } else {
        // Retain the junction outside the profile tree, on the same volume. Never
        // remove a link or expose its shared target to a profile recycle operation.
        let retire = |source: &Path| -> Result<()> {
            let parent = Path::new(&profile.data_dir)
                .parent()
                .expect("app parent")
                .join(".ideck-retained-history");
            engine
                .platform
                .ensure_directory(&parent.to_string_lossy())?;
            engine.platform.move_history(
                source,
                &parent.join(format!("{}-{}", profile.id, ulid::Ulid::generate())),
            )
        };
        match current {
            Kind::Junction(target)
                if paths::normalize_windows(&target.to_string_lossy())
                    == paths::normalize_windows(&shared.to_string_lossy()) =>
            {
                retire(&sessions)?
            }
            Kind::Junction(_) => {
                return Err(Error::Invalid(
                    "Unexpected sessions junction; nothing was moved".into(),
                ));
            }
            Kind::Directory if backup == Kind::Directory => retire(&sessions)?,
            _ => {}
        }
        if engine.platform.history_kind(&sessions)? == Kind::Missing {
            match backup {
                Kind::Directory => engine.platform.move_history(&private, &sessions)?,
                Kind::Missing => engine
                    .platform
                    .ensure_directory(&sessions.to_string_lossy())?,
                Kind::Junction(_) => {
                    return Err(Error::Invalid(
                        "Private sessions backup cannot be a junction".into(),
                    ));
                }
            }
        }
    }
    engine.store.transaction(|state| {
        let saved = state
            .profiles
            .iter_mut()
            .find(|p| p.id == id)
            .ok_or_else(|| Error::NotFound("Profile".into()))?;
        if saved.data_dir != profile.data_dir {
            return Err(Error::Invalid("Profile moved during history change".into()));
        }
        saved.options.shared_codex_history = enabled;
        Ok(())
    })?;
    journal.complete = true;
    write(engine.store.root(), &journal)
}
