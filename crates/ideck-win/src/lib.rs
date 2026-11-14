//! Windows discovery, process and launch adapters; no target credential file reads.
pub mod command;
pub mod discovery;
pub mod icons;
#[cfg(windows)]
mod platform;
#[cfg(windows)]
pub mod shell;
#[cfg(windows)]
mod window_identity;
#[cfg(windows)]
pub use platform::WindowsPlatform;
#[cfg(windows)]
mod desktop;
#[cfg(windows)]
mod history;
#[cfg(windows)]
mod signin;
