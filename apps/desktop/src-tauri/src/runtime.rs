//! Tauri owns manager hotkeys and its own autostart registration. It never hooks
//! another process; all profile actions go through the validated service.
use crate::{
    service::{DesktopState, ProfileAction},
    tray,
};
use ideck_core::{
    Error, Profile, Result, Settings,
    shell::{ShellRuntime, normalize_hotkey, validate_hotkeys},
};
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

#[derive(Clone, PartialEq, Eq)]
enum Action {
    Switcher,
    Profile(String),
}
pub struct NativeRuntime {
    app: AppHandle,
    registered: Mutex<BTreeMap<String, Action>>,
}
impl NativeRuntime {
    pub fn new(app: AppHandle) -> Self {
        Self {
            app,
            registered: Mutex::new(BTreeMap::new()),
        }
    }
    fn register(&self, key: &str, action: &Action) -> Result<()> {
        let action = action.clone();
        self.app
            .global_shortcut()
            .on_shortcut(key, move |app, _, event| {
                if event.state != ShortcutState::Pressed {
                    return;
                }
                match &action {
                    Action::Switcher => show_switcher(app),
                    Action::Profile(id) => launch(app, id.clone()),
                }
            })
            .map_err(|error| {
                Error::Platform(format!(
                    "Could not register {key}: {error}. Choose a different hotkey."
                ))
            })
    }
    fn restore(&self, changed: &[String], previous: &BTreeMap<String, Action>) {
        for key in changed {
            let _ = self.app.global_shortcut().unregister(key.as_str());
            if let Some(action) = previous.get(key)
                && let Err(error) = self.register(key, action)
            {
                tracing::warn!(%error,"hotkey rollback failed");
            }
        }
    }
}
impl ShellRuntime for NativeRuntime {
    fn apply(
        &self,
        settings: &Settings,
        profiles: &[Profile],
        change_autostart: bool,
    ) -> Result<()> {
        validate_hotkeys(settings, profiles)?;
        let mut desired = BTreeMap::from([(
            normalize_hotkey(&settings.switcher_hotkey)?,
            Action::Switcher,
        )]);
        for profile in profiles {
            if let Some(key) = &profile.hotkey {
                desired.insert(normalize_hotkey(key)?, Action::Profile(profile.id.clone()));
            }
        }
        let mut registered = self
            .registered
            .lock()
            .map_err(|_| Error::Platform("Restart InstanceDeck to reset hotkeys".into()))?;
        let mut changed: Vec<_> = registered
            .keys()
            .chain(desired.keys())
            .filter(|key| registered.get(*key) != desired.get(*key))
            .cloned()
            .collect();
        changed.sort();
        changed.dedup();
        for key in &changed {
            if registered.contains_key(key)
                && let Err(error) = self.app.global_shortcut().unregister(key.as_str())
            {
                self.restore(&changed, &registered);
                return Err(Error::Platform(error.to_string()));
            }
        }
        for key in &changed {
            if let Some(action) = desired.get(key)
                && let Err(error) = self.register(key, action)
            {
                self.restore(&changed, &registered);
                return Err(error);
            }
        }
        if change_autostart {
            let auto = self.app.autolaunch();
            let previous = match auto.is_enabled() {
                Ok(value) => value,
                Err(error) => {
                    self.restore(&changed, &registered);
                    return Err(Error::Platform(error.to_string()));
                }
            };
            let result = if settings.start_with_windows {
                auto.enable()
            } else {
                auto.disable()
            };
            if let Err(error) = result {
                self.restore(&changed, &registered);
                let rollback = if previous {
                    auto.enable()
                } else {
                    auto.disable()
                };
                if let Err(error) = rollback {
                    tracing::warn!(%error,"autostart rollback failed");
                }
                return Err(Error::Platform(format!(
                    "Could not change Start with Windows: {error}"
                )));
            }
        }
        *registered = desired;
        Ok(())
    }
}
fn launch(app: &AppHandle, id: String) {
    let app = app.clone();
    tauri::async_runtime::spawn_blocking(move || {
        let state = app.state::<Arc<DesktopState>>();
        if let Err(error) = state.action(&id, ProfileAction::Launch, None) {
            tracing::warn!(%error,profile_id = %id,"profile shortcut launch failed");
            let _ = app.emit("desktop-error", error.to_string());
            tray::show(&app);
        }
    });
}
pub fn show_switcher(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("switcher") {
        let _ = window.show();
        let _ = window.set_focus();
        let _ = app.emit_to("switcher", "switcher-open", ());
    }
}
/// Returns true when a cold manager should stay in the tray.
pub fn forwarded_args(app: &AppHandle, args: &[String]) -> bool {
    match args.get(1..).unwrap_or_default() {
        [command, id] if command == "launch" => {
            launch(app, id.clone());
            true
        }
        [flag] if flag == "--autostart" => true,
        [] => {
            tray::show(app);
            false
        }
        _ => {
            tray::show(app);
            let _ = app.emit(
                "desktop-error",
                "Expected InstanceDeck.exe launch <profileId>",
            );
            false
        }
    }
}
