use crate::{Error, Result, template};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Recipe {
    pub id: String,
    pub recipe_version: u32,
    pub name: String,
    pub vendor: String,
    pub category: String,
    pub discovery: Vec<Discovery>,
    pub launch: Launch,
    pub isolation: Isolation,
    pub sign_in: SignIn,
    pub process_match: ProcessMatch,
    pub limitations: Vec<String>,
    pub verified_on: Vec<Verification>,
    pub status: RecipeStatus,
    pub seo: Seo,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(
    tag = "type",
    rename_all = "kebab-case",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Discovery {
    Msix {
        package_family: String,
        exe: String,
        #[serde(default = "default_app_id")]
        app_id: String,
    },
    Registry {
        display_name: String,
        exe: String,
    },
    KnownPath {
        path: String,
        #[serde(default)]
        version_prefix: Option<String>,
    },
}
fn default_app_id() -> String {
    "App".into()
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Launch {
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
    pub strategies: Vec<Strategy>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Strategy {
    Direct,
    PackageContext,
}
impl std::fmt::Display for Strategy {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Direct => "direct",
            Self::PackageContext => "package-context",
        })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum Isolation {
    Full,
    Partial,
    Unknown,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum RecipeStatus {
    Verified,
    Community,
    Experimental,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct SignIn {
    pub strategy: SignInStrategy,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SignInStrategy {
    InProfile,
    Snapshot,
    Manual,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ProcessMatch {
    pub exe_name: String,
    pub arg: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Verification {
    pub app_version: String,
    pub os: String,
    pub date: String,
    pub by: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Seo {
    pub slug: String,
    pub title: String,
    pub description: String,
    pub intro: String,
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
        && !id.starts_with('-')
        && !id.ends_with('-')
}

impl Recipe {
    pub fn from_json(input: &str) -> Result<Self> {
        let recipe: Self = serde_json::from_str(input)?;
        recipe.validate()?;
        Ok(recipe)
    }
    pub fn validate(&self) -> Result<()> {
        if !valid_id(&self.id)
            || !valid_id(&self.seo.slug)
            || self.recipe_version == 0
            || self.name.trim().is_empty()
            || self.discovery.is_empty()
        {
            return Err(Error::Invalid(
                "Recipe requires a safe id, version, name, slug and discovery method".into(),
            ));
        }
        if self.launch.strategies.first() != Some(&Strategy::Direct)
            || self.launch.strategies.len() > 2
            || (self.launch.strategies.len() == 2
                && self.launch.strategies[1] != Strategy::PackageContext)
        {
            return Err(Error::Invalid(
                "Strategies must be direct, optionally followed once by package-context".into(),
            ));
        }
        if self.process_match.exe_name.contains(['\\', '/'])
            || !self
                .process_match
                .exe_name
                .to_ascii_lowercase()
                .ends_with(".exe")
        {
            return Err(Error::Invalid(
                "Process match requires an executable file name".into(),
            ));
        }
        let paired = self
            .process_match
            .arg
            .split_once('=')
            .is_some_and(|(flag, value)| {
                self.launch
                    .args
                    .windows(2)
                    .any(|pair| pair[0] == flag && pair[1] == value)
            });
        let environment_only = self.process_match.arg.is_empty()
            && self.id.starts_with("custom-")
            && self
                .launch
                .env
                .values()
                .any(|value| value.starts_with("{profile}"));
        if !environment_only
            && (!self.process_match.arg.contains("{profile}")
                || (!self.launch.args.contains(&self.process_match.arg) && !paired))
        {
            return Err(Error::Invalid(
                "The exact profile match argument must be part of the launch arguments".into(),
            ));
        }
        for arg in &self.launch.args {
            template::validate(arg)?;
        }
        let mut keys = BTreeSet::new();
        for (key, value) in &self.launch.env {
            if key.is_empty()
                || !key.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                || key.as_bytes()[0].is_ascii_digit()
                || !keys.insert(key.to_ascii_uppercase())
            {
                return Err(Error::Invalid(
                    "Environment names must be unique Windows variable names".into(),
                ));
            }
            template::validate(value.strip_suffix('?').unwrap_or(value))?;
        }
        for discovery in &self.discovery {
            match discovery {
                Discovery::Msix {
                    package_family,
                    exe,
                    app_id,
                } => {
                    if package_family.is_empty()
                        || app_id.is_empty()
                        || exe.starts_with(['/', '\\'])
                        || exe.split(['/', '\\']).any(|s| s == "..")
                        || exe.contains(':')
                    {
                        return Err(Error::Invalid(
                            "MSIX discovery requires a family and a package-relative executable"
                                .into(),
                        ));
                    }
                }
                Discovery::Registry { display_name, exe } => {
                    if display_name.trim().is_empty()
                        || exe.starts_with(['/', '\\'])
                        || exe.contains(':')
                        || exe.split(['/', '\\']).any(|s| s == "..")
                    {
                        return Err(Error::Invalid(
                            "Registry discovery requires a display name and relative executable"
                                .into(),
                        ));
                    }
                }
                Discovery::KnownPath {
                    path,
                    version_prefix,
                } => {
                    template::validate(path)?;
                    if version_prefix.as_ref().is_some_and(|prefix| {
                        prefix.is_empty()
                            || !prefix.bytes().all(|b| {
                                b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.')
                            })
                    }) {
                        return Err(Error::Invalid(
                            "A version-folder prefix must be a file-name fragment".into(),
                        ));
                    }
                }
            }
        }
        Ok(())
    }
}

pub fn built_in_recipes() -> Result<Vec<Recipe>> {
    [
        include_str!("../../../packages/recipes/claude-desktop.recipe.json"),
        include_str!("../../../packages/recipes/codex.recipe.json"),
        include_str!("../../../packages/recipes/vscode.recipe.json"),
        include_str!("../../../packages/recipes/cursor.recipe.json"),
        include_str!("../../../packages/recipes/chrome.recipe.json"),
        include_str!("../../../packages/recipes/edge.recipe.json"),
        include_str!("../../../packages/recipes/brave.recipe.json"),
        include_str!("../../../packages/recipes/telegram-desktop.recipe.json"),
    ]
    .into_iter()
    .map(Recipe::from_json)
    .collect()
}

pub fn recipe_schema() -> Result<String> {
    Ok(serde_json::to_string_pretty(&schemars::schema_for!(Recipe))? + "\n")
}
