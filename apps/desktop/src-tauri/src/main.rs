#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod bindings;
mod commands;
mod runtime;
mod service;
mod tray;
mod updates;
use ideck_core::shell::ShellOperations;
use std::{path::PathBuf, sync::Arc, time::Duration};
use tauri::{Emitter, Manager};

fn main() {
    if let Err(error) = run() {
        eprintln!("InstanceDeck could not start: {error}");
    }
}
fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::env::var_os("INSTANCEDECK_STATE_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("InstanceDeck")
        });
    if !root.is_absolute() {
        return Err("InstanceDeck state directory must be absolute".into());
    }
    let logs = root.join("logs");
    std::fs::create_dir_all(&logs)?;
    let (writer, _log_guard) =
        tracing_appender::non_blocking(tracing_appender::rolling::daily(&logs, "launches.jsonl"));
    tracing_subscriber::fmt()
        .json()
        .with_writer(writer)
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .map_err(|error| error.to_string())?;
    let state = Arc::new(service::DesktopState::new(root)?);
    let startup_args = std::env::args().collect::<Vec<_>>();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, args, _| {
            runtime::forwarded_args(app, &args);
        }))
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_log::Builder::new().skip_logger().build())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostart"])
                .build(),
        )
        .plugin(
            tauri_plugin_window_state::Builder::default()
                .with_filename(
                    state
                        .engine
                        .store
                        .root()
                        .join("window-state.json")
                        .to_string_lossy(),
                )
                .with_denylist(&["switcher"])
                .with_state_flags(
                    tauri_plugin_window_state::StateFlags::SIZE
                        | tauri_plugin_window_state::StateFlags::POSITION
                        | tauri_plugin_window_state::StateFlags::MAXIMIZED,
                )
                .build(),
        )
        .manage(Arc::clone(&state))
        .manage(updates::Updates::default())
        .invoke_handler(bindings::commands().invoke_handler())
        .setup(move |app| {
            if let Err(error) =
                state.install_runtime(Arc::new(runtime::NativeRuntime::new(app.handle().clone())))
            {
                tracing::warn!(%error,"shell runtime needs attention");
            }
            tray::install(app.handle(), Arc::clone(&state))?;
            if runtime::forwarded_args(app.handle(), &startup_args)
                && let Some(window) = app.get_webview_window("main")
            {
                let _ = window.hide();
            }
            let handle = app.handle().clone();
            std::thread::spawn(move || {
                let mut previous = String::new();
                loop {
                    match state.snapshot(false) {
                        Ok(snapshot) => match serde_json::to_string(&snapshot) {
                            Ok(current) if current != previous => {
                                previous = current;
                                let _ = handle.emit("desktop-status", &snapshot);
                                if let Err(error) = tray::refresh(&handle, &state) {
                                    tracing::warn!(%error, "tray refresh failed");
                                }
                            }
                            Ok(_) => {}
                            Err(error) => {
                                let _ = handle.emit("desktop-error", error.to_string());
                            }
                        },
                        Err(error) => {
                            let _ = handle.emit("desktop-error", error.to_string());
                        }
                    }
                    std::thread::sleep(Duration::from_secs(2));
                }
            });
            Ok(())
        })
        .on_window_event(|window, event| {
            if window.label() == "switcher" && matches!(event, tauri::WindowEvent::Focused(false)) {
                let _ = window.hide();
            }
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if window.label() == "switcher" {
                    api.prevent_close();
                    let _ = window.hide();
                    return;
                }
                let state = window.state::<Arc<service::DesktopState>>();
                if state
                    .settings
                    .read()
                    .map(|settings| settings.close_to_tray)
                    .unwrap_or(true)
                {
                    api.prevent_close();
                    let _ = window.hide();
                } else {
                    state.engine.platform.stop_window_identity();
                    window.app_handle().exit(0);
                }
            }
        })
        .build(tauri::generate_context!())?
        .run(|app, event| {
            if matches!(event, tauri::RunEvent::Exit)
                && let Some(state) = app.try_state::<Arc<service::DesktopState>>()
            {
                state.engine.platform.stop_window_identity();
            }
        });
    Ok(())
}
