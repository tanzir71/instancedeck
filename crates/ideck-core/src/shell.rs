//! Shell plans contain only manager metadata, never target app binaries.
use crate::{Error, Profile, Result, Settings, paths};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "kebab-case")]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub enum ShortcutLocation {
    Desktop,
    StartMenu,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityPlan {
    pub app_user_model_id: String,
    pub launcher: String,
    pub args: Vec<String>,
    pub display_name: String,
    pub icon: String,
}
impl IdentityPlan {
    pub fn new(profile: &Profile, launcher: &str, icon: &str) -> Result<Self> {
        profile.validate()?;
        paths::validate_absolute(launcher)?;
        paths::validate_absolute(icon)?;
        Ok(Self {
            app_user_model_id: format!("InstanceDeck.{}.{}", profile.app_id, profile.id),
            launcher: launcher.into(),
            args: vec!["launch".into(), profile.id.clone()],
            display_name: profile.name.clone(),
            icon: icon.into(),
        })
    }
    pub fn shortcut_filename(&self) -> String {
        let name: String = self
            .display_name
            .chars()
            .filter(|c| !c.is_control() && !"<>:\"/\\|?*".contains(*c))
            .take(60)
            .collect();
        format!(
            "{} — {}.lnk",
            name.trim().trim_end_matches('.'),
            self.args[1]
        )
    }
}

pub trait ShellOperations: Send + Sync {
    fn create_shortcuts(
        &self,
        profile: &Profile,
        locations: &[ShortcutLocation],
    ) -> Result<Vec<String>>;
    fn sync_window_identity(&self, _: Vec<IdentityBinding>) -> Result<BTreeMap<String, String>> {
        Ok(BTreeMap::new())
    }
    fn stop_window_identity(&self) {}
}
#[derive(Clone)]
pub struct IdentityBinding {
    pub profile: Profile,
    pub processes: Vec<crate::process::ProcessInfo>,
}
/// Native manager hotkeys/autostart are injected, so service tests never change
/// the user's registrations. Autostart changes only after an explicit setting edit.
pub trait ShellRuntime: Send + Sync {
    fn apply(
        &self,
        settings: &Settings,
        profiles: &[Profile],
        change_autostart: bool,
    ) -> Result<()>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowCandidate {
    pub handle: usize,
    pub pid: u32,
    pub process_started_at: u64,
}
/// Window handles are never trusted without their current PID and creation time.
/// The native adapter supplies only eligible, visible top-level windows.
#[derive(Default)]
pub struct StableWindows {
    seen: BTreeMap<usize, (WindowCandidate, u64)>,
}
impl StableWindows {
    pub fn observe(&mut self, now_ms: u64, candidates: &[WindowCandidate]) -> Vec<WindowCandidate> {
        self.seen.retain(|handle, _| {
            candidates
                .iter()
                .any(|c| c.handle == *handle && c.pid != 0 && c.process_started_at != 0)
        });
        let mut ready = Vec::new();
        for candidate in candidates
            .iter()
            .filter(|c| c.handle != 0 && c.pid != 0 && c.process_started_at != 0)
        {
            let entry = self
                .seen
                .entry(candidate.handle)
                .or_insert((*candidate, now_ms));
            if entry.0 != *candidate {
                *entry = (*candidate, now_ms);
            }
            if now_ms.saturating_sub(entry.1) >= 750 {
                ready.push(*candidate);
            }
        }
        ready
    }
}

pub fn normalize_hotkey(input: &str) -> Result<String> {
    if input.len() > 80 || input.chars().any(char::is_control) {
        return Err(Error::Invalid(
            "Hotkeys must be at most 80 characters without control characters".into(),
        ));
    }
    let mut ctrl = false;
    let mut alt = false;
    let mut shift = false;
    let mut super_key = false;
    let mut key = None;
    for part in input.split('+').map(str::trim) {
        match part.to_ascii_lowercase().as_str() {
            "ctrl" | "control" if !ctrl => ctrl = true,
            "alt" if !alt => alt = true,
            "shift" if !shift => shift = true,
            "super" | "win" | "meta" if !super_key => super_key = true,
            _ if key.is_none() => {
                let upper = part.to_ascii_uppercase();
                key = Some(
                    if upper.len() == 1 && upper.as_bytes()[0].is_ascii_alphanumeric() {
                        upper
                    } else if let Some(f) = upper
                        .strip_prefix('F')
                        .and_then(|v| v.parse::<u8>().ok())
                        .filter(|n| (1..=24).contains(n))
                    {
                        format!("F{f}")
                    } else if upper == "SPACE" {
                        "Space".into()
                    } else if matches!(upper.as_str(), "BACKQUOTE" | "`") {
                        "Backquote".into()
                    } else {
                        return Err(Error::Invalid(
                            "Use modifiers with a letter, number, F1–F24, Space or Backquote"
                                .into(),
                        ));
                    },
                );
            }
            _ => {
                return Err(Error::Invalid(
                    "A hotkey must contain one key and no repeated modifiers".into(),
                ));
            }
        }
    }
    if !(ctrl || alt || super_key) || key.is_none() {
        return Err(Error::Invalid(
            "Global hotkeys need Ctrl, Alt or Win and one key".into(),
        ));
    }
    let mut parts = Vec::new();
    if ctrl {
        parts.push("Ctrl".into());
    }
    if alt {
        parts.push("Alt".into());
    }
    if shift {
        parts.push("Shift".into());
    }
    if super_key {
        parts.push("Super".into());
    }
    parts.push(key.expect("validated key"));
    Ok(parts.join("+"))
}
pub fn validate_hotkeys(settings: &Settings, profiles: &[Profile]) -> Result<()> {
    let mut used = BTreeSet::from([normalize_hotkey(&settings.switcher_hotkey)?]);
    for profile in profiles {
        if let Some(hotkey) = &profile.hotkey
            && !used.insert(normalize_hotkey(hotkey)?)
        {
            return Err(Error::Invalid(
                "Each global hotkey must be unique, including the switcher".into(),
            ));
        }
    }
    Ok(())
}
