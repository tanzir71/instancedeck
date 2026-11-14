//! Explicit manual QA: launches one existing profile through the MSIX fallback.
//! Run only with a disposable profile id; no sign-in or credential files are read.
use clap::Parser;
use ideck_core::{Engine, Error, Platform, ProfileStore, Result, Strategy, built_in_recipes};
use ideck_win::WindowsPlatform;
use std::{path::PathBuf, time::Duration};

#[derive(Parser)]
struct Args {
    #[arg(long)]
    state_dir: PathBuf,
    id: String,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let engine = Engine::new(
        WindowsPlatform::new(&args.state_dir),
        ProfileStore::new(&args.state_dir),
        built_in_recipes()?,
    )?;
    let profile = engine.store.read()?.profile(&args.id)?.clone();
    if !engine.running(&profile)?.is_empty() {
        return Err(Error::Invalid(
            "The QA profile must be stopped before this explicit fallback probe".into(),
        ));
    }
    let plan = engine.plan(&profile, true)?;
    for directory in &plan.directories {
        engine.platform.ensure_directory(directory)?;
    }
    let _lock = engine.platform.launch_lock()?;
    let helper = engine.platform.launch(&plan, Strategy::PackageContext)?;
    for _ in 0..40 {
        let processes = engine.running(&profile)?;
        if !processes.is_empty() {
            println!(
                "{}",
                serde_json::json!({ "profileId": profile.id, "appId": profile.app_id, "appVersion": plan.app.version, "strategy": "package-context", "helperPid": helper, "runningPids": processes.iter().map(|p| p.pid).collect::<Vec<_>>() })
            );
            return Ok(());
        }
        engine.platform.wait(Duration::from_millis(500));
    }
    Err(Error::Platform(
        "The package-context QA probe did not produce a matching process within 20 seconds".into(),
    ))
}
