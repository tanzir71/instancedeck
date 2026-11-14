//! Claude layout probing reads directory names and file metadata only.
use crate::{Error, Result, error::io};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LayoutProbe {
    pub path: PathBuf,
    pub live: bool,
    pub relocated: bool,
}

pub fn probe(path: &Path) -> Result<LayoutProbe> {
    let entries = match std::fs::read_dir(path) {
        Ok(entries) => entries,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(LayoutProbe {
                path: path.into(),
                live: false,
                relocated: false,
            });
        }
        Err(e) => return Err(io(path, e)),
    };
    let mut live_marker = false;
    let mut relocated = false;
    for entry in entries {
        let entry = entry.map_err(|e| io(path, e))?;
        let name = entry.file_name().to_string_lossy().to_lowercase();
        if name.starts_with(".user-data-left") {
            relocated = true;
        }
        if matches!(name.as_str(), "local state" | "preferences" | "network") {
            live_marker = true;
        }
    }
    Ok(LayoutProbe {
        path: path.into(),
        live: live_marker && !relocated,
        relocated,
    })
}

pub fn claude_candidates(local_app_data: &Path, app_data: &Path) -> Vec<PathBuf> {
    vec![
        local_app_data
            .join("Claude-Data")
            .join("claude_pzs8sxrjxfjjc"),
        local_app_data
            .join("Packages")
            .join("Claude_pzs8sxrjxfjjc")
            .join("LocalCache")
            .join("Roaming")
            .join("Claude"),
        app_data.join("Claude"),
    ]
}

pub fn resolve(candidates: &[PathBuf]) -> Result<PathBuf> {
    let probes = candidates
        .iter()
        .map(|p| probe(p))
        .collect::<Result<Vec<_>>>()?;
    let live: Vec<_> = probes.iter().filter(|p| p.live).collect();
    match live.as_slice() {
        [slot] => Ok(slot.path.clone()),
        [] => Err(Error::Invalid("Claude's live default data layout is unknown. No data was moved.".into())),
        _ => Err(Error::Invalid("Multiple Claude data slots look live. Sign-in is blocked until the active slot is confirmed.".into())),
    }
}
