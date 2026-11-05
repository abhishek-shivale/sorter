use std::path::PathBuf;

const APP_DIR: &str = "";

pub fn app_dir() -> PathBuf {
    dirs::home_dir()
        .expect("Home directory not found")
        .join(".local")
        .join("share")
        .join("sorter")
}
