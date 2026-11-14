use crate::WindowsPlatform;
use ideck_core::{
    Error, Result,
    error::io,
    signin::{MetadataActivity, SignInFiles},
};
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::{
    fs,
    os::windows::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use windows::{
    Win32::Storage::FileSystem::{
        CopyFileW, GetDiskFreeSpaceExW, MOVEFILE_WRITE_THROUGH, MoveFileExW,
    },
    core::HSTRING,
};

fn check(path: &Path) -> Result<()> {
    for part in path.ancestors() {
        match fs::symlink_metadata(part) {
            Ok(m) if m.file_attributes() & 0x400 != 0 => {
                return Err(Error::Invalid(
                    "Sign-in cannot traverse a link or reparse point".into(),
                ));
            }
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(io(part, e)),
        }
    }
    Ok(())
}
fn tree_bytes(path: &Path) -> Result<u64> {
    check(path)?;
    let m = fs::symlink_metadata(path).map_err(|e| io(path, e))?;
    if m.is_file() {
        return Ok(m.len());
    }
    if !m.is_dir() {
        return Err(Error::Invalid(
            "Sign-in data must contain only regular files and directories".into(),
        ));
    }
    let mut size = 0u64;
    for entry in fs::read_dir(path).map_err(|e| io(path, e))? {
        let entry = entry.map_err(|e| io(path, e))?;
        size = size.saturating_add(tree_bytes(&entry.path())?);
    }
    Ok(size)
}
fn copy_tree(from: &Path, to: &Path) -> Result<()> {
    check(from)?;
    check(to)?;
    let m = fs::symlink_metadata(from).map_err(|e| io(from, e))?;
    if m.is_file() {
        // CopyFileW performs the opaque transfer. No target file is opened or read by Rust.
        return unsafe { CopyFileW(&HSTRING::from(from), &HSTRING::from(to), true) }
            .map_err(|e| Error::Platform(e.to_string()));
    }
    if !m.is_dir() {
        return Err(Error::Invalid(
            "Unsupported file kind in sign-in data".into(),
        ));
    }
    fs::create_dir(to).map_err(|e| io(to, e))?;
    for entry in fs::read_dir(from).map_err(|e| io(from, e))? {
        let entry = entry.map_err(|e| io(from, e))?;
        copy_tree(&entry.path(), &to.join(entry.file_name()))?;
    }
    Ok(())
}
impl SignInFiles for WindowsPlatform {
    fn tree_bytes(&self, path: &Path) -> Result<u64> {
        tree_bytes(path)
    }
    fn free_bytes(&self, path: &Path) -> Result<u64> {
        check(path)?;
        let mut available = 0;
        unsafe { GetDiskFreeSpaceExW(&HSTRING::from(path), Some(&mut available), None, None) }
            .map_err(|e| Error::Platform(e.to_string()))?;
        Ok(available)
    }
    fn rename_data(&self, from: &Path, to: &Path) -> Result<()> {
        check(from)?;
        check(to)?;
        let mut last = None;
        for attempt in 0..10 {
            if to.try_exists().map_err(|e| io(to, e))? {
                return Err(Error::Invalid(
                    "The destination already exists; data was not overwritten".into(),
                ));
            }
            // Omitting REPLACE_EXISTING enforces no-overwrite inside the OS,
            // including a destination created after the metadata check.
            match unsafe {
                MoveFileExW(
                    &HSTRING::from(from),
                    &HSTRING::from(to),
                    MOVEFILE_WRITE_THROUGH,
                )
            } {
                Ok(()) => return Ok(()),
                Err(e) => last = Some(std::io::Error::from_raw_os_error(e.code().0 & 0xffff)),
            }
            if attempt < 9 {
                std::thread::sleep(Duration::from_millis(1_666));
            }
        }
        Err(io(from, last.expect("ten rename attempts")))
    }
    fn copy_data(&self, from: &Path, to: &Path) -> Result<()> {
        tree_bytes(from)?;
        check(to)?;
        let parent = to
            .parent()
            .ok_or_else(|| Error::Invalid("Missing profile parent".into()))?;
        fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
        if self.free_bytes(parent)? < tree_bytes(from)? {
            return Err(Error::Invalid(
                "The profile drive has insufficient free space; the capture remains retained"
                    .into(),
            ));
        }
        copy_tree(from, to)
    }
    fn sign_in_hint(&self, slot: &Path, session: &str) -> Result<bool> {
        let mut watch = self
            .sign_in_watch
            .lock()
            .map_err(|_| Error::Platform("Metadata watcher lock failed".into()))?;
        if watch
            .as_ref()
            .is_none_or(|w| w.slot != slot || w.session != session)
        {
            *watch = Some(MetadataWatch::new(slot, session)?);
        }
        watch.as_mut().expect("watch created").sample()
    }
}

pub(crate) struct MetadataWatch {
    session: String,
    slot: PathBuf,
    paths: Vec<PathBuf>,
    metadata: Vec<Option<(u64, SystemTime)>>,
    _watcher: RecommendedWatcher,
    events: mpsc::Receiver<notify::Result<Event>>,
    activity: MetadataActivity,
}
impl MetadataWatch {
    fn new(slot: &Path, session: &str) -> Result<Self> {
        let (sender, events) = mpsc::sync_channel(512);
        let mut watcher = notify::recommended_watcher(move |event| {
            let _ = sender.try_send(event);
        })
        .map_err(|e| Error::Platform(e.to_string()))?;
        let parent = slot
            .parent()
            .ok_or_else(|| Error::Invalid("Missing Claude slot parent".into()))?;
        watcher
            .watch(parent, RecursiveMode::Recursive)
            .map_err(|e| Error::Platform(e.to_string()))?;
        let paths = [
            "Network/Cookies",
            "Network/Cookies-journal",
            "Network/Cookies-wal",
            "config.json",
        ]
        .iter()
        .map(|name| slot.join(name))
        .collect::<Vec<_>>();
        let metadata = paths
            .iter()
            .map(|path| {
                fs::metadata(path)
                    .ok()
                    .and_then(|m| m.modified().ok().map(|t| (m.len(), t)))
            })
            .collect();
        Ok(Self {
            session: session.into(),
            slot: slot.into(),
            paths,
            metadata,
            _watcher: watcher,
            events,
            activity: MetadataActivity::default(),
        })
    }
    fn sample(&mut self) -> Result<bool> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        for event in self.events.try_iter() {
            let event = event.map_err(|e| Error::Platform(e.to_string()))?;
            if matches!(event.kind, EventKind::Create(_) | EventKind::Modify(_))
                && event.paths.iter().any(|p| self.paths.contains(p))
            {
                self.activity.record(now);
            }
        }
        for (index, path) in self.paths.iter().enumerate() {
            let value = fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok().map(|t| (m.len(), t)));
            if value != self.metadata[index] {
                self.activity.record(now);
                self.metadata[index] = value;
            }
        }
        Ok(self.activity.ready(now))
    }
}
