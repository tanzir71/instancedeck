use crate::{Recipe, Result, Strategy, process::ProcessInfo};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Directories {
    pub local_app_data: String,
    pub app_data: String,
    pub user_profile: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InstalledApp {
    pub app_id: String,
    pub name: String,
    pub version: String,
    pub executable: String,
    pub install_dir: String,
    pub install_type: String,
    pub package_family: Option<String>,
    pub package_app_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchPlan {
    pub profile_id: String,
    pub recipe_id: String,
    pub recipe_version: u32,
    pub app: InstalledApp,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub working_dir: String,
    pub directories: Vec<String>,
    pub match_argument: String,
}

pub trait LaunchLock {}

pub trait DesktopOperations: Send + Sync {
    fn open_folder(&self, path: &str) -> Result<()>;
    /// Caller must have verified an exact profile-name confirmation and stopped state.
    fn recycle_profile(&self, profile: &crate::Profile) -> Result<()>;
}

/// Every Windows operation is behind this interface so the engine can run with fakes.
pub trait Platform: Send + Sync {
    fn track_profile_windows(&self, _: &crate::Profile, _: &[ProcessInfo]) -> Result<()> {
        Ok(())
    }
    fn directories(&self) -> Result<Directories>;
    fn discover(&self, recipe: &Recipe, refresh: bool) -> Result<InstalledApp>;
    fn exists(&self, path: &str) -> Result<bool>;
    fn ensure_directory(&self, path: &str) -> Result<()>;
    fn launch_lock(&self) -> Result<Box<dyn LaunchLock + '_>>;
    fn launch(&self, plan: &LaunchPlan, strategy: Strategy) -> Result<u32>;
    fn processes(&self) -> Result<Vec<ProcessInfo>>;
    fn request_close(&self, processes: &[ProcessInfo]) -> Result<()>;
    fn force_close(&self, processes: &[ProcessInfo]) -> Result<()>;
    fn focus(&self, processes: &[ProcessInfo]) -> Result<()>;
    fn wait(&self, duration: std::time::Duration);
}
