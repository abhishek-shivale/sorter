use std::fs;
use std::io::ErrorKind;
use std::path::Path;

use serde::Deserialize;
use toml_edit::{DocumentMut, value};

use crate::error::SorterError;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub url: String,
    #[serde(default)]
    pub ignore: Vec<String>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, SorterError> {
        let text = std::fs::read_to_string(path).map_err(|e| match e.kind() {
            ErrorKind::NotFound => SorterError::ConfigMissing(path.to_path_buf()),
            _ => e.into(),
        })?;
        Self::parse(&text)
    }

    pub fn set_dir(path: &Path, dir: &str) -> Result<(), SorterError> {
        let not_directory = || SorterError::UrlNotDirectory(dir.to_string());

        // Absolute, because the daemon does not run from the caller's directory.
        let absolute = fs::canonicalize(dir).map_err(|_| not_directory())?;
        if !absolute.is_dir() {
            return Err(not_directory());
        }
        let url = absolute.to_str().ok_or_else(not_directory)?;
        if is_home_or_parent(&absolute) {
            return Err(SorterError::UnsafeDir(url.to_string()));
        }

        let text = match fs::read_to_string(path) {
            Ok(text) => text,
            Err(e) if e.kind() == ErrorKind::NotFound => String::new(),
            Err(e) => return Err(e.into()),
        };

        let mut document: DocumentMut = text.parse()?;
        document["url"] = value(url);

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, document.to_string())?;
        println!("saved url = {url} in {}", path.display());
        Ok(())
    }

    fn parse(text: &str) -> Result<Self, SorterError> {
        let config: Self = toml::from_str(text)?;
        if !Path::new(&config.url).is_dir() {
            return Err(SorterError::UrlNotDirectory(config.url));
        }
        if is_home_or_parent(Path::new(&config.url)) {
            return Err(SorterError::UnsafeDir(config.url));
        }
        Ok(config)
    }
}

// Canonical paths, so "/Users/me/", "/Users/me/x/.." and symlinks cannot slip past.
fn is_home_or_parent(dir: &Path) -> bool {
    let home = dirs::home_dir().expect("Home directory not found");
    let home = fs::canonicalize(&home).unwrap_or(home);
    fs::canonicalize(dir).map_or(true, |dir| home.starts_with(dir))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_existing_directory() {
        assert_eq!(Config::parse("url = \"src\"").unwrap().url, "src");
    }

    #[test]
    fn reads_optional_ignore_list() {
        assert!(Config::parse("url = \"src\"").unwrap().ignore.is_empty());
        let config = Config::parse("url = \"src\"\nignore = [\"pdf\", \"zip\"]").unwrap();
        assert_eq!(config.ignore, ["pdf", "zip"]);
    }

    #[test]
    fn set_dir_keeps_existing_config_and_rejects_bad_input() {
        let root = std::env::temp_dir().join(format!("sorter-config-test-{}", std::process::id()));
        let path = root.join("nested/config.toml");

        Config::set_dir(&path, "src").unwrap();
        let config = Config::load(&path).unwrap();
        assert!(Path::new(&config.url).is_absolute());
        assert!(config.url.ends_with("src"));

        let original = "# my comment\nurl = \"old\"\nignore = [\"pdf\"]\n";
        fs::write(&path, original).unwrap();
        Config::set_dir(&path, "src").unwrap();
        let text = fs::read_to_string(&path).unwrap();
        assert!(text.contains("# my comment"));
        assert!(!text.contains("\"old\""));
        assert_eq!(Config::load(&path).unwrap().ignore, ["pdf"]);

        let before = fs::read_to_string(&path).unwrap();
        assert!(matches!(
            Config::set_dir(&path, "/nonexistent"),
            Err(SorterError::UrlNotDirectory(_))
        ));
        assert!(matches!(
            Config::set_dir(&path, "Cargo.toml"),
            Err(SorterError::UrlNotDirectory(_))
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), before);

        fs::write(&path, "not valid toml =").unwrap();
        assert!(matches!(
            Config::set_dir(&path, "src"),
            Err(SorterError::ConfigEdit(_))
        ));
        assert_eq!(fs::read_to_string(&path).unwrap(), "not valid toml =");

        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn rejects_home_and_its_parents() {
        let home = dirs::home_dir().unwrap();
        let path = std::env::temp_dir().join(format!("sorter-unsafe-test-{}", std::process::id()));
        let trailing = format!("{}/", home.display());
        let dotted = format!("{}/.", home.display());
        let parent = home.parent().unwrap().to_str().unwrap();

        for dir in ["/", home.to_str().unwrap(), &trailing, &dotted, parent] {
            assert!(matches!(
                Config::set_dir(&path, dir),
                Err(SorterError::UnsafeDir(_))
            ));
            assert!(matches!(
                Config::parse(&format!("url = \"{dir}\"")),
                Err(SorterError::UnsafeDir(_))
            ));
        }
        assert!(!path.exists());
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
