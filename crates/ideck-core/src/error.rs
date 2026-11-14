use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Invalid configuration: {0}")]
    Invalid(String),
    #[error("{0} was not found")]
    NotFound(String),
    #[error("Cannot access {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("Invalid JSON: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Windows operation failed: {0}")]
    Platform(String),
    #[error("The app is still running. Confirm Force close only after a graceful close attempt.")]
    StillRunning,
    #[error("Launch failed after trying {strategies}: {detail}")]
    Launch { strategies: String, detail: String },
}

pub type Result<T> = std::result::Result<T, Error>;

pub fn io(path: impl Into<PathBuf>, source: std::io::Error) -> Error {
    Error::Io {
        path: path.into(),
        source,
    }
}
