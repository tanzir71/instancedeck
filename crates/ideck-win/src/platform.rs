use crate::command;
use ideck_core::{
    Directories, Discovery, Error, InstalledApp, LaunchLock, LaunchPlan, Platform, Recipe, Result,
    Strategy,
    error::io,
    paths,
    process::ProcessInfo,
    template::{self, Context},
};
use std::{
    collections::{BTreeSet, HashMap},
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{Mutex, OnceLock},
    time::Duration,
};
use windows::{
    Management::Deployment::PackageManager,
    Win32::{
        Foundation::{
            CloseHandle, FILETIME, HANDLE, HWND, LPARAM, WAIT_ABANDONED, WAIT_OBJECT_0, WPARAM,
        },
        System::{
            Threading::{
                CreateMutexW, GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
                PROCESS_TERMINATE, ReleaseMutex, TerminateProcess, WaitForSingleObject,
            },
            WinRT::{RO_INIT_MULTITHREADED, RoInitialize, RoUninitialize},
        },
        UI::WindowsAndMessaging::{
            EnumWindows, GetWindowThreadProcessId, IsWindowVisible, PostMessageW, SW_RESTORE,
            SetForegroundWindow, ShowWindow, WM_CLOSE,
        },
    },
    core::{BOOL, HSTRING, PCWSTR},
};
use winreg::{
    RegKey,
    enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ, KEY_WOW64_32KEY, KEY_WOW64_64KEY},
};

const CREATE_FLAGS: u32 = 0x00000008 | 0x00000200 | 0x00000400;
const CREATE_NO_WINDOW: u32 = 0x08000000;

