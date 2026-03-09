use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Default)]
pub struct Config {
    #[serde(default)]
    pub storage: StorageConfig,

    #[serde(default)]
    pub jurisdictions: JurisdictionsConfig,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    #[serde(default = "default_backend")]
    pub backend: String,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            backend: default_backend(),
        }
    }
}

fn default_backend() -> String {
    "secret-service".to_string()
}

#[derive(Debug, Deserialize, Default)]
pub struct JurisdictionsConfig {
    #[serde(default)]
    pub extra_paths: Vec<PathBuf>,
}

impl Config {
    pub fn load(path: &Path) -> Result<Self, Error> {
        if path.exists() {
            let contents = std::fs::read_to_string(path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            tracing::debug!(?path, "config file not found, using defaults");
            Ok(Self::default())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn default_config() {
        let config = Config::default();
        assert_eq!(config.storage.backend, "secret-service");
        assert!(config.jurisdictions.extra_paths.is_empty());
    }

    #[test]
    fn load_missing_file_returns_defaults() {
        let config = Config::load(Path::new("/tmp/aged-test-nonexistent.toml")).unwrap();
        assert_eq!(config.storage.backend, "secret-service");
    }

    #[test]
    fn load_valid_config() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        let mut f = std::fs::File::create(&path).unwrap();
        writeln!(
            f,
            r#"
[storage]
backend = "file"

[jurisdictions]
extra_paths = ["/tmp/extra.toml"]
"#
        )
        .unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.storage.backend, "file");
        assert_eq!(
            config.jurisdictions.extra_paths,
            vec![PathBuf::from("/tmp/extra.toml")]
        );
    }

    #[test]
    fn load_partial_config_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "[storage]\n").unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.storage.backend, "secret-service");
        assert!(config.jurisdictions.extra_paths.is_empty());
    }

    #[test]
    fn load_empty_config_uses_defaults() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "").unwrap();

        let config = Config::load(&path).unwrap();
        assert_eq!(config.storage.backend, "secret-service");
    }

    #[test]
    fn load_invalid_toml_returns_error() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("config.toml");
        std::fs::write(&path, "not valid {{{{ toml").unwrap();

        assert!(Config::load(&path).is_err());
    }
}
