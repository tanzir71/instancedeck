use crate::service::{CreateProfile, DesktopState, MainAction, ProfileAction, SignInAction};
use ideck_core::{Profile, Settings};
use serde::Deserialize;
use serde_json::{Value, json};
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Deserialize, specta::Type)]
#[serde(tag = "type", rename_all = "camelCase", deny_unknown_fields)]
pub enum Request {
    Snapshot {
        refresh: Option<bool>,
    },
    Create {
        input: CreateProfile,
    },
    Update {
        profile: Profile,
    },
    Action {
        id: String,
        action: ProfileAction,
        confirmation: Option<String>,
    },
    MainAction {
        #[serde(rename = "appId")]
        app_id: String,
        action: MainAction,
    },
    SaveSettings {
        settings: Settings,
    },
    Diagnostics {
        redact: bool,
    },
    Plan {
        id: String,
    },
    Browse,
    SignIn {
        id: String,
        action: SignInAction,
        #[serde(rename = "closeMainConfirmed")]
        close_main_confirmed: Option<bool>,
    },
    CopyMcp {
        from: String,
        to: String,
    },
    ConfirmAccount {
        id: String,
        matches: bool,
    },
    Window {
        action: WindowAction,
    },
}
#[derive(Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub enum WindowAction {
    Minimize,
    Maximize,
    Close,
}
#[tauri::command]
#[specta::specta]
pub async fn desktop_request(
    app: AppHandle,
    state: State<'_, Arc<DesktopState>>,
    request: Request,
) -> std::result::Result<Value, String> {
    let state = Arc::clone(state.inner());
    tauri::async_runtime::spawn_blocking(move || -> std::result::Result<Value, String> {
        let execute = || -> ideck_core::Result<Value> {
            match request {
                Request::Snapshot { refresh } => Ok(serde_json::to_value(
                    state.snapshot(refresh.unwrap_or(false))?,
                )?),
                Request::Create { input } => Ok(serde_json::to_value(state.create(input)?)?),
                Request::Update { profile } => {
                    state.update(profile)?;
                    Ok(Value::Null)
                }
                Request::Action {
                    id,
                    action,
                    confirmation,
                } => {
                    state.action(&id, action, confirmation.as_deref())?;
                    Ok(Value::Null)
                }
                Request::MainAction { app_id, action } => {
                    state.main_action(&app_id, action)?;
                    Ok(Value::Null)
                }
                Request::SaveSettings { settings } => {
                    state.save_settings(&settings)?;
                    Ok(Value::Null)
                }
                Request::Diagnostics { redact } => state.diagnostics(redact),
                Request::SignIn {
                    id,
                    action,
                    close_main_confirmed,
                } => state.sign_in(&id, action, close_main_confirmed.unwrap_or(false)),
                Request::CopyMcp { from, to } => {
                    state.copy_mcp(&from, &to)?;
                    Ok(Value::Null)
                }
                Request::ConfirmAccount { id, matches } => {
                    state.confirm_account(&id, matches)?;
                    Ok(Value::Null)
                }
                Request::Plan { id } => {
                    let profile = state.engine.store.read()?.profile(&id)?.clone();
                    Ok(serde_json::to_value(state.engine.plan(&profile, false)?)?)
                }
                Request::Browse => Ok(json!(
                    app.dialog()
                        .file()
                        .blocking_pick_folder()
                        .and_then(|path| path.into_path().ok())
                        .map(|path| path.to_string_lossy().into_owned())
                )),
                Request::Window { action } => {
                    let window = app
                        .get_webview_window("main")
                        .ok_or_else(|| ideck_core::Error::NotFound("Main window".into()))?;
                    match action {
                        WindowAction::Minimize => window.minimize(),
                        WindowAction::Maximize => {
                            if window.is_maximized().unwrap_or(false) {
                                window.unmaximize()
                            } else {
                                window.maximize()
                            }
                        }
                        WindowAction::Close => window.close(),
                    }
                    .map_err(|error| ideck_core::Error::Platform(error.to_string()))?;
                    Ok(Value::Null)
                }
            }
        };
        execute().map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}
