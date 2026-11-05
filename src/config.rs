use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;

use crate::error::SorterError;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, SorterError> {
        let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => SorterError::ConfigMissing(path.to_path_buf()),
            _ => e.into(),
        })?;
        Self::parse(&text)
    }

    fn parse(text: &str) -> Result<Self, SorterError> {
        let config: Self = toml::from_str(text)?;
        if !Path::new(&config.url).is_dir() {
            return Err(SorterError::UrlNotDirectory(config.url));
        }
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_existing_directory() {
        assert_eq!(Config::parse("url = \"src\"").unwrap().url, "src");
    }

    #[test]
    fn rejects_bad_configs() {
        assert!(matches!(
            Config::load(Path::new("/nonexistent/config.toml")),
            Err(SorterError::ConfigMissing(_))
        ));
        assert!(matches!(
            Config::parse("other = 1"),
            Err(SorterError::ConfigInvalid(_))
        ));
        assert!(matches!(
            Config::parse("url = \"/nonexistent\""),
            Err(SorterError::UrlNotDirectory(_))
        ));
    }
}
