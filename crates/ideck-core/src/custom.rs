//! InstanceDeck-owned custom recipe metadata and launch identity receipts.
use crate::{
    Error, LaunchPlan, Recipe, Result,
    error::io,
    paths,
    process::{self, ProcessInfo},
    recipe::*,
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "bindings", derive(specta::Type))]
pub struct CustomApp {
    pub id: String,
    pub name: String,
    pub executable: String,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    /// Empty for environment-only launch. Otherwise a launch argument or flag=value pair.
    pub match_argument: String,
}
impl CustomApp {
    pub fn recipe(&self) -> Result<Recipe> {
        paths::validate_absolute(&self.executable)?;
        if self.executable.contains(['{', '}'])
            || !self.id.starts_with("custom-")
            || !valid_id(&self.id)
            || self.name.chars().count() > 120
            || self.name.chars().any(char::is_control)
            || !self.executable.to_ascii_lowercase().ends_with(".exe")
            || self.args.len() > 64
            || self.env.len() > 32
            || self.args.iter().any(|a| a.len() > 8192)
            || self.env.iter().any(|(k, v)| {
                v.len() > 8192
                    || ["TOKEN", "PASSWORD", "SECRET", "COOKIE", "AUTH"]
                        .iter()
                        .any(|s| k.to_ascii_uppercase().contains(s))
            })
        {
            return Err(Error::Invalid("Use a custom- id, a name, an absolute .exe path, bounded arguments and non-secret environment variables".into()));
        }
        let recipe = Recipe {
            id: self.id.clone(), recipe_version: 1, name: self.name.trim().into(),
            vendor: "User configured".into(), category: "custom".into(),
            discovery: vec![Discovery::KnownPath { path: self.executable.clone(), version_prefix: None }],
            launch: Launch { args: self.args.clone(), env: self.env.clone(), strategies: vec![Strategy::Direct] },
            isolation: Isolation::Unknown, sign_in: SignIn { strategy: SignInStrategy::Manual },
            process_match: ProcessMatch { exe_name: self.executable.rsplit(['\\','/']).next().unwrap_or_default().into(), arg: self.match_argument.clone() },
            limitations: vec!["User-configured, unverified isolation. Check the visible data path and account. Environment-only tracking follows only the process created by InstanceDeck and its proven children; singleton hand-offs cannot be tracked.".into()],
            verified_on: vec![], status: RecipeStatus::Experimental,
            seo: Seo { slug: self.id.clone(), title: self.name.clone(), description: "Local custom recipe".into(), intro: "User-configured app".into() },
        };
        recipe.validate()?;
        Ok(recipe)
    }
}
pub fn load(root: &Path) -> Result<Vec<Recipe>> {
    let directory = root.join("recipes");
    if !directory.try_exists().map_err(|e| io(&directory, e))? {
        return Ok(vec![]);
    }
    let mut recipes = Vec::new();
    for entry in fs::read_dir(&directory).map_err(|e| io(&directory, e))? {
        let path = entry.map_err(|e| io(&directory, e))?.path();
        if !path
            .file_name()
            .is_some_and(|n| n.to_string_lossy().ends_with(".recipe.json"))
        {
            continue;
        }
        let metadata = fs::symlink_metadata(&path).map_err(|e| io(&path, e))?;
        if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 128 * 1024 {
            return Err(Error::Invalid(
                "Custom recipe must be regular metadata smaller than 128 KB".into(),
            ));
        }
        let recipe = Recipe::from_json(&fs::read_to_string(&path).map_err(|e| io(&path, e))?)?;
        if !recipe.id.starts_with("custom-") || recipes.iter().any(|r: &Recipe| r.id == recipe.id) {
            return Err(Error::Invalid(
                "Invalid or duplicate custom recipe id".into(),
            ));
        }
        recipes.push(recipe);
        if recipes.len() > 256 {
            return Err(Error::Invalid(
                "At most 256 custom recipes are supported".into(),
            ));
        }
    }
    Ok(recipes)
}
pub fn save(root: &Path, recipe: &Recipe) -> Result<()> {
    use fs2::FileExt;
    let directory = root.join("recipes");
    fs::create_dir_all(&directory).map_err(|e| io(&directory, e))?;
    let lock_path = directory.join("recipes.lock");
    let lock = OpenOptions::new()
        .create(true)
        .truncate(false)
        .read(true)
        .write(true)
        .open(&lock_path)
        .map_err(|e| io(&lock_path, e))?;
    FileExt::lock_exclusive(&lock).map_err(|e| io(&lock_path, e))?;
    if load(root)?.len() >= 256 {
        return Err(Error::Invalid(
            "At most 256 custom recipes are supported".into(),
        ));
    }
    let path = directory.join(format!("{}.recipe.json", recipe.id));
    if path.try_exists().map_err(|e| io(&path, e))? {
        return Err(Error::Invalid("Custom app id already exists".into()));
    }
    let staged = directory.join(format!("pending-{}.json", ulid::Ulid::generate()));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&staged)
        .map_err(|e| io(&staged, e))?;
    file.write_all((serde_json::to_string_pretty(recipe)? + "\n").as_bytes())
        .and_then(|()| file.sync_all())
        .map_err(|e| io(&staged, e))?;
    drop(file);
    fs::rename(&staged, &path).map_err(|e| io(&path, e))
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    pid: u32,
    started: u64,
    executable: String,
}
pub fn record_owned(root: &Path, plan: &LaunchPlan, pid: u32, started: u64) -> Result<()> {
    if pid == 0 || started == 0 {
        return Err(Error::Invalid("Unproven launch identity".into()));
    }
    let directory = root.join("owned-processes");
    fs::create_dir_all(&directory).map_err(|e| io(&directory, e))?;
    let path = directory.join(format!("{}.json", plan.profile_id));
    if path.try_exists().map_err(|e| io(&path, e))? {
        fs::rename(
            &path,
            directory.join(format!(
                "{}.retained-{}.json",
                plan.profile_id,
                ulid::Ulid::generate()
            )),
        )
        .map_err(|e| io(&path, e))?;
    }
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(&path)
        .map_err(|e| io(&path, e))?;
    file.write_all(
        serde_json::to_string(&Receipt {
            pid,
            started,
            executable: plan.app.executable.clone(),
        })?
        .as_bytes(),
    )
    .and_then(|()| file.sync_all())
    .map_err(|e| io(&path, e))
}
pub fn owned_tree(
    root: &Path,
    plan: &LaunchPlan,
    processes: &[ProcessInfo],
) -> Result<Vec<ProcessInfo>> {
    let path = root
        .join("owned-processes")
        .join(format!("{}.json", plan.profile_id));
    let metadata = match fs::symlink_metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(io(&path, e)),
    };
    if !metadata.is_file() || metadata.file_type().is_symlink() || metadata.len() > 16 * 1024 {
        return Err(Error::Invalid("Invalid process receipt".into()));
    }
    let receipt: Receipt =
        serde_json::from_str(&fs::read_to_string(&path).map_err(|e| io(&path, e))?)?;
    let roots: Vec<u32> = processes
        .iter()
        .filter(|p| {
            p.pid == receipt.pid
                && receipt.started != 0
                && p.started_at == receipt.started
                && paths::normalize_windows(&p.executable)
                    == paths::normalize_windows(&receipt.executable)
                && paths::normalize_windows(&p.executable)
                    == paths::normalize_windows(&plan.app.executable)
        })
        .map(|p| p.pid)
        .collect();
    Ok(process::tree(processes, &roots))
}
