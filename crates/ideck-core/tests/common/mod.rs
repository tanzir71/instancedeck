#![allow(dead_code)]
use ideck_core::{process::ProcessInfo, *};
use std::{collections::BTreeSet, path::Path, sync::Mutex, time::Duration};

#[derive(Default)]
pub struct FakeState {
    pub processes: Vec<ProcessInfo>,
    pub attempts: Vec<Strategy>,
    pub directories: Vec<String>,
    pub existing_paths: BTreeSet<String>,
    pub events: Vec<&'static str>,
    pub fail_direct: bool,
    pub fail_package: bool,
    pub fail_force: bool,
    pub no_process: bool,
    pub stubborn: bool,
    pub missing_app: bool,
    pub update_exe: bool,
    pub reused_pid_on_close: bool,
    pub waits: usize,
    pub refreshes: usize,
}
#[derive(Default)]
pub struct FakePlatform(pub Mutex<FakeState>);
struct FakeLock;
impl LaunchLock for FakeLock {}
pub fn app(recipe: &Recipe, updated: bool) -> InstalledApp {
    let install = if updated {
        r"C:\Apps\updated"
    } else {
        r"C:\Apps\official"
    };
    InstalledApp {
        app_id: recipe.id.clone(),
        name: recipe.name.clone(),
        version: if updated { "2" } else { "1" }.into(),
        executable: format!("{install}\\app\\{}", recipe.process_match.exe_name),
        install_dir: install.into(),
        install_type: "msix".into(),
        package_family: Some("Fixture_Family".into()),
        package_app_id: Some("App".into()),
    }
}
impl Platform for FakePlatform {
    fn directories(&self) -> Result<Directories> {
        Ok(Directories {
            local_app_data: r"C:\Users\Zoë Example\AppData\Local".into(),
            app_data: r"C:\Users\Zoë Example\AppData\Roaming".into(),
            user_profile: r"C:\Users\Zoë Example".into(),
        })
    }
    fn discover(&self, recipe: &Recipe, refresh: bool) -> Result<InstalledApp> {
        let mut state = self.0.lock().unwrap();
        if refresh {
            state.refreshes += 1;
        }
        if state.missing_app {
            return Err(Error::NotFound("Fixture app".into()));
        }
        Ok(app(recipe, refresh && state.update_exe))
    }
    fn exists(&self, path: &str) -> Result<bool> {
        Ok(self.0.lock().unwrap().existing_paths.contains(path))
    }
    fn ensure_directory(&self, path: &str) -> Result<()> {
        self.0.lock().unwrap().directories.push(path.into());
        Ok(())
    }
    fn launch_lock(&self) -> Result<Box<dyn LaunchLock + '_>> {
        Ok(Box::new(FakeLock))
    }
    fn launch(&self, plan: &LaunchPlan, strategy: Strategy) -> Result<u32> {
        let mut state = self.0.lock().unwrap();
        state.attempts.push(strategy);
        state.events.push("launch");
        if (strategy == Strategy::Direct
            && state.fail_direct
            && !(state.update_exe && plan.app.version == "2"))
            || (strategy == Strategy::PackageContext && state.fail_package)
        {
            return Err(Error::Platform("Fixture access denied (5)".into()));
        }
        if !state.no_process {
            state.processes.push(ProcessInfo {
                pid: 77,
                parent_pid: None,
                executable: plan.app.executable.clone(),
                args: plan.args.clone(),
                started_at: 100,
            });
        }
        Ok(77)
    }
    fn processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(self.0.lock().unwrap().processes.clone())
    }
    fn request_close(&self, processes: &[ProcessInfo]) -> Result<()> {
        let mut state = self.0.lock().unwrap();
        state.events.push("graceful-close");
        if state.reused_pid_on_close {
            for process in &mut state.processes {
                process.started_at += 1;
            }
        } else if !state.stubborn {
            state
                .processes
                .retain(|p| !processes.iter().any(|old| old.pid == p.pid));
        }
        Ok(())
    }
    fn force_close(&self, processes: &[ProcessInfo]) -> Result<()> {
        let mut state = self.0.lock().unwrap();
        state.events.push("force-close");
        if state.fail_force {
            return Ok(());
        }
        state
            .processes
            .retain(|p| !processes.iter().any(|old| old.pid == p.pid));
        Ok(())
    }
    fn focus(&self, _: &[ProcessInfo]) -> Result<()> {
        self.0.lock().unwrap().events.push("focus");
        Ok(())
    }
    fn wait(&self, _: Duration) {
        self.0.lock().unwrap().waits += 1;
    }
}

pub fn profile(app_id: &str) -> Profile {
    Profile::new(
        app_id,
        "Client — 日本語",
        Path::new(r"D:\Profiles with spaces\Zoë"),
        ProfileColor::Teal,
    )
    .unwrap()
}
pub fn fixture() -> (tempfile::TempDir, Engine<FakePlatform>, Profile) {
    let temp = tempfile::tempdir().unwrap();
    let store = ProfileStore::new(temp.path());
    let profile = store.create(profile("claude-desktop")).unwrap();
    let engine = Engine::new(FakePlatform::default(), store, built_in_recipes().unwrap()).unwrap();
    (temp, engine, profile)
}
