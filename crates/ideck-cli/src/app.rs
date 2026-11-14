use clap::{ArgAction, Parser, Subcommand};
use ideck_core::{
    Engine, Error, Platform, Profile, ProfileColor, ProfileStore, Result, built_in_recipes, layout,
    paths, recipe_schema,
};
use ideck_win::WindowsPlatform;
use serde::Serialize;
use serde_json::{Value, json};
use std::{
    fs,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Parser)]
#[command(
    name = "ideck",
    version,
    about = "Official apps, separate profiles. No credential access."
)]
struct Cli {
    #[arg(long, global = true)]
    state_dir: Option<PathBuf>,
    #[arg(long, global = true)]
    profiles_root: Option<PathBuf>,
    #[command(subcommand)]
    command: Command,
}
#[derive(Subcommand)]
enum Command {
    Apps {
        #[arg(long)]
        refresh: bool,
    },
    Profiles {
        #[command(subcommand)]
        command: Profiles,
    },
    Launch {
        id: String,
    },
    Close {
        id: String,
        #[arg(long)]
        force: bool,
    },
    /// Show the resolved arguments and environment without launching anything.
    Plan {
        id: String,
    },
    /// Metadata and manager-generated launch logs only; never app file contents.
    Doctor {
        #[arg(long, default_value_t = true, action = ArgAction::Set)]
        redact: bool,
    },
    /// Generate the recipe JSON schema from the actual Rust types.
    RecipeSchema {
        #[arg(long)]
        check: Option<PathBuf>,
    },
    /// Journaled Claude sign-in; never reads target file contents.
    SignIn {
        #[command(subcommand)]
        command: SignInCommand,
    },
}
#[derive(Subcommand)]
enum SignInCommand {
    List,
    Start {
        id: String,
        #[arg(long)]
        close_main: bool,
    },
    Resume {
        id: String,
    },
    Finish {
        id: String,
    },
    Restore {
        id: String,
    },
}
#[derive(Subcommand)]
enum Profiles {
    List,
    Create {
        app: String,
        name: String,
        #[arg(long, default_value = "graphite")]
        color: String,
    },
    Rename {
        id: String,
        name: String,
    },
    /// Remove the profile from the deck and keep every data file.
    Remove {
        id: String,
    },
}

pub fn run() -> Result<()> {
    let cli = Cli::parse();
    if let Command::RecipeSchema { check } = &cli.command {
        let schema = recipe_schema()?;
        if let Some(path) = check {
            if path.file_name().and_then(|name| name.to_str()) != Some("recipe.schema.json") {
                return Err(Error::Invalid(
                    "Schema checks accept only a generated recipe.schema.json file".into(),
                ));
            }
            let saved = fs::read_to_string(path).map_err(|e| ideck_core::error::io(path, e))?;
            if saved.replace("\r\n", "\n") != schema {
                return Err(Error::Invalid(
                    "Recipe schema is out of date; regenerate it with ideck recipe-schema".into(),
                ));
            }
            println!("Recipe schema matches Rust types.");
        } else {
            print!("{schema}");
        }
        return Ok(());
    }
    let state_dir = cli.state_dir.unwrap_or_else(|| {
        PathBuf::from(std::env::var_os("LOCALAPPDATA").unwrap_or_default()).join("InstanceDeck")
    });
    if !state_dir.is_absolute() {
        return Err(Error::Invalid(
            "State directory must be an absolute Windows path".into(),
        ));
    }
    let log_dir = state_dir.join("logs");
    fs::create_dir_all(&log_dir).map_err(|e| ideck_core::error::io(&log_dir, e))?;
    let appender = tracing_appender::rolling::daily(&log_dir, "launches.jsonl");
    let (writer, _guard) = tracing_appender::non_blocking(appender);
    tracing_subscriber::fmt()
        .json()
        .with_current_span(false)
        .with_span_list(false)
        .with_writer(writer)
        .with_max_level(tracing::Level::INFO)
        .try_init()
        .map_err(|e| Error::Platform(format!("Log initialization: {e}")))?;
    let platform = WindowsPlatform::new(&state_dir);
    let engine = Engine::new(platform, ProfileStore::new(&state_dir), built_in_recipes()?)?;
    match cli.command {
        Command::Apps { refresh } => print_json(&engine.apps(refresh)),
        Command::Profiles { command } => match command {
            Profiles::List => print_json(&engine.store.read()?),
            Profiles::Create { app, name, color } => {
                engine.recipe(&app)?;
                let color: ProfileColor = serde_json::from_value(Value::String(color))?;
                let root = cli
                    .profiles_root
                    .unwrap_or_else(|| state_dir.join("Profiles"));
                let profile = Profile::new(&app, &name, &root, color)?;
                print_json(&engine.store.create(profile)?)
            }
            Profiles::Rename { id, name } => {
                engine.store.rename(&id, &name)?;
                print_json(engine.store.read()?.profile(&id)?)
            }
            Profiles::Remove { id } => {
                let profile = engine.store.remove(&id)?;
                print_json(
                    &json!({ "removedFromDeck": profile.id, "dataKeptAt": profile.data_dir }),
                )
            }
        },
        Command::Launch { id } => print_json(&engine.launch(&id)?),
        Command::Plan { id } => {
            print_json(&engine.plan(engine.store.read()?.profile(&id)?, false)?)
        }
        Command::Close { id, force } => {
            match engine.close(&id) {
                Ok(()) => {}
                Err(Error::StillRunning) if force => {
                    let profile = engine.store.read()?.profile(&id)?.clone();
                    eprintln!(
                        "{} is still running after 8 seconds. Force close can lose unsaved work.",
                        profile.name
                    );
                    eprint!("Type the profile name to confirm Force close: ");
                    io::stderr()
                        .flush()
                        .map_err(|e| Error::Platform(e.to_string()))?;
                    let mut confirmation = String::new();
                    io::stdin()
                        .read_line(&mut confirmation)
                        .map_err(|e| Error::Platform(e.to_string()))?;
                    engine.force_close(&id, confirmation.trim_end() == profile.name)?;
                }
                Err(error) => return Err(error),
            }
            print_json(&json!({ "closed": id }))
        }
        Command::Doctor { redact } => doctor(&engine, &log_dir, redact),
        Command::SignIn { command } => {
            use ideck_core::signin::{JournalStore, SignIn};
            let runner = SignIn::new(&engine);
            match command {
                SignInCommand::List => {
                    print_json(&JournalStore::new(engine.store.root()).active()?)
                }
                SignInCommand::Start { id, close_main } => {
                    let journal = runner.prepare(&id, close_main)?;
                    print_json(&runner.resume(&journal.id)?)
                }
                SignInCommand::Resume { id } => print_json(&runner.resume(&id)?),
                SignInCommand::Finish { id } => print_json(&runner.finish(&id)?),
                SignInCommand::Restore { id } => print_json(&runner.restore(&id)?),
            }
        }
        Command::RecipeSchema { .. } => unreachable!(),
    }
}

