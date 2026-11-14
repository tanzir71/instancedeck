use crate::service::{CreateProfile, DesktopState, MainAction, ProfileAction, SignInAction};
use ideck_core::{Platform, Profile, Settings};
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
    AddCustom {
        input: ideck_core::custom::CustomApp,
    },
    PreviewCustom {
        input: ideck_core::custom::CustomApp,
        root: String,
    },
    PickImport,
    ImportProfiles {
        document: String,
    },
    ExportProfiles,
    SharedHistory {
        id: String,
        enabled: bool,
    },
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
    Shortcuts {
        id: String,
        locations: Vec<ideck_core::shell::ShortcutLocation>,
    },
    ConfirmAccount {
        id: String,
        matches: bool,
    },
    Window {
        action: WindowAction,
        label: Option<String>,
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
                Request::Shortcuts { id, locations } => {
                    Ok(serde_json::to_value(state.shortcuts(&id, &locations)?)?)
                }
                Request::Plan { id } => {
                    let profile = state.engine.store.read()?.profile(&id)?.clone();
                    Ok(serde_json::to_value(state.engine.plan(&profile, false)?)?)
                }
                Request::SharedHistory { id, enabled } => {state.shared_history(&id,enabled)?;Ok(Value::Null)}
                Request::AddCustom { input } => state.add_custom(input),
                Request::PreviewCustom { input, root } => {
                    input.recipe()?;
                    ideck_core::paths::validate_absolute(&root)?;
                    let context = ideck_core::template::Context { profile: format!("{root}\\{}\\01M7CMM9SCQVK72JN29KCEXWQT", input.id), install_dir: input.executable.rsplit_once(['\\','/']).map_or("", |p| p.0).into(), exe_dir: input.executable.rsplit_once(['\\','/']).map_or("", |p| p.0).into(), user_profile: state.engine.platform.directories()?.user_profile, local_app_data: state.engine.platform.directories()?.local_app_data };
                    let args=input.args.iter().map(|a|ideck_core::template::expand(a,&context)).collect::<ideck_core::Result<Vec<_>>>()?;
                    let command_line=std::iter::once(input.executable.as_str()).chain(args.iter().map(String::as_str)).map(ideck_win::command::quote_windows).collect::<Vec<_>>().join(" ");
                    Ok(json!({"app": {"executable": input.executable,"version":"Preview"}, "commandLine":command_line,"args":args, "env": input.env.iter().map(|(k,v)| Ok((k.clone(),ideck_core::template::expand(v,&context)?))).collect::<ideck_core::Result<std::collections::BTreeMap<_,_>>>()?, "workingDir": context.exe_dir}))
                }
                Request::PickImport => {
                    let Some(path) = app.dialog().file().add_filter("InstanceDeck profiles metadata", &["json"]).blocking_pick_file().and_then(|p| p.into_path().ok()) else { return Ok(Value::Null); };
                    if path.file_name().and_then(|n| n.to_str()) != Some("profiles.json") { return Err(ideck_core::Error::Invalid("Select an InstanceDeck export named profiles.json".into())); }
                    let metadata = std::fs::symlink_metadata(&path).map_err(|e| ideck_core::error::io(&path,e))?;
                    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 32*1024*1024 { return Err(ideck_core::Error::Invalid("Select a regular profiles.json metadata file below 32 MB".into())); }
                    let document = std::fs::read_to_string(&path).map_err(|e| ideck_core::error::io(&path,e))?;
                    let review = state.import_profiles(&document, false)?;
                    Ok(json!({"document": document, "added": review["added"], "skipped": review["skipped"]}))
                }
                Request::ImportProfiles { document } => state.import_profiles(&document, true),
                Request::ExportProfiles => {
                    let Some(path) = app.dialog().file().set_file_name("profiles.json").add_filter("InstanceDeck metadata", &["json"]).blocking_save_file().and_then(|p| p.into_path().ok()) else { return Ok(Value::Null); };
                    if path.file_name().and_then(|n| n.to_str()) != Some("profiles.json") { return Err(ideck_core::Error::Invalid("Save with the filename profiles.json".into())); }
                    let document = state.export_profiles()?;
                    if path.try_exists().map_err(|e| ideck_core::error::io(&path,e))? { std::fs::rename(&path, path.with_file_name(format!("profiles.retained-{}.json", ulid::Ulid::generate()))).map_err(|e| ideck_core::error::io(&path,e))?; }
                    use std::io::Write;
                    let mut file = std::fs::OpenOptions::new().write(true).create_new(true).open(&path).map_err(|e| ideck_core::error::io(&path,e))?;
                    file.write_all(document.as_bytes()).and_then(|()| file.sync_all()).map_err(|e| ideck_core::error::io(&path,e))?;
                    Ok(json!(path.to_string_lossy()))
                }
                Request::Browse => Ok(json!(
                    app.dialog()
                        .file()
                        .blocking_pick_folder()
                        .and_then(|path| path.into_path().ok())
                        .map(|path| path.to_string_lossy().into_owned())
                )),
                Request::Window { action, label } => {
                    let label = label.as_deref().unwrap_or("main");
                    if !matches!(label, "main" | "switcher") {
                        return Err(ideck_core::Error::Invalid("Unknown manager window".into()));
                    }
                    let window = app
                        .get_webview_window(label)
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
