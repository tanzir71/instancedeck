use crate::{WindowsPlatform, command, icons};
use ideck_core::{
    Error, Profile, Result,
    error::io,
    shell::{IdentityPlan, ShellOperations, ShortcutLocation},
};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};
use windows::{
    Win32::{
        Storage::{
            EnhancedStorage::{
                PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand,
                PKEY_AppUserModel_RelaunchDisplayNameResource,
                PKEY_AppUserModel_RelaunchIconResource,
            },
            FileSystem::{MOVEFILE_WRITE_THROUGH, MoveFileExW},
        },
        System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
            CoTaskMemFree, CoUninitialize, IPersistFile, STGM_READWRITE,
            StructuredStorage::PROPVARIANT,
        },
        UI::Shell::{
            FOLDERID_Desktop, FOLDERID_Programs, IShellLinkW, KF_FLAG_DEFAULT,
            PropertiesSystem::IPropertyStore, SHGetKnownFolderPath, ShellLink,
        },
    },
    core::{HSTRING, Interface},
};

pub(crate) struct ComScope;
impl ComScope {
    pub(crate) fn new() -> Result<Self> {
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok() }.map_err(win_error)?;
        Ok(Self)
    }
}
impl Drop for ComScope {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}
pub(crate) fn win_error(error: windows::core::Error) -> Error {
    Error::Platform(error.to_string())
}
pub(crate) fn no_replace_rename(from: &Path, to: &Path) -> Result<()> {
    unsafe {
        MoveFileExW(
            &HSTRING::from(from),
            &HSTRING::from(to),
            MOVEFILE_WRITE_THROUGH,
        )
    }
    .map_err(win_error)
}
pub fn relaunch_command(plan: &IdentityPlan) -> String {
    std::iter::once(&plan.launcher)
        .chain(plan.args.iter())
        .map(|value| command::quote_windows(value))
        .collect::<Vec<_>>()
        .join(" ")
}
pub(crate) fn set_properties(store: &IPropertyStore, plan: &IdentityPlan) -> Result<()> {
    for (key, value) in [
        (PKEY_AppUserModel_ID, plan.app_user_model_id.as_str()),
        (
            PKEY_AppUserModel_RelaunchCommand,
            relaunch_command(plan).as_str(),
        ),
        (
            PKEY_AppUserModel_RelaunchDisplayNameResource,
            plan.display_name.as_str(),
        ),
        (PKEY_AppUserModel_RelaunchIconResource, plan.icon.as_str()),
    ] {
        unsafe { store.SetValue(&key, &PROPVARIANT::from(value)) }.map_err(win_error)?;
    }
    unsafe { store.Commit() }.map_err(win_error)
}
pub fn verify_shortcut(path: &Path, plan: &IdentityPlan) -> Result<()> {
    let _scope = ComScope::new()?;
    let link: IShellLinkW =
        unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.map_err(win_error)?;
    let persist: IPersistFile = link.cast().map_err(win_error)?;
    // Read/write load is intentional: persist/reload is part of the shell recipe.
    unsafe { persist.Load(&HSTRING::from(path), STGM_READWRITE) }.map_err(win_error)?;
    let store: IPropertyStore = link.cast().map_err(win_error)?;
    for (key, expected) in [
        (PKEY_AppUserModel_ID, plan.app_user_model_id.as_str()),
        (
            PKEY_AppUserModel_RelaunchCommand,
            relaunch_command(plan).as_str(),
        ),
        (
            PKEY_AppUserModel_RelaunchDisplayNameResource,
            plan.display_name.as_str(),
        ),
        (PKEY_AppUserModel_RelaunchIconResource, plan.icon.as_str()),
    ] {
        let actual = unsafe { store.GetValue(&key) }
            .map_err(win_error)?
            .to_string();
        if actual != expected {
            return Err(Error::Platform(
                "Shortcut properties did not survive reload".into(),
            ));
        }
    }
    Ok(())
}
pub fn write_shortcut(path: &Path, plan: &IdentityPlan) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| Error::Invalid("Shortcut needs a parent folder".into()))?;
    fs::create_dir_all(parent).map_err(|e| io(parent, e))?;
    let pending = parent.join(format!("InstanceDeck-pending-{}.lnk", ulid_id()));
    {
        let _scope = ComScope::new()?;
        let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
            .map_err(win_error)?;
        unsafe {
            link.SetPath(&HSTRING::from(&plan.launcher))
                .map_err(win_error)?;
            link.SetArguments(&HSTRING::from(
                plan.args
                    .iter()
                    .map(|v| command::quote_windows(v))
                    .collect::<Vec<_>>()
                    .join(" "),
            ))
            .map_err(win_error)?;
            link.SetDescription(&HSTRING::from(&plan.display_name))
                .map_err(win_error)?;
            link.SetIconLocation(&HSTRING::from(&plan.icon), 0)
                .map_err(win_error)?;
        }
        let store: IPropertyStore = link.cast().map_err(win_error)?;
        set_properties(&store, plan)?;
        let persist: IPersistFile = link.cast().map_err(win_error)?;
        unsafe { persist.Save(&HSTRING::from(pending.as_path()), true) }.map_err(win_error)?;
    }
    verify_shortcut(&pending, plan)?;
    if path.try_exists().map_err(|e| io(path, e))? {
        let backup = parent.join(format!(
            "{}.ideck-backup-{}",
            path.file_name().unwrap_or_default().to_string_lossy(),
            ulid_id()
        ));
        no_replace_rename(path, &backup)?;
    }
    no_replace_rename(&pending, path)?;
    verify_shortcut(path, plan)
}
// Own metadata suffix; no target app state is consulted.
fn ulid_id() -> String {
    static NONCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    format!(
        "{}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos(),
        NONCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}
pub fn ensure_icon(state_root: &Path, profile: &Profile) -> Result<PathBuf> {
    let color = serde_json::to_value(profile.color)?;
    let dir = state_root.join("icons");
    fs::create_dir_all(&dir).map_err(|e| io(&dir, e))?;
    let path = dir.join(format!(
        "{}-{}.ico",
        profile.id,
        color.as_str().expect("color string")
    ));
    if path.try_exists().map_err(|e| io(&path, e))? {
        return Ok(path);
    }
    let pending = dir.join(format!("pending-{}.ico", ulid_id()));
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&pending)
        .map_err(|e| io(&pending, e))?;
    file.write_all(&icons::encode_profile_icon(profile.color)?)
        .and_then(|_| file.sync_all())
        .map_err(|e| io(&pending, e))?;
    drop(file);
    no_replace_rename(&pending, &path)?;
    Ok(path)
}
fn known_folder(id: &windows::core::GUID) -> Result<PathBuf> {
    let raw = unsafe { SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None) }.map_err(win_error)?;
    let value = unsafe { raw.to_string() }.map_err(|error| Error::Platform(error.to_string()));
    unsafe { CoTaskMemFree(Some(raw.0.cast())) };
    value.map(PathBuf::from)
}
impl ShellOperations for WindowsPlatform {
    fn create_shortcuts(
        &self,
        profile: &Profile,
        locations: &[ShortcutLocation],
    ) -> Result<Vec<String>> {
        profile.validate()?;
        let launcher = std::env::current_exe().map_err(|e| io("InstanceDeck executable", e))?;
        let icon = ensure_icon(&self.state_dir, profile)?;
        let plan = IdentityPlan::new(
            profile,
            &launcher.to_string_lossy(),
            &icon.to_string_lossy(),
        )?;
        let mut paths = Vec::new();
        for location in locations {
            let directory = match location {
                ShortcutLocation::Desktop => known_folder(&FOLDERID_Desktop)?,
                ShortcutLocation::StartMenu => {
                    known_folder(&FOLDERID_Programs)?.join("InstanceDeck")
                }
            };
            let path = directory.join(plan.shortcut_filename());
            write_shortcut(&path, &plan)?;
            paths.push(path.to_string_lossy().into_owned());
        }
        Ok(paths)
    }
}
