//! Out-of-context window events and shell properties only. No target-process hook,
//! injection, title reading or target binary/resource access.
use crate::{
    WindowsPlatform,
    shell::{ComScope, ensure_icon, set_properties, win_error},
};
use ideck_core::{
    Error, Result,
    shell::{IdentityBinding, IdentityPlan, StableWindows, WindowCandidate},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, VecDeque},
    path::PathBuf,
    sync::{Arc, Mutex, mpsc},
    time::{Duration, Instant},
};
use windows::{
    Win32::{
        Foundation::{CloseHandle, FILETIME, HWND, LPARAM, WPARAM},
        Storage::EnhancedStorage::{
            PKEY_AppUserModel_ID, PKEY_AppUserModel_RelaunchCommand,
            PKEY_AppUserModel_RelaunchDisplayNameResource, PKEY_AppUserModel_RelaunchIconResource,
        },
        System::{
            Com::StructuredStorage::PROPVARIANT,
            Threading::{GetProcessTimes, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION},
        },
        UI::{
            Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
            Shell::PropertiesSystem::{IPropertyStore, SHGetPropertyStoreForWindow},
            WindowsAndMessaging::*,
        },
    },
    core::HSTRING,
};

enum Command {
    Replace(Vec<IdentityBinding>),
    Upsert(IdentityBinding),
    Stop(mpsc::Sender<()>),
}
pub(crate) struct IdentityMonitor {
    sender: mpsc::SyncSender<Command>,
    warnings: Arc<Mutex<BTreeMap<String, String>>>,
}
impl WindowsPlatform {
    pub(crate) fn identity_monitor(&self) -> &IdentityMonitor {
        self.identity_monitor
            .get_or_init(|| IdentityMonitor::start(self.state_dir.clone()))
    }
}
impl IdentityMonitor {
    fn start(root: PathBuf) -> Self {
        let (sender, receiver) = mpsc::sync_channel(32);
        let warnings = Arc::new(Mutex::new(BTreeMap::new()));
        let reports = Arc::clone(&warnings);
        let worker = std::thread::Builder::new()
            .name("InstanceDeck window identity".into())
            .spawn(move || {
                if let Err(error) = monitor(root, receiver, Arc::clone(&reports)) {
                    if let Ok(mut reports) = reports.lock() {
                        reports.insert("window-identity".into(), error.to_string());
                    }
                    tracing::warn!(%error,"window identity monitor stopped");
                }
            });
        if let Err(error) = worker
            && let Ok(mut reports) = warnings.lock()
        {
            reports.insert(
                "window-identity".into(),
                format!("Could not start window identity monitor: {error}"),
            );
        }
        Self { sender, warnings }
    }
    pub(crate) fn upsert(&self, binding: IdentityBinding) {
        let _ = self.sender.try_send(Command::Upsert(binding));
    }
    pub(crate) fn replace(&self, bindings: Vec<IdentityBinding>) -> BTreeMap<String, String> {
        let _ = self.sender.try_send(Command::Replace(bindings));
        self.warnings.lock().map(|v| v.clone()).unwrap_or_else(|_| {
            BTreeMap::from([(
                "window-identity".into(),
                "Window identity monitor needs a restart".into(),
            )])
        })
    }
    pub(crate) fn stop(&self) {
        let (ack, done) = mpsc::channel();
        if self.sender.try_send(Command::Stop(ack)).is_ok() {
            let _ = done.recv_timeout(Duration::from_millis(750));
        }
    }
}
thread_local! { static EVENTS: RefCell<VecDeque<(usize,u32)>> = const { RefCell::new(VecDeque::new()) }; }
unsafe extern "system" fn event_callback(
    _: HWINEVENTHOOK,
    event: u32,
    hwnd: HWND,
    object: i32,
    child: i32,
    _: u32,
    _: u32,
) {
    if object == 0 && child == 0 && !hwnd.0.is_null() {
        EVENTS.with(|events| {
            if let Ok(mut events) = events.try_borrow_mut()
                && events.len() < 512
            {
                events.push_back((hwnd.0 as usize, event));
            }
        });
    }
}
struct Hook(HWINEVENTHOOK);
impl Drop for Hook {
    fn drop(&mut self) {
        unsafe {
            let _ = UnhookWinEvent(self.0);
        }
    }
}
struct Watch {
    binding: IdentityBinding,
    stable: StableWindows,
    started: Instant,
    applied: bool,
}
struct Applied {
    candidate: WindowCandidate,
    profile_id: String,
    signature: String,
    last_verified: Instant,
}
struct WindowResources {
    applied: BTreeMap<usize, Applied>,
    icons: BTreeMap<PathBuf, (HICON, HICON)>,
}
impl Drop for WindowResources {
    fn drop(&mut self) {
        for value in self.applied.values() {
            clear(value);
        }
        for (big, small) in self.icons.values() {
            unsafe {
                let _ = DestroyIcon(*big);
                let _ = DestroyIcon(*small);
            }
        }
    }
}
fn same_process(candidate: WindowCandidate) -> bool {
    let mut pid = 0;
    unsafe {
        GetWindowThreadProcessId(HWND(candidate.handle as *mut _), Some(&mut pid));
    }
    if pid != candidate.pid {
        return false;
    }
    let Ok(handle) = (unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) }) else {
        return false;
    };
    let mut created = FILETIME::default();
    let mut exit = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    let ok =
        unsafe { GetProcessTimes(handle, &mut created, &mut exit, &mut kernel, &mut user).is_ok() };
    unsafe {
        let _ = CloseHandle(handle);
    }
    ok && ((created.dwHighDateTime as u64) << 32 | created.dwLowDateTime as u64)
        == candidate.process_started_at
}
fn clear(applied: &Applied) {
    if !same_process(applied.candidate) {
        return;
    }
    if let Ok(store) = unsafe {
        SHGetPropertyStoreForWindow::<IPropertyStore>(HWND(applied.candidate.handle as *mut _))
    } {
        for key in [
            PKEY_AppUserModel_ID,
            PKEY_AppUserModel_RelaunchCommand,
            PKEY_AppUserModel_RelaunchDisplayNameResource,
            PKEY_AppUserModel_RelaunchIconResource,
        ] {
            unsafe {
                let _ = store.SetValue(&key, &PROPVARIANT::default());
            }
        }
    }
}
fn candidates(binding: &IdentityBinding) -> Vec<WindowCandidate> {
    struct Context<'a> {
        binding: &'a IdentityBinding,
        output: Vec<WindowCandidate>,
    }
    unsafe extern "system" fn enumerate(hwnd: HWND, parameter: LPARAM) -> windows::core::BOOL {
        let context = unsafe { &mut *(parameter.0 as *mut Context<'_>) };
        if !unsafe { IsWindowVisible(hwnd) }.as_bool()
            || unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) } as u32 & WS_EX_TOOLWINDOW.0 != 0
        {
            return true.into();
        }
        if unsafe { GetWindow(hwnd, GW_OWNER) }.is_ok_and(|owner| !owner.0.is_null()) {
            return true.into();
        }
        let mut pid = 0;
        unsafe {
            GetWindowThreadProcessId(hwnd, Some(&mut pid));
        }
        if let Some(process) = context
            .binding
            .processes
            .iter()
            .find(|p| p.pid == pid && p.started_at != 0)
        {
            let value = WindowCandidate {
                handle: hwnd.0 as usize,
                pid,
                process_started_at: process.started_at,
            };
            if same_process(value) {
                context.output.push(value);
            }
        }
        true.into()
    }
    let mut context = Context {
        binding,
        output: Vec::new(),
    };
    unsafe {
        let _ = EnumWindows(
            Some(enumerate),
            LPARAM((&mut context as *mut Context<'_>) as isize),
        );
    }
    context.output
}
fn apply(
    candidate: WindowCandidate,
    plan: &IdentityPlan,
    resources: &mut WindowResources,
) -> Result<()> {
    if !same_process(candidate) {
        return Err(Error::Platform(
            "The window's process identity changed".into(),
        ));
    }
    let hwnd = HWND(candidate.handle as *mut _);
    let store: IPropertyStore = unsafe { SHGetPropertyStoreForWindow(hwnd) }.map_err(win_error)?;
    set_properties(&store, plan)?;
    let icon_path = PathBuf::from(&plan.icon);
    if !resources.icons.contains_key(&icon_path) {
        let raw = unsafe {
            LoadImageW(
                None,
                &HSTRING::from(&plan.icon),
                IMAGE_ICON,
                256,
                256,
                LR_LOADFROMFILE,
            )
        }
        .map_err(win_error)?;
        let small = unsafe {
            LoadImageW(
                None,
                &HSTRING::from(&plan.icon),
                IMAGE_ICON,
                32,
                32,
                LR_LOADFROMFILE,
            )
        }
        .map_err(|error| {
            unsafe {
                let _ = DestroyIcon(HICON(raw.0));
            }
            win_error(error)
        })?;
        resources
            .icons
            .insert(icon_path.clone(), (HICON(raw.0), HICON(small.0)));
    }
    let (big, small) = resources.icons[&icon_path];
    // A hung target can delay at most 100ms per icon; no memory is written in it.
    for (size, icon) in [(ICON_BIG, big), (ICON_SMALL, small)] {
        unsafe {
            SendMessageTimeoutW(
                hwnd,
                WM_SETICON,
                WPARAM(size as usize),
                LPARAM(icon.0 as isize),
                SMTO_ABORTIFHUNG,
                100,
                None,
            );
        }
    }
    Ok(())
}
fn monitor(
    root: PathBuf,
    receiver: mpsc::Receiver<Command>,
    warnings: Arc<Mutex<BTreeMap<String, String>>>,
) -> Result<()> {
    let _com = ComScope::new()?;
    let launcher =
        std::env::current_exe().map_err(|e| ideck_core::error::io("InstanceDeck executable", e))?;
    let launcher = if launcher
        .file_stem()
        .is_some_and(|v| v.eq_ignore_ascii_case("instancedeck"))
    {
        launcher
    } else {
        let sibling = launcher.with_file_name("InstanceDeck.exe");
        if !sibling.is_file() {
            return Err(Error::NotFound(
                "InstanceDeck.exe for profile relaunch".into(),
            ));
        }
        sibling
    };
    let hook = unsafe {
        SetWinEventHook(
            EVENT_OBJECT_CREATE,
            EVENT_OBJECT_HIDE,
            None,
            Some(event_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };
    if hook.0.is_null() {
        return Err(Error::Platform(
            "Windows refused the out-of-context window event monitor".into(),
        ));
    }
    let _hook = Hook(hook);
    let mut watches: BTreeMap<String, Watch> = BTreeMap::new();
    let mut resources = WindowResources {
        applied: BTreeMap::new(),
        icons: BTreeMap::new(),
    };
    loop {
        loop {
            let command = match receiver.try_recv() {
                Ok(command) => command,
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => return Ok(()),
            };
            let bindings = match command {
                Command::Stop(ack) => {
                    drop(resources);
                    let _ = ack.send(());
                    return Ok(());
                }
                Command::Replace(bindings) => {
                    watches.retain(|id, _| bindings.iter().any(|b| b.profile.id == *id));
                    bindings
                }
                Command::Upsert(binding) => vec![binding],
            };
            for binding in bindings {
                watches
                    .entry(binding.profile.id.clone())
                    .and_modify(|watch| {
                        let overlap = watch.binding.processes.iter().any(|old| {
                            binding
                                .processes
                                .iter()
                                .any(|new| old.pid == new.pid && old.started_at == new.started_at)
                        });
                        if !overlap {
                            watch.started = Instant::now();
                            watch.stable = StableWindows::default();
                            watch.applied = false;
                        }
                        watch.binding = binding.clone();
                    })
                    .or_insert(Watch {
                        binding,
                        stable: StableWindows::default(),
                        started: Instant::now(),
                        applied: false,
                    });
            }
        }
        let mut message = MSG::default();
        while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
            unsafe {
                let _ = TranslateMessage(&message);
                DispatchMessageW(&message);
            }
        }
        let events = EVENTS.with(|events| events.borrow_mut().drain(..).collect::<Vec<_>>());
        for (handle, event) in events {
            if matches!(event, EVENT_OBJECT_HIDE | EVENT_OBJECT_DESTROY)
                && let Some(applied) = resources.applied.remove(&handle)
            {
                clear(&applied);
            }
        }
        let mut reports = BTreeMap::new();
        for (id, watch) in &mut watches {
            if watch.binding.processes.is_empty() {
                continue;
            }
            let available = candidates(&watch.binding);
            let now = watch.started.elapsed().as_millis() as u64;
            for candidate in watch.stable.observe(now, &available) {
                let result = (|| -> Result<()> {
                    let icon = ensure_icon(&root, &watch.binding.profile)?;
                    let plan = IdentityPlan::new(
                        &watch.binding.profile,
                        &launcher.to_string_lossy(),
                        &icon.to_string_lossy(),
                    )?;
                    let signature = serde_json::to_string(&plan)?;
                    if let Some(applied) = resources.applied.get_mut(&candidate.handle)
                        && applied.candidate == candidate
                        && applied.signature == signature
                    {
                        if applied.last_verified.elapsed() < Duration::from_secs(2) {
                            return Ok(());
                        }
                        // Only the app-model identifier is queried, never titles or app content.
                        let store: IPropertyStore = unsafe {
                            SHGetPropertyStoreForWindow(HWND(candidate.handle as *mut _))
                        }
                        .map_err(win_error)?;
                        let identity = unsafe { store.GetValue(&PKEY_AppUserModel_ID) }
                            .map_err(win_error)?
                            .to_string();
                        if identity == plan.app_user_model_id {
                            applied.last_verified = Instant::now();
                            return Ok(());
                        }
                    }
                    apply(candidate, &plan, &mut resources)?;
                    resources.applied.insert(
                        candidate.handle,
                        Applied {
                            candidate,
                            profile_id: id.clone(),
                            signature,
                            last_verified: Instant::now(),
                        },
                    );
                    Ok(())
                })();
                match result {
                    Ok(()) => watch.applied = true,
                    Err(error) => {
                        reports.insert(id.clone(), error.to_string());
                    }
                }
            }
            watch.applied = available.iter().any(|candidate| {
                resources
                    .applied
                    .get(&candidate.handle)
                    .is_some_and(|applied| {
                        applied.candidate == *candidate && applied.profile_id == *id
                    })
            });
            if !watch.applied && now >= 20_000 {
                reports.entry(id.clone()).or_insert_with(|| "No stable eligible window received profile identity within 20 seconds. Launch succeeded; verify taskbar identity manually.".into());
            }
        }
        resources.applied.retain(|_, applied| {
            if !watches.contains_key(&applied.profile_id) {
                clear(applied);
                false
            } else {
                same_process(applied.candidate)
            }
        });
        if let Ok(mut current) = warnings.lock() {
            *current = reports;
        }
        std::thread::sleep(Duration::from_millis(250));
    }
}
