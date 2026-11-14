//! Pure update policy. No network operation lives in this crate.
use crate::{Error, Result, Settings, settings::UpdateChannel};
pub fn permitted(settings: &Settings) -> Result<()> {
    if !settings.onboarding_completed || !settings.allow_update_checks {
        return Err(Error::Invalid(
            "Enable update checks after first-run before contacting GitHub".into(),
        ));
    }
    Ok(())
}
pub fn endpoint(channel: &UpdateChannel) -> &'static str {
    match channel {
        UpdateChannel::Stable => {
            "https://github.com/tanzir71/instancedeck/releases/latest/download/latest.json"
        }
        UpdateChannel::Beta => {
            "https://github.com/tanzir71/instancedeck/releases/download/updater-beta/latest.json"
        }
    }
}
pub fn trusted_download(url: &str) -> bool {
    url.len() < 512
        && url.starts_with("https://github.com/tanzir71/instancedeck/releases/download/v")
        && url.ends_with(".exe")
        && !url.contains(['\\', '?', '#', '%', '@'])
        && !url.split('/').any(|part| matches!(part, "." | ".."))
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn first_run_and_disabled_preferences_deny_network_and_urls_stay_in_our_release_namespace() {
        let mut settings = Settings::defaults(std::path::Path::new(r"C:\Manager"));
        assert!(permitted(&settings).is_err());
        settings.onboarding_completed = true;
        assert!(permitted(&settings).is_ok());
        settings.allow_update_checks = false;
        assert!(permitted(&settings).is_err());
        assert!(trusted_download(
            "https://github.com/tanzir71/instancedeck/releases/download/v0.1.0/InstanceDeck_0.1.0_x64-setup.exe"
        ));
        for url in [
            "http://github.com/tanzir71/instancedeck/releases/download/v1/a.exe",
            "https://github.com/other/repo/releases/download/v1/a.exe",
            "https://github.com/tanzir71/instancedeck/releases/download/v1/../a.exe",
            "https://github.com/tanzir71/instancedeck/releases/download/v1/a.exe?redirect=other",
            "https://github.com/tanzir71/instancedeck/releases/download/v1/a%2f.exe",
            "file:///C:/a.exe",
        ] {
            assert!(!trusted_download(url));
        }
        assert_ne!(
            endpoint(&UpdateChannel::Stable),
            endpoint(&UpdateChannel::Beta)
        );
    }
}
