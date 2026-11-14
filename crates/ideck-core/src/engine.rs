use crate::{
    Error, InstalledApp, LaunchPlan, Platform, Profile, ProfileStore, Recipe, Result, Strategy,
    process::{self, ProcessInfo},
    template::{self, Context},
};
use serde::Serialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    time::Duration,
};

pub struct Engine<P: Platform> {
    pub platform: P,
    pub store: ProfileStore,
    recipes: Vec<Recipe>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LaunchOutcome {
    pub profile_id: String,
    pub pid: u32,
    pub already_running: bool,
    pub strategy: Option<Strategy>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppDetection {
    pub recipe: Recipe,
    pub installed: Option<InstalledApp>,
    pub error: Option<String>,
}

impl<P: Platform> Engine<P> {
    pub fn new(platform: P, store: ProfileStore, recipes: Vec<Recipe>) -> Result<Self> {
        let mut ids = BTreeSet::new();
        for recipe in &recipes {
            recipe.validate()?;
            if !ids.insert(&recipe.id) {
                return Err(Error::Invalid("Duplicate recipe id".into()));
            }
        }
        Ok(Self {
            platform,
            store,
            recipes,
        })
    }
    pub fn recipes(&self) -> &[Recipe] {
        &self.recipes
    }
    pub fn recipe(&self, id: &str) -> Result<&Recipe> {
        self.recipes
            .iter()
            .find(|recipe| recipe.id == id)
            .ok_or_else(|| Error::NotFound("Recipe".into()))
    }
    pub fn apps(&self, refresh: bool) -> Vec<AppDetection> {
        self.recipes
            .iter()
            .map(|recipe| match self.platform.discover(recipe, refresh) {
                Ok(app) => AppDetection {
                    recipe: recipe.clone(),
                    installed: Some(app),
                    error: None,
                },
                Err(error) => AppDetection {
                    recipe: recipe.clone(),
                    installed: None,
                    error: Some(error.to_string()),
                },
            })
            .collect()
    }
    pub fn plan(&self, profile: &Profile, refresh: bool) -> Result<LaunchPlan> {
        profile.validate()?;
        let recipe = self.recipe(&profile.app_id)?;
        let app = self.platform.discover(recipe, refresh)?;
        let directories = self.platform.directories()?;
        let exe_dir = app
            .executable
            .rsplit_once(['\\', '/'])
            .map(|(parent, _)| parent)
            .ok_or_else(|| Error::Invalid("Executable has no parent directory".into()))?
            .to_owned();
        let context = Context {
            profile: profile.data_dir.clone(),
            install_dir: app.install_dir.clone(),
            exe_dir: exe_dir.clone(),
            user_profile: directories.user_profile,
            local_app_data: directories.local_app_data,
        };
        let args = recipe
            .launch
            .args
            .iter()
            .map(|arg| template::expand(arg, &context))
            .collect::<Result<Vec<_>>>()?;
        let mut env = BTreeMap::new();
        let mut needed_dirs = BTreeSet::from([profile.data_dir.clone()]);
        for (key, value) in &recipe.launch.env {
            if key == "CLAUDE_CONFIG_DIR" && !profile.options.isolate_claude_code {
                continue;
            }
            let optional = value.ends_with('?');
            let source = value.strip_suffix('?').unwrap_or(value);
            let resolved = template::expand(source, &context)?;
            if optional && !self.platform.exists(&resolved)? {
                continue;
            }
            if source.starts_with("{profile}") {
                needed_dirs.insert(resolved.clone());
            }
            env.insert(key.clone(), resolved);
        }
        for source in &recipe.launch.args {
            let value = source
                .split_once('=')
                .map_or(source.as_str(), |(_, value)| value);
            if value.starts_with("{profile}") {
                needed_dirs.insert(template::expand(value, &context)?);
            }
        }
        for directory in &needed_dirs {
            if !crate::paths::is_within_windows(directory, &profile.data_dir) {
                return Err(Error::Invalid(
                    "A recipe data directory escapes the profile root".into(),
                ));
            }
        }
        let match_argument = template::expand(&recipe.process_match.arg, &context)?;
        Ok(LaunchPlan {
            profile_id: profile.id.clone(),
            recipe_id: recipe.id.clone(),
            recipe_version: recipe.recipe_version,
            app,
            args,
            env,
            working_dir: exe_dir,
            directories: needed_dirs.into_iter().collect(),
            match_argument,
        })
    }
    fn matching(&self, plan: &LaunchPlan, processes: &[ProcessInfo]) -> Result<Vec<ProcessInfo>> {
        let matcher = &self.recipe(&plan.recipe_id)?.process_match;
        let roots: Vec<_> = processes
            .iter()
            .filter(|p| process::matches_profile(p, &plan.app, matcher, &plan.match_argument))
            .map(|p| p.pid)
            .collect();
        Ok(process::tree(processes, &roots))
    }
    pub fn running(&self, profile: &Profile) -> Result<Vec<ProcessInfo>> {
        let plan = self.plan(profile, false)?;
        self.matching(&plan, &self.platform.processes()?)
    }
    pub fn main_processes(&self, app_id: &str) -> Result<Vec<ProcessInfo>> {
        let recipe = self.recipe(app_id)?;
        let app = self.platform.discover(recipe, false)?;
        let processes = self.platform.processes()?;
        let mut managed = BTreeSet::new();
        for profile in self
            .store
            .read()?
            .profiles
            .iter()
            .filter(|p| p.app_id == app_id)
        {
            let plan = self.plan(profile, false)?;
            managed.extend(self.matching(&plan, &processes)?.iter().map(|p| p.pid));
        }
        let roots: Vec<_> = processes
            .iter()
            .filter(|p| {
                !managed.contains(&p.pid) && process::matches_main(p, &app, &recipe.process_match)
            })
            .map(|p| p.pid)
            .collect();
        Ok(process::tree(&processes, &roots)
            .into_iter()
            .filter(|p| !managed.contains(&p.pid))
            .collect())
    }
    pub fn launch(&self, id: &str) -> Result<LaunchOutcome> {
        let _lock = self.platform.launch_lock()?;
        let profile = self.store.read()?.profile(id)?.clone();
        crate::signin::guard_launch(self.store.root(), &profile.app_id)?;
        let mut plan = self.plan(&profile, false)?;
        let existing = self.matching(&plan, &self.platform.processes()?)?;
        if let Some(process) = existing.first() {
            self.platform.focus(&existing)?;
            return Ok(LaunchOutcome {
                profile_id: id.into(),
                pid: process.pid,
                already_running: true,
                strategy: None,
            });
        }
        for path in &plan.directories {
            self.platform.ensure_directory(path)?;
        }
        let recipe = self.recipe(&profile.app_id)?;
        let mut tried = Vec::new();
        let mut failure = String::new();
        for strategy in &recipe.launch.strategies {
            tried.push(strategy.to_string());
            match self.try_launch(&plan, *strategy) {
                Ok(pid) => return self.finish_launch(id, &plan, *strategy, pid),
                Err(error) => {
                    failure = error.to_string();
                    tracing::warn!(recipe_id = %recipe.id, recipe_version = recipe.recipe_version, app_version = %plan.app.version, strategy = %strategy, exit_status = "failed", "launch attempt");
                }
            }
            if *strategy == Strategy::Direct {
                // One fresh resolution after failure; retry direct only if an update changed the exe.
                if let Ok(refreshed) = self.plan(&profile, true)
                    && refreshed.app.executable != plan.app.executable
                {
                    plan = refreshed;
                    tried.push("direct-after-update".into());
                    match self.try_launch(&plan, Strategy::Direct) {
                        Ok(pid) => return self.finish_launch(id, &plan, Strategy::Direct, pid),
                        Err(error) => failure = error.to_string(),
                    }
                }
            }
        }
        Err(Error::Launch {
            strategies: tried.join(", "),
            detail: failure,
        })
    }
    fn try_launch(&self, plan: &LaunchPlan, strategy: Strategy) -> Result<u32> {
        if strategy == Strategy::PackageContext && plan.app.package_family.is_none() {
            return Err(Error::Invalid(
                "Package-context launch requires an MSIX package".into(),
            ));
        }
        self.platform.launch(plan, strategy)?;
        // A successful CreateProcess is insufficient: require a tracked isolated process.
        for attempt in 0..40 {
            let processes = self.platform.processes()?;
            if let Some(process) = self.matching(plan, &processes)?.first() {
                return Ok(process.pid);
            }
            if attempt < 39 {
                self.platform.wait(Duration::from_millis(500));
            }
        }
        Err(Error::Platform(
            "No isolated process appeared within 20 seconds; no process was killed".into(),
        ))
    }
    fn finish_launch(
        &self,
        id: &str,
        plan: &LaunchPlan,
        strategy: Strategy,
        pid: u32,
    ) -> Result<LaunchOutcome> {
        self.store.mark_launched(id)?;
        tracing::info!(recipe_id = %plan.recipe_id, recipe_version = plan.recipe_version, app_version = %plan.app.version, strategy = %strategy, exit_status = "running", "launch");
        Ok(LaunchOutcome {
            profile_id: id.into(),
            pid,
            already_running: false,
            strategy: Some(strategy),
        })
    }
    pub fn close(&self, id: &str) -> Result<()> {
        let profile = self.store.read()?.profile(id)?.clone();
        self.close_processes(self.running(&profile)?)
    }
    pub fn close_main(&self, app_id: &str) -> Result<()> {
        crate::signin::guard_launch(self.store.root(), app_id)?;
        self.close_processes(self.main_processes(app_id)?)
    }
    pub(crate) fn close_main_for_sign_in(&self) -> Result<()> {
        self.close_processes(self.main_processes("claude-desktop")?)
    }
    pub(crate) fn launch_main_for_sign_in(&self) -> Result<()> {
        self.launch_main_locked("claude-desktop")
    }
    pub fn launch_main(&self, app_id: &str) -> Result<()> {
        let _lock = self.platform.launch_lock()?;
        crate::signin::guard_launch(self.store.root(), app_id)?;
        self.launch_main_locked(app_id)
    }
    fn launch_main_locked(&self, app_id: &str) -> Result<()> {
        let current = self.main_processes(app_id)?;
        if !current.is_empty() {
            return self.platform.focus(&current);
        }
        let recipe = self.recipe(app_id)?;
        let app = self.platform.discover(recipe, true)?;
        let working_dir = app
            .executable
            .rsplit_once(['\\', '/'])
            .ok_or_else(|| Error::Invalid("Executable has no parent".into()))?
            .0
            .to_owned();
        let plan = LaunchPlan {
            profile_id: "main".into(),
            recipe_id: app_id.into(),
            recipe_version: recipe.recipe_version,
            app,
            args: Vec::new(),
            env: BTreeMap::new(),
            working_dir,
            directories: Vec::new(),
            match_argument: String::new(),
        };
        let mut failure = String::new();
        for strategy in &recipe.launch.strategies {
            if *strategy == Strategy::PackageContext && plan.app.package_family.is_none() {
                continue;
            }
            match self.platform.launch(&plan, *strategy) {
                Ok(_) => {
                    for _ in 0..40 {
                        if !self.main_processes(app_id)?.is_empty() {
                            return Ok(());
                        }
                        self.platform.wait(Duration::from_millis(500));
                    }
                    failure = "No main process appeared within 20 seconds".into();
                }
                Err(error) => failure = error.to_string(),
            }
        }
        Err(Error::Launch {
            strategies: recipe
                .launch
                .strategies
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(", "),
            detail: failure,
        })
    }
    fn close_processes(&self, processes: Vec<ProcessInfo>) -> Result<()> {
        if processes.is_empty() {
            return Ok(());
        }
        if processes.iter().any(|p| p.started_at == 0) {
            return Err(Error::Platform(
                "Cannot prove process creation time; close was refused".into(),
            ));
        }
        self.platform.request_close(&processes)?;
        for attempt in 0..17 {
            if self.remaining(&processes)?.is_empty() {
                return Ok(());
            }
            if attempt < 16 {
                self.platform.wait(Duration::from_millis(500));
            }
        }
        Err(Error::StillRunning)
    }
    fn remaining(&self, original: &[ProcessInfo]) -> Result<Vec<ProcessInfo>> {
        let current = self.platform.processes()?;
        // Start time and exe prevent PID reuse from granting authority over another process.
        let roots: Vec<_> = current
            .iter()
            .filter(|p| {
                original.iter().any(|old| {
                    old.pid == p.pid
                        && old.started_at == p.started_at
                        && crate::paths::normalize_windows(&old.executable)
                            == crate::paths::normalize_windows(&p.executable)
                })
            })
            .map(|p| p.pid)
            .collect();
        Ok(process::tree(&current, &roots))
    }
    /// The caller must obtain explicit confirmation; graceful close is always attempted first.
    pub fn force_close(&self, id: &str, confirmed: bool) -> Result<()> {
        if !confirmed {
            return Err(Error::Invalid(
                "Force close requires explicit confirmation".into(),
            ));
        }
        let profile = self.store.read()?.profile(id)?.clone();
        let original = self.running(&profile)?;
        match self.close_processes(original.clone()) {
            Ok(()) => Ok(()),
            Err(Error::StillRunning) => {
                self.platform.force_close(&self.remaining(&original)?)?;
                for attempt in 0..17 {
                    if self.running(&profile)?.is_empty() {
                        return Ok(());
                    }
                    if attempt < 16 {
                        self.platform.wait(Duration::from_millis(500));
                    }
                }
                Err(Error::StillRunning)
            }
            Err(error) => Err(error),
        }
    }
}
