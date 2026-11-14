use crate::service::DesktopState;
use ideck_core::shell::ShellOperations;
use ideck_core::{Error, Result, Settings, SettingsStore, updates};
use serde::Serialize;
use std::{sync::Arc, time::Duration};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_updater::{Update, UpdaterExt};

type CachedUpdate = Option<(String, Update)>;
#[derive(Default)]
pub struct Updates {
    cached: tokio::sync::Mutex<CachedUpdate>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInfo {
    pub status: &'static str,
    pub version: Option<String>,
    pub notes: Option<String>,
    pub portable: bool,
    pub current_version: &'static str,
}
fn portable() -> bool {
    std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("portable-mode.txt")))
        .is_some_and(|p| p.exists())
}
fn info(status: &'static str, version: Option<String>, notes: Option<String>) -> UpdateInfo {
    UpdateInfo {
        status,
        version,
        notes,
        portable: portable(),
        current_version: env!("CARGO_PKG_VERSION"),
    }
}
impl Updates {
    async fn authorized<'a>(
        &'a self,
        store: &SettingsStore,
    ) -> Result<(tokio::sync::MutexGuard<'a, CachedUpdate>, Settings)> {
        // A queued request must use permission/channel saved after it acquires
        // the request lock, never a snapshot taken before an earlier request.
        let cached = self.cached.lock().await;
        let settings = store.read()?;
        updates::permitted(&settings)?;
        Ok((cached, settings))
    }
    pub async fn check(&self, app: AppHandle, state: Arc<DesktopState>) -> Result<UpdateInfo> {
        let (mut cached, settings) = self.authorized(&state.settings).await?;
        let channel = updates::endpoint(&settings.update_channel).to_owned();
        *cached = None;
        // The plugin itself makes no requests on registration. This is the only
        // manifest request, and both saved consent gates precede client creation.
        let stop = Arc::clone(&state);
        let updater = app
            .updater_builder()
            .endpoints(vec![channel.parse().map_err(|_| {
                Error::Invalid("Invalid built-in update endpoint".into())
            })?])
            .map_err(|e| Error::Platform(e.to_string()))?
            .timeout(Duration::from_secs(20))
            .restart_after_install(false)
            .on_before_exit(move || stop.engine.platform.stop_window_identity())
            .build()
            .map_err(|e| Error::Platform(e.to_string()))?;
        let checked = updater.check().await;
        let current = state.settings.read()?;
        updates::permitted(&current)?;
        if updates::endpoint(&current.update_channel) != channel {
            return Err(Error::Invalid("Update channel changed; check again".into()));
        }
        match checked {
            Ok(Some(update)) => {
                if !updates::trusted_download(update.download_url.as_str()) {
                    return Err(Error::Invalid(
                        "Update URL is outside this project's HTTPS release assets".into(),
                    ));
                }
                let result = info(
                    "available",
                    Some(update.version.clone()),
                    update.body.clone(),
                );
                *cached = Some((channel, update));
                Ok(result)
            }
            Ok(None) => Ok(info("current", None, None)),
            Err(tauri_plugin_updater::Error::ReleaseNotFound) => {
                Ok(info("unavailable", None, None))
            }
            Err(error) => Err(Error::Platform(format!("Update check: {error}"))),
        }
    }
    pub async fn install(
        &self,
        app: AppHandle,
        state: Arc<DesktopState>,
        version: String,
        confirmed: bool,
    ) -> Result<()> {
        if !confirmed {
            return Err(Error::Invalid(
                "Confirm the displayed update before downloading".into(),
            ));
        }
        if portable() {
            return Err(Error::Invalid("Portable builds update by extracting a new portable package. Your profiles stay in the configured folder.".into()));
        }
        let (mut cached, settings) = self.authorized(&state.settings).await?;
        let (channel, update) = cached
            .as_ref()
            .filter(|(channel, u)| {
                u.version == version && channel == updates::endpoint(&settings.update_channel)
            })
            .cloned()
            .ok_or_else(|| {
                Error::Invalid("Check again; this update is no longer the selected release".into())
            })?;
        let mut downloaded = 0u64;
        let bytes = update
            .download(
                |chunk, total| {
                    downloaded += chunk as u64;
                    let _ = app.emit(
                        "update-progress",
                        serde_json::json!({"downloaded":downloaded,"total":total}),
                    );
                },
                || {},
            )
            .await
            .map_err(|e| Error::Platform(format!("Update integrity/download: {e}")))?;
        // download() verifies the artifact and its signed version before returning.
        // Re-check consent/channel and journals after the network request too.
        let current = state.settings.read()?;
        updates::permitted(&current)?;
        if channel != updates::endpoint(&current.update_channel) {
            return Err(Error::Invalid(
                "Update channel changed; nothing was installed".into(),
            ));
        }
        *cached = None;
        tauri::async_runtime::spawn_blocking(move || {
            state.run_installer(&channel, || {
                update
                    .install(bytes)
                    .map_err(|e| Error::Platform(format!("Update installation: {e}")))
            })
        })
        .await
        .map_err(|e| Error::Platform(e.to_string()))?
    }
}
pub async fn check(app: AppHandle, state: Arc<DesktopState>) -> Result<UpdateInfo> {
    app.state::<Updates>().check(app.clone(), state).await
}
pub async fn install(
    app: AppHandle,
    state: Arc<DesktopState>,
    version: String,
    confirmed: bool,
) -> Result<()> {
    app.state::<Updates>()
        .install(app.clone(), state, version, confirmed)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn queued_requests_recheck_saved_consent_and_channel_after_waiting() {
        let temp = tempfile::tempdir().unwrap();
        let store = SettingsStore::new(temp.path().join("Manager Zoë 日本語"));
        let mut settings = store.read().unwrap();
        settings.onboarding_completed = true;
        store.write(&settings).unwrap();
        let updates = Updates::default();
        let held = updates.cached.lock().await;
        let attempt = updates.authorized(&store);
        tokio::pin!(attempt);
        assert!(
            tokio::time::timeout(Duration::from_millis(5), &mut attempt)
                .await
                .is_err()
        );
        settings.allow_update_checks = false;
        store.write(&settings).unwrap();
        drop(held);
        assert!(attempt.await.is_err());

        settings.allow_update_checks = true;
        store.write(&settings).unwrap();
        let held = updates.cached.lock().await;
        let attempt = updates.authorized(&store);
        tokio::pin!(attempt);
        assert!(
            tokio::time::timeout(Duration::from_millis(5), &mut attempt)
                .await
                .is_err()
        );
        settings.update_channel = ideck_core::settings::UpdateChannel::Beta;
        store.write(&settings).unwrap();
        drop(held);
        let (_guard, current) = attempt.await.unwrap();
        assert!(matches!(
            current.update_channel,
            ideck_core::settings::UpdateChannel::Beta
        ));
    }
}