fn print_json(value: &impl Serialize) -> Result<()> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn doctor(engine: &Engine<WindowsPlatform>, log_dir: &std::path::Path, redact: bool) -> Result<()> {
    let directories = engine.platform.directories()?;
    let apps = engine.apps(true);
    let main_instances = apps.iter().filter(|app| app.installed.is_some()).map(|app| {
        match engine.main_processes(&app.recipe.id) {
            Ok(processes) => json!({ "appId": app.recipe.id, "runningPids": processes.iter().map(|p| p.pid).collect::<Vec<_>>() }),
            Err(error) => json!({ "appId": app.recipe.id, "error": error.to_string() }),
        }
    }).collect::<Vec<_>>();
    let state = engine.store.read()?;
    let mut profiles = Vec::new();
    for profile in &state.profiles {
        let status = match engine.running(profile) {
            Ok(processes) => {
                json!({ "id": profile.id, "appId": profile.app_id, "runningPids": processes.iter().map(|p| p.pid).collect::<Vec<_>>() })
            }
            Err(error) => {
                json!({ "id": profile.id, "appId": profile.app_id, "error": error.to_string() })
            }
        };
        profiles.push(status);
    }
    let candidates = layout::claude_candidates(
        std::path::Path::new(&directories.local_app_data),
        std::path::Path::new(&directories.app_data),
    );
    let layout = candidates
        .iter()
        .map(|path| match layout::probe(path) {
            Ok(probe) => {
                json!({ "path": probe.path, "live": probe.live, "relocated": probe.relocated })
            }
            Err(error) => json!({ "path": path, "error": error.to_string() }),
        })
        .collect::<Vec<_>>();
    let mut logs = Vec::new();
    let mut files = fs::read_dir(log_dir)
        .map_err(|e| ideck_core::error::io(log_dir, e))?
        .filter_map(std::result::Result::ok)
        .filter(|entry| {
            if !entry
                .file_type()
                .is_ok_and(|kind| kind.is_file() && !kind.is_symlink())
            {
                return false;
            }
            entry
                .file_name()
                .to_string_lossy()
                .starts_with("launches.jsonl.")
        })
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    files.sort();
    for file in files.into_iter().rev() {
        // Only our structured launch logs are read, never target data or arbitrary files.
        if let Ok(text) = fs::read_to_string(&file) {
            for line in text.lines().rev().take(50 - logs.len()) {
                if let Ok(value) = serde_json::from_str::<Value>(line) {
                    logs.push(value);
                }
            }
        }
        if logs.len() >= 50 {
            break;
        }
    }
    let mut report = json!({ "product": "InstanceDeck", "version": env!("CARGO_PKG_VERSION"), "directories": directories, "apps": apps, "mainInstances": main_instances, "profiles": profiles, "claudeLayouts": layout, "launchLogs": logs, "accountHealth": "Check the visible account inside each profile; credentials are never inspected." });
    if redact {
        redact_value(&mut report, &directories.user_profile);
    }
    print_json(&report)
}

fn redact_value(value: &mut Value, user_profile: &str) {
    match value {
        Value::String(text) => *text = paths::redact(text, user_profile),
        Value::Array(values) => {
            for value in values {
                redact_value(value, user_profile);
            }
        }
        Value::Object(values) => {
            for value in values.values_mut() {
                redact_value(value, user_profile);
            }
        }
        _ => {}
    }
}