pub struct WindowsPlatform {
    pub(crate) state_dir: PathBuf,
    cache: Mutex<HashMap<String, InstalledApp>>,
    spawned: Mutex<HashMap<u32, u64>>,
    pub(crate) sign_in_watch: Mutex<Option<crate::signin::MetadataWatch>>,
    pub(crate) identity_monitor: OnceLock<crate::window_identity::IdentityMonitor>,
}
impl WindowsPlatform {
    pub fn new(state_dir: impl Into<PathBuf>) -> Self {
        Self {
            state_dir: state_dir.into(),
            cache: Mutex::new(HashMap::new()),
            spawned: Mutex::new(HashMap::new()),
            sign_in_watch: Mutex::new(None),
            identity_monitor: OnceLock::new(),
        }
    }
    fn msix(
        &self,
        recipe: &Recipe,
        family: &str,
        relative_exe: &str,
        app_id: &str,
    ) -> Result<InstalledApp> {
        // RoInitialize may report changed apartment mode when the host already uses STA.
        let _apartment = RoScope(unsafe { RoInitialize(RO_INIT_MULTITHREADED).is_ok() });
        let manager = PackageManager::new().map_err(platform_error)?;
        let packages = manager
            .FindPackagesByUserSecurityIdPackageFamilyName(&HSTRING::new(), &HSTRING::from(family))
            .map_err(platform_error)?;
        let mut candidates = Vec::new();
        for package in packages {
            let identity = package.Id().map_err(platform_error)?;
            let version = identity.Version().map_err(platform_error)?;
            let directory = package
                .InstalledLocation()
                .map_err(platform_error)?
                .Path()
                .map_err(platform_error)?
                .to_string();
            let executable = Path::new(&directory).join(relative_exe);
            if executable.try_exists().map_err(|e| io(&executable, e))? {
                candidates.push((
                    (
                        version.Major,
                        version.Minor,
                        version.Build,
                        version.Revision,
                    ),
                    InstalledApp {
                        app_id: recipe.id.clone(),
                        name: recipe.name.clone(),
                        version: format!(
                            "{}.{}.{}.{}",
                            version.Major, version.Minor, version.Build, version.Revision
                        ),
                        executable: executable.to_string_lossy().into_owned(),
                        install_dir: directory,
                        install_type: "msix".into(),
                        package_family: Some(family.into()),
                        package_app_id: Some(app_id.into()),
                    },
                ));
            }
        }
        candidates.sort_by_key(|(version, _)| *version);
        candidates
            .pop()
            .map(|(_, app)| app)
            .ok_or_else(|| Error::NotFound(format!("{} for this Windows user", recipe.name)))
    }
    fn registry(
        &self,
        recipe: &Recipe,
        display_name: &str,
        relative_exe: &str,
    ) -> Result<InstalledApp> {
        for hive in [HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE] {
            for view in [KEY_WOW64_64KEY, KEY_WOW64_32KEY] {
                let root = RegKey::predef(hive);
                let Ok(uninstall) = root.open_subkey_with_flags(
                    "Software\\Microsoft\\Windows\\CurrentVersion\\Uninstall",
                    KEY_READ | view,
                ) else {
                    continue;
                };
                for entry in uninstall.enum_keys().flatten() {
                    let Ok(key) = uninstall.open_subkey_with_flags(entry, KEY_READ) else {
                        continue;
                    };
                    let Ok(name) = key.get_value::<String, _>("DisplayName") else {
                        continue;
                    };
                    if !name.eq_ignore_ascii_case(display_name)
                        && !name
                            .to_lowercase()
                            .starts_with(&format!("{} ", display_name.to_lowercase()))
                    {
                        continue;
                    }
                    let Ok(location) = key.get_value::<String, _>("InstallLocation") else {
                        continue;
                    };
                    if location.trim().is_empty() {
                        continue;
                    }
                    let path = Path::new(&location).join(relative_exe);
                    if path.try_exists().map_err(|e| io(&path, e))? {
                        return Ok(InstalledApp {
                            app_id: recipe.id.clone(),
                            name: recipe.name.clone(),
                            version: key
                                .get_value("DisplayVersion")
                                .unwrap_or_else(|_| "unknown".into()),
                            executable: path.to_string_lossy().into_owned(),
                            install_dir: location,
                            install_type: "registry".into(),
                            package_family: None,
                            package_app_id: None,
                        });
                    }
                }
            }
        }
        Err(Error::NotFound(format!("{} registry install", recipe.name)))
    }
    fn known_path(
        &self,
        recipe: &Recipe,
        path: &str,
        version_prefix: Option<&str>,
    ) -> Result<InstalledApp> {
        let dirs = self.directories()?;
        let context = Context {
            profile: String::new(),
            install_dir: String::new(),
            exe_dir: String::new(),
            user_profile: dirs.user_profile,
            local_app_data: dirs.local_app_data,
        };
        let expanded = template::expand(path, &context)?;
        paths::validate_absolute(&expanded)?;
        let mut executable = PathBuf::from(&expanded);
        let mut version = "unknown".to_owned();
        if let Some(prefix) = version_prefix
            && let Some((found, folder_version)) =
                crate::discovery::versioned_executable(&executable, prefix)?
        {
            executable = found;
            version = folder_version;
        }
        if !executable.try_exists().map_err(|e| io(&executable, e))? {
            return Err(Error::NotFound(format!("{} executable", recipe.name)));
        }
        Ok(InstalledApp {
            app_id: recipe.id.clone(),
            name: recipe.name.clone(),
            version,
            install_dir: executable
                .parent()
                .ok_or_else(|| Error::Invalid("Executable has no parent".into()))?
                .to_string_lossy()
                .into_owned(),
            executable: executable.to_string_lossy().into_owned(),
            install_type: "known-path".into(),
            package_family: None,
            package_app_id: None,
        })
    }
    fn powershell(&self) -> Result<PathBuf> {
        Ok(PathBuf::from(
            std::env::var_os("SystemRoot")
                .ok_or_else(|| Error::Platform("SystemRoot is not set".into()))?,
        )
        .join("System32")
        .join("WindowsPowerShell")
        .join("v1.0")
        .join("powershell.exe"))
    }
}

fn platform_error(error: windows::core::Error) -> Error {
    Error::Platform(format!("{} ({:#010x})", error.message(), error.code().0))
}
fn env_path(name: &str) -> Result<String> {
    std::env::var(name)
        .map_err(|_| Error::Platform(format!("{name} is unavailable for this Windows user")))
}

