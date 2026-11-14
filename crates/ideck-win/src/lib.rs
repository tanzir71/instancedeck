//! Windows discovery, process and launch adapters; no target credential file reads.
pub mod command;
pub mod discovery;
#[cfg(windows)]
mod platform;
#[cfg(windows)]
pub use platform::WindowsPlatform;
#[cfg(windows)]
mod desktop;
