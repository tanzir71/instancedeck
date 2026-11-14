use crate::WindowsPlatform;
use ideck_core::{DesktopOperations, Error, Profile, Result, error::io};
use std::{
    fs,
    os::windows::{ffi::OsStrExt, fs::MetadataExt, process::CommandExt},
    path::Path,
    process::Command,
};

impl DesktopOperations for WindowsPlatform {
    fn open_folder(&self, path: &str) -> Result<()> {
        ideck_core::paths::validate_absolute(path)?;
        if !Path::new(path).is_dir() {
            return Err(Error::NotFound(
                "Profile data folder; launch the profile once to create it".into(),
            ));
        }
        Command::new("explorer.exe")
            .arg(path)
            .creation_flags(0x08000000)
            .spawn()
            .map_err(|error| io(path, error))?;
        Ok(())
    }
    fn recycle_profile(&self, profile: &Profile) -> Result<()> {
        profile.validate()?;
        let path = Path::new(&profile.data_dir);
        // Only a manager-shaped ULID folder is eligible; no arbitrary filesystem target.
        if path.file_name().and_then(|name| name.to_str()) != Some(&profile.id)
            || path
                .parent()
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                != Some(&profile.app_id)
        {
            return Err(Error::Invalid(
                "Only the profile's own app/ULID data directory can be recycled".into(),
            ));
        }
        let metadata = fs::symlink_metadata(path).map_err(|error| io(path, error))?;
        if !metadata.is_dir() || metadata.file_attributes() & 0x400 != 0 {
            return Err(Error::Invalid(
                "Reparse points and non-directory profile targets cannot be recycled".into(),
            ));
        }
        recycle_only(path)
    }
}

/// Explicitly require recycling and refuse the shell's permanent-delete fallback.
/// No confirmation is answered automatically. If the bin is disabled, unavailable
/// or too small, WANTNUKEWARNING + NOERRORUI aborts instead of accepting destruction.
/// https://learn.microsoft.com/windows/win32/api/shobjidl_core/nf-shobjidl_core-ifileoperation-setoperationflags
fn recycle_only(path: &Path) -> Result<()> {
    use windows::{
        Win32::{
            System::Com::{
                CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize,
            },
            UI::Shell::{
                FOF_NOERRORUI, FOF_SILENT, FOF_WANTNUKEWARNING, FOFX_ADDUNDORECORD,
                FOFX_EARLYFAILURE, FOFX_RECYCLEONDELETE, FileOperation, IFileOperation, IShellItem,
                SHCreateItemFromParsingName,
            },
        },
        core::PCWSTR,
    };
    struct ComGuard;
    impl Drop for ComGuard {
        fn drop(&mut self) {
            // Balanced only after a successful initialization on this thread.
            unsafe { CoUninitialize() };
        }
    }
    let operation = || -> windows::core::Result<bool> {
        // The caller runs on a blocking worker. A conflicting COM apartment is
        // an error, never a reason to take a different deletion path.
        unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()? };
        let _com = ComGuard;
        let wide: Vec<u16> = path.as_os_str().encode_wide().chain(Some(0)).collect();
        unsafe {
            let operation: IFileOperation =
                CoCreateInstance(&FileOperation, None, CLSCTX_INPROC_SERVER)?;
            operation.SetOperationFlags(
                FOFX_RECYCLEONDELETE
                    | FOFX_ADDUNDORECORD
                    | FOFX_EARLYFAILURE
                    | FOF_WANTNUKEWARNING
                    | FOF_NOERRORUI
                    | FOF_SILENT,
            )?;
            let item: IShellItem = SHCreateItemFromParsingName(PCWSTR(wide.as_ptr()), None)?;
            operation.DeleteItem(&item, None)?;
            operation.PerformOperations()?;
            Ok(operation.GetAnyOperationsAborted()?.as_bool())
        }
    };
    if operation().map_err(|error| Error::Platform(format!("Recycle Bin: {error}")))? {
        return Err(Error::Platform(
            "Windows could not recycle this profile; its deck entry has been retained".into(),
        ));
    }
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(io(path, error)),
        Ok(_) => Err(Error::Platform(
            "Windows did not recycle the profile folder; its deck entry has been retained".into(),
        )),
    }
}
