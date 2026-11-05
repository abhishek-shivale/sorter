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

    #[error("invalid config: {0}")]
    ConfigEdit(#[from] toml_edit::TomlError),

    #[error("url in config is not an existing directory: {0}")]
    UrlNotDirectory(String),

    #[error("refusing to sort {0}: it is the home directory or a parent of it")]
    UnsafeDir(String),

    #[error("daemon already running")]
    AlreadyRunning,

    #[error("daemon is not running")]
    NotRunning,

    #[error("could not start daemon: launchctl bootstrap failed")]
    StartFailed,

    #[error("could not stop daemon: launchctl bootout failed")]
    StopFailed,
}
