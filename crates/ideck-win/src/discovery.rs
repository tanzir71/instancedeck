//! Squirrel-style version folder discovery uses names and metadata only.
use ideck_core::{Result, error::io};
use std::path::{Path, PathBuf};

pub fn versioned_executable(executable: &Path, prefix: &str) -> Result<Option<(PathBuf, String)>> {
    let Some(root) = executable.parent() else {
        return Ok(None);
    };
    let Some(file_name) = executable.file_name() else {
        return Ok(None);
    };
    let entries = match std::fs::read_dir(root) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(io(root, error)),
    };
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| io(root, error))?;
        let name = entry.file_name().to_string_lossy().into_owned();
        let Some(version) = name.strip_prefix(prefix) else {
            continue;
        };
        let Ok(numbers) = version
            .split('.')
            .map(str::parse::<u64>)
            .collect::<std::result::Result<Vec<_>, _>>()
        else {
            continue;
        };
        let target = entry.path().join(file_name);
        if target.is_file() {
            candidates.push((numbers, target, version.to_owned()));
        }
    }
    candidates.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(candidates
        .pop()
        .map(|(_, target, version)| (target, version)))
}
