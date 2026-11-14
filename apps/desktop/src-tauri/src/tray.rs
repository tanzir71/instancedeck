use crate::service::{DesktopState, ProfileAction};
use std::sync::{Arc, OnceLock};
use tauri::{
    AppHandle, Emitter, Manager,
    menu::{Menu, MenuBuilder, SubmenuBuilder},
    tray::{TrayIconBuilder, TrayIconEvent},
};

pub fn show(app: &AppHandle) {
    if let Some(window) = app.get_webview_window("main") {
        let _ = window.show();
        let _ = window.unminimize();
        let _ = window.set_focus();
    }
}
fn label(key: &str) -> &'static str {
    static COPY: OnceLock<serde_json::Value> = OnceLock::new();
    COPY.get_or_init(|| {
        serde_json::from_str(include_str!("../../../../packages/ui/src/copy/en.json"))
            .expect("Bundled UI copy is valid")
    })["desktop"][key]
        .as_str()
        .expect("Bundled tray label exists")
}
fn menu(app: &AppHandle, state: &DesktopState) -> tauri::Result<Menu<tauri::Wry>> {
    let profiles = state
        .engine
        .store
        .read()
        .map(|state| state.profiles)
        .unwrap_or_default();
    let mut builder = MenuBuilder::new(app);
    for recipe in state.engine.recipes() {
        let app_profiles = profiles
            .iter()
            .filter(|profile| profile.app_id == recipe.id)
            .collect::<Vec<_>>();
        if app_profiles.is_empty() {
            continue;
        }
        let mut group = SubmenuBuilder::new(app, &recipe.name);
        for profile in app_profiles {
            let running = state
                .engine
                .running(profile)
                .map(|processes| !processes.is_empty())
                .unwrap_or(false);
            group = group.text(
                format!("profile:{}", profile.id),
                format!(
                    "{} {}",
                    if running {
                        label("focus")
                    } else {
                        label("launch")
                    },
                    profile.name
                ),
            );
        }
        builder = builder.item(&group.build()?);
    }
    builder
        .separator()
        .text("open", label("openDeck"))
        .text("quit", label("quit"))
        .build()
}
pub fn install(app: &AppHandle, state: Arc<DesktopState>) -> tauri::Result<()> {
    let icon = app
        .default_window_icon()
        .cloned()
        .ok_or_else(|| tauri::Error::AssetNotFound("Application icon".into()))?;
    TrayIconBuilder::with_id("deck")
        .icon(icon)
        .tooltip("InstanceDeck")
        .menu(&menu(app, &state)?)
        .on_menu_event(|app, event| {
            let id = event.id().as_ref();
            if id == "quit" {
                app.exit(0);
            } else if id == "open" {
                show(app);
            } else if let Some(profile_id) = id.strip_prefix("profile:") {
                let profile_id = profile_id.to_owned();
                let app = app.clone();
                tauri::async_runtime::spawn_blocking(move || {
                    let state = app.state::<Arc<DesktopState>>();
                    if let Err(error) = state.action(&profile_id, ProfileAction::Launch, None) {
                        let _ = app.emit("desktop-error", error.to_string());
                        show(&app);
                    }
                });
            }
        })
        .on_tray_icon_event(|tray, event| {
            if matches!(event, TrayIconEvent::DoubleClick { .. }) {
                show(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
pub fn refresh(app: &AppHandle, state: &DesktopState) -> tauri::Result<()> {
    if let Some(tray) = app.tray_by_id("deck") {
        tray.set_menu(Some(menu(app, state)?))?;
    }
    Ok(())
}
