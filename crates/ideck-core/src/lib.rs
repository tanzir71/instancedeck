//! Platform-independent profile and recipe engine. Target credentials are never read.

pub mod engine;
pub mod error;
pub mod layout;
pub mod paths;
pub mod platform;
pub mod process;
pub mod profile;
pub mod recipe;
pub mod settings;
pub mod shell;
pub mod signin;
pub mod template;
pub use settings::{Settings, SettingsStore};

pub use engine::Engine;
pub use error::{Error, Result};
pub use platform::*;
pub use profile::*;
pub use recipe::*;

pub const PRODUCT_NAME: &str = "InstanceDeck";
