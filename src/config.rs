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
