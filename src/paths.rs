use std::path::PathBuf;

pub fn config_path() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory not found")
        .join(".config")
        .join("sorter")
        .join("config.toml")
}

pub fn app_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory not found")
        .join(".local")
        .join("share")
        .join("sorter")
}
