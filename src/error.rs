use std::path::PathBuf;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum SorterError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("config file not found: {}", .0.display())]
    ConfigMissing(PathBuf),

    #[error("invalid config: {0}")]
    ConfigInvalid(#[from] toml::de::Error),

    #[error("url in config is not an existing directory: {0}")]
    UrlNotDirectory(String),

    #[error("daemon already running (pid {0})")]
    AlreadyRunning(u32),

    #[error("daemon is not running")]
    NotRunning,

    #[error("could not stop daemon (pid {0})")]
    StopFailed(u32),
}
