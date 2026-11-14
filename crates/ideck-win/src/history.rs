use crate::WindowsPlatform;
use ideck_core::{Error, Profile, Result, error::io, history::Kind, paths};
use std::{fs, os::windows::fs::MetadataExt, path::Path};
use windows::{
    Win32::Storage::FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
    core::HSTRING,
};

fn check_parents(path: &Path) -> Result<()> {
    for parent in path.ancestors().skip(1) {
        match fs::symlink_metadata(parent) {
            Ok(m) if m.file_attributes() & 0x400 != 0 => {
                return Err(Error::Invalid(
                    "History paths cannot traverse reparse points".into(),
                ));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(parent, e)),
        }
    }
    Ok(())
}
pub(crate) fn kind(path: &Path) -> Result<Kind> {
    check_parents(path)?;
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Kind::Missing),
        Err(e) => Err(io(path, e)),
        Ok(m) if m.file_attributes() & 0x400 != 0 => {
            if !junction::exists(path).map_err(|e| io(path, e))? {
                return Err(Error::Invalid(
                    "History path is an unsupported reparse point".into(),
                ));
            }
            Ok(Kind::Junction(
                junction::get_target(path).map_err(|e| io(path, e))?,
            ))
        }
        Ok(m) if m.is_dir() => Ok(Kind::Directory),
        Ok(_) => Err(Error::Invalid("Sessions must be a directory".into())),
    }
}
pub(crate) fn check(profile: &Profile, root: &Path) -> Result<()> {
    if profile.app_id != "codex" {
        return Ok(());
    }
    let sessions = Path::new(&profile.data_dir)
        .join("codex-home")
        .join("sessions");
    match (profile.options.shared_codex_history,kind(&sessions)?) {
        (true,Kind::Junction(target)) if paths::normalize_windows(&target.to_string_lossy()) == paths::normalize_windows(&root.join("shared-codex-sessions").to_string_lossy())=>Ok(()),
        (false,Kind::Missing|Kind::Directory)=>Ok(()),
        _=>Err(Error::Invalid("Sessions layout does not match the sharing preference. Restore private history or resume sharing before launch.".into())),
    }
}
pub(crate) fn move_path(from: &Path, to: &Path) -> Result<()> {
    check_parents(from)?;
    check_parents(to)?;
    let mut last = None;
    for attempt in 0..10 {
        match unsafe {
            MoveFileExW(
                &HSTRING::from(from),
                &HSTRING::from(to),
                MOVEFILE_WRITE_THROUGH,
            )
        } {
            Ok(()) => return Ok(()),
            Err(e) => last = Some(e),
        }
        if attempt < 9 {
            std::thread::sleep(std::time::Duration::from_millis(1500));
        }
    }
    Err(Error::Platform(format!(
        "History rename did not complete; all paths retained: {}",
        last.expect("rename error")
    )))
}
pub(crate) fn create(target: &Path, link: &Path) -> Result<()> {
    if kind(target)? != Kind::Directory || kind(link)? != Kind::Missing {
        return Err(Error::Invalid(
            "Junction requires a regular target and unused link path".into(),
        ));
    }
    junction::create(target, link).map_err(|e| io(link, e))?;
    if !matches!(kind(link)?,Kind::Junction(ref found) if paths::normalize_windows(&found.to_string_lossy())==paths::normalize_windows(&target.to_string_lossy()))
    {
        return Err(Error::Platform(
            "Junction target verification failed; paths retained".into(),
        ));
    }
    Ok(())
}
// These wrappers keep the filesystem surface in ideck-win behind the core trait.
impl WindowsPlatform {
    pub(crate) fn history_check(&self, p: &Profile, root: &Path) -> Result<()> {
        check(p, root)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ideck_core::ProfileColor;
    #[test]
    fn native_sessions_junction_roundtrip_retains_private_and_shared_fixtures() {
        let root = tempfile::tempdir().unwrap();
        let profile = Profile::new(
            "codex",
            "Fixture",
            &root.path().join("Profiles Zoë 日本語 with spaces"),
            ProfileColor::Iris,
        )
        .unwrap();
        let sessions = Path::new(&profile.data_dir)
            .join("codex-home")
            .join("sessions");
        let private = sessions.with_file_name("sessions.retained-fixture");
        let shared = root.path().join("shared-codex-sessions");
        fs::create_dir_all(&sessions).unwrap();
        fs::create_dir_all(&shared).unwrap();
        fs::write(
            sessions.join("synthetic-session.txt"),
            b"private fixture, no account data",
        )
        .unwrap();
        fs::write(
            shared.join("synthetic-session.txt"),
            b"shared fixture, no account data",
        )
        .unwrap();
        move_path(&sessions, &private).unwrap();
        create(&shared, &sessions).unwrap();
        assert!(matches!(kind(&sessions).unwrap(), Kind::Junction(_)));
        assert!(check(&profile, root.path()).is_err());
        let mut enabled = profile.clone();
        enabled.options.shared_codex_history = true;
        check(&enabled, root.path()).unwrap();
        assert!(kind(&sessions.join("nested")).is_err());
        let retired = root.path().join("retained-link");
        move_path(&sessions, &retired).unwrap();
        move_path(&private, &sessions).unwrap();
        assert_eq!(
            fs::read(sessions.join("synthetic-session.txt")).unwrap(),
            b"private fixture, no account data"
        );
        assert_eq!(
            fs::read(shared.join("synthetic-session.txt")).unwrap(),
            b"shared fixture, no account data"
        );
        assert!(matches!(kind(&retired).unwrap(), Kind::Junction(_)));
        check(&profile, root.path()).unwrap();
    }
}