struct MutexGuard(HANDLE);
impl LaunchLock for MutexGuard {}
impl Drop for MutexGuard {
    fn drop(&mut self) {
        unsafe {
            let _ = ReleaseMutex(self.0);
            let _ = CloseHandle(self.0);
        }
    }
}

impl Platform for WindowsPlatform {
    fn check_history(&self, p: &ideck_core::Profile, root: &Path) -> Result<()> {
        self.history_check(p, root)
    }
    fn history_kind(&self, path: &Path) -> Result<ideck_core::history::Kind> {
        crate::history::kind(path)
    }
    fn move_history(&self, from: &Path, to: &Path) -> Result<()> {
        crate::history::move_path(from, to)
    }
    fn create_history_junction(&self, target: &Path, link: &Path) -> Result<()> {
        crate::history::create(target, link)
    }
    fn track_profile_windows(
        &self,
        profile: &ideck_core::Profile,
        processes: &[ProcessInfo],
    ) -> Result<()> {
        self.identity_monitor()
            .upsert(ideck_core::shell::IdentityBinding {
                profile: profile.clone(),
                processes: processes.to_vec(),
            });
        Ok(())
    }
    fn directories(&self) -> Result<Directories> {
        Ok(Directories {
            local_app_data: env_path("LOCALAPPDATA")?,
            app_data: env_path("APPDATA")?,
            user_profile: env_path("USERPROFILE")?,
        })
    }
    fn discover(&self, recipe: &Recipe, refresh: bool) -> Result<InstalledApp> {
        if !refresh
            && let Some(app) = self
                .cache
                .lock()
                .map_err(|_| Error::Platform("Discovery cache lock failed".into()))?
                .get(&recipe.id)
                .cloned()
            && self.exists(&app.executable)?
        {
            return Ok(app);
        }
        let mut last_error = Error::NotFound(recipe.name.clone());
        for method in &recipe.discovery {
            let found = match method {
                Discovery::Msix {
                    package_family,
                    exe,
                    app_id,
                } => self.msix(recipe, package_family, exe, app_id),
                Discovery::Registry { display_name, exe } => {
                    self.registry(recipe, display_name, exe)
                }
                Discovery::KnownPath {
                    path,
                    version_prefix,
                } => self.known_path(recipe, path, version_prefix.as_deref()),
            };
            match found {
                Ok(app) => {
                    self.cache
                        .lock()
                        .map_err(|_| Error::Platform("Discovery cache lock failed".into()))?
                        .insert(recipe.id.clone(), app.clone());
                    return Ok(app);
                }
                Err(error) => last_error = error,
            }
        }
        Err(last_error)
    }
    fn exists(&self, path: &str) -> Result<bool> {
        Path::new(path).try_exists().map_err(|e| io(path, e))
    }
    fn ensure_directory(&self, path: &str) -> Result<()> {
        fs::create_dir_all(path).map_err(|e| io(path, e))
    }
    fn launch_lock(&self) -> Result<Box<dyn LaunchLock + '_>> {
        let name: Vec<u16> = "Local\\InstanceDeck.Launch\0".encode_utf16().collect();
        let handle =
            unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())).map_err(platform_error)? };
        let result = unsafe { WaitForSingleObject(handle, 20_000) };
        if result == WAIT_OBJECT_0 || result == WAIT_ABANDONED {
            Ok(Box::new(MutexGuard(handle)))
        } else {
            unsafe {
                let _ = CloseHandle(handle);
            }
            Err(Error::Platform(
                "Another launch did not finish within 20 seconds".into(),
            ))
        }
    }
    fn launch(&self, plan: &LaunchPlan, strategy: Strategy) -> Result<u32> {
        match strategy {
            Strategy::Direct => Command::new(&plan.app.executable)
                .args(&plan.args)
                .envs(&plan.env)
                .current_dir(&plan.working_dir)
                .creation_flags(CREATE_FLAGS)
                .stdin(Stdio::null())
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .and_then(|child| {
                    if plan.match_argument.is_empty() && plan.profile_id != "main" {
                        use std::os::windows::io::AsRawHandle;
                        let mut created = FILETIME::default();
                        let mut exit = FILETIME::default();
                        let mut kernel = FILETIME::default();
                        let mut user = FILETIME::default();
                        unsafe {
                            GetProcessTimes(
                                HANDLE(child.as_raw_handle()),
                                &mut created,
                                &mut exit,
                                &mut kernel,
                                &mut user,
                            )
                        }
                        .map_err(std::io::Error::other)?;
                        let started = (u64::from(created.dwHighDateTime) << 32)
                            | u64::from(created.dwLowDateTime);
                        self.spawned
                            .lock()
                            .map_err(|_| std::io::Error::other("Launch identity lock failed"))?
                            .insert(child.id(), started);
                    }
                    Ok(child.id())
                })
                .map_err(|error| Error::Platform(format!("CreateProcessW: {error}"))),
            Strategy::PackageContext => {
                let family = plan.app.package_family.as_deref().ok_or_else(|| {
                    Error::Invalid("Package context is unavailable for this install".into())
                })?;
                let powershell = self.powershell()?;
                let powershell = powershell.to_string_lossy();
                let run = self.state_dir.join("run");
                fs::create_dir_all(&run).map_err(|e| io(&run, e))?;
                let wrapper = run.join(format!(
                    "launch-{}-{}.cmd",
                    plan.profile_id, plan.recipe_version
                ));
                let line = command::wrapper_line(&powershell, plan)?;
                fs::write(&wrapper, command::utf16_le(&line, true)).map_err(|e| io(&wrapper, e))?;
                let script = command::package_script(
                    family,
                    plan.app.package_app_id.as_deref().unwrap_or("App"),
                    &powershell,
                    &wrapper.to_string_lossy(),
                );
                Command::new(powershell.as_ref())
                    .args([
                        "-NoProfile",
                        "-NonInteractive",
                        "-EncodedCommand",
                        &command::encoded_powershell(&script),
                    ])
                    .creation_flags(CREATE_NO_WINDOW | 0x00000400)
                    .stdin(Stdio::null())
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .spawn()
                    .map(|child| child.id())
                    .map_err(|error| Error::Platform(format!("Package-context helper: {error}")))
            }
        }
    }
    fn created_process_time(&self, pid: u32) -> Result<Option<u64>> {
        Ok(self
            .spawned
            .lock()
            .map_err(|_| Error::Platform("Launch identity lock failed".into()))?
            .remove(&pid))
    }
    fn processes(&self) -> Result<Vec<ProcessInfo>> {
        use sysinfo::{ProcessRefreshKind, ProcessesToUpdate, System, UpdateKind};
        let mut system =
            System::new().map_err(|error| Error::Platform(format!("Process snapshot: {error}")))?;
        system.refresh_processes_specifics(
            ProcessesToUpdate::All,
            true,
            ProcessRefreshKind::nothing().with_exe(UpdateKind::Always),
        );
        let installs: Vec<String> = self
            .cache
            .lock()
            .map_err(|_| Error::Platform("Discovery cache lock failed".into()))?
            .values()
            .map(|app| app.install_dir.clone())
            .collect();
        let selected: Vec<_> = system
            .processes()
            .iter()
            .filter(|(_, process)| {
                process.exe().is_some_and(|exe| {
                    installs
                        .iter()
                        .any(|install| paths::is_within_windows(&exe.to_string_lossy(), install))
                })
            })
            .map(|(pid, _)| *pid)
            .collect();
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&selected),
            true,
            ProcessRefreshKind::nothing().with_cmd(UpdateKind::Always),
        );
        let manager_exe = std::env::current_exe().ok();
        Ok(system
            .processes()
            .iter()
            .filter_map(|(pid, process)| {
                let executable = process.exe()?.to_string_lossy().into_owned();
                if manager_exe.as_ref().is_some_and(|manager| {
                    paths::normalize_windows(&manager.to_string_lossy())
                        == paths::normalize_windows(&executable)
                }) {
                    return None;
                }
                Some(ProcessInfo {
                    pid: pid.as_u32(),
                    parent_pid: process.parent().map(|pid| pid.as_u32()),
                    executable,
                    args: process
                        .cmd()
                        .iter()
                        .map(|arg| arg.to_string_lossy().into_owned())
                        .collect(),
                    started_at: process_started_at(pid.as_u32()).unwrap_or(0),
                })
            })
            .collect())
    }
    fn request_close(&self, processes: &[ProcessInfo]) -> Result<()> {
        let current = self.processes()?;
        let pids = current
            .iter()
            .filter(|p| processes.iter().any(|old| same_process(old, p)))
            .map(|p| p.pid)
            .collect();
        for window in windows_for(&pids)? {
            unsafe {
                let _ = PostMessageW(Some(window), WM_CLOSE, WPARAM(0), LPARAM(0));
            }
        }
        Ok(())
    }
    fn force_close(&self, processes: &[ProcessInfo]) -> Result<()> {
        // Reopen and revalidate each process immediately before TerminateProcess.
        for process in processes.iter().rev() {
            if process.started_at == 0 {
                return Err(Error::Platform(
                    "Cannot prove process identity; force close refused".into(),
                ));
            }
            let handle = match unsafe {
                OpenProcess(
                    PROCESS_TERMINATE | PROCESS_QUERY_LIMITED_INFORMATION,
                    false,
                    process.pid,
                )
            } {
                Ok(handle) => handle,
                Err(error) if error.code() == windows::core::HRESULT::from_win32(87) => continue,
                Err(error) => return Err(platform_error(error)),
            };
            let guard = ProcessHandle(handle);
            if handle_started_at(guard.0) == Some(process.started_at) {
                unsafe {
                    TerminateProcess(guard.0, 1).map_err(platform_error)?;
                }
            }
        }
        Ok(())
    }
    fn focus(&self, processes: &[ProcessInfo]) -> Result<()> {
        let pids = self
            .processes()?
            .iter()
            .filter(|p| processes.iter().any(|old| same_process(old, p)))
            .map(|p| p.pid)
            .collect();
        if let Some(window) = windows_for(&pids)?.first() {
            unsafe {
                let _ = ShowWindow(*window, SW_RESTORE);
                let _ = SetForegroundWindow(*window);
            }
        }
        Ok(())
    }
    fn wait(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

fn same_process(a: &ProcessInfo, b: &ProcessInfo) -> bool {
    a.pid == b.pid
        && a.started_at != 0
        && a.started_at == b.started_at
        && paths::normalize_windows(&a.executable) == paths::normalize_windows(&b.executable)
}
struct ProcessHandle(HANDLE);
struct RoScope(bool);
impl Drop for RoScope {
    fn drop(&mut self) {
        if self.0 {
            unsafe {
                RoUninitialize();
            }
        }
    }
}
impl Drop for ProcessHandle {
    fn drop(&mut self) {
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}
fn process_started_at(pid: u32) -> Option<u64> {
    let handle = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()? };
    handle_started_at(ProcessHandle(handle).0)
}
fn handle_started_at(handle: HANDLE) -> Option<u64> {
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(handle, &mut created, &mut exited, &mut kernel, &mut user).ok()?;
    }
    Some(((created.dwHighDateTime as u64) << 32) | created.dwLowDateTime as u64)
}

fn windows_for(pids: &BTreeSet<u32>) -> Result<Vec<HWND>> {
    struct Enumeration<'a> {
        pids: &'a BTreeSet<u32>,
        windows: Vec<HWND>,
    }
    unsafe extern "system" fn callback(window: HWND, parameter: LPARAM) -> BOOL {
        let context = unsafe { &mut *(parameter.0 as *mut Enumeration<'_>) };
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(window, Some(&mut pid));
        }
        if context.pids.contains(&pid) && unsafe { IsWindowVisible(window).as_bool() } {
            context.windows.push(window);
        }
        BOOL(1)
    }
    let mut enumeration = Enumeration {
        pids,
        windows: Vec::new(),
    };
    unsafe {
        EnumWindows(
            Some(callback),
            LPARAM((&mut enumeration as *mut Enumeration<'_>) as isize),
        )
        .map_err(platform_error)?;
    }
    Ok(enumeration.windows)
}
