#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod service;
mod tray;
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
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            tray::show(app)
        }))
        .plugin(tauri_plugin_dialog::init())
        .manage(Arc::clone(&state))
        .invoke_handler(tauri::generate_handler![commands::desktop_request])
        .setup(move |app| {
            tray::install(app.handle(), Arc::clone(&state))?;
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
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                let state = window.state::<Arc<service::DesktopState>>();
                if state
                    .settings
                    .read()
                    .map(|settings| settings.close_to_tray)
                    .unwrap_or(true)
                {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .run(tauri::generate_context!())?;
    Ok(())
}
