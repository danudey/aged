use std::path::PathBuf;

use async_trait::async_trait;
use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::storage::Storage;

#[derive(Debug, Serialize, Deserialize, Default)]
struct FileData {
    birthdate: Option<String>,
    default_jurisdiction: Option<String>,
}

pub struct FileStorage {
    path: PathBuf,
}

impl FileStorage {
    pub fn new() -> Self {
        let dir = directories::ProjectDirs::from("org", "aged", "aged")
            .map(|d| d.config_dir().to_path_buf())
            .unwrap_or_else(|| {
                directories::BaseDirs::new()
                    .map(|d| d.config_dir().join("aged"))
                    .unwrap_or_else(|| PathBuf::from(".config/aged"))
            });
        Self {
            path: dir.join("data.toml"),
        }
    }

    fn read_data(&self) -> Result<FileData, Error> {
        if self.path.exists() {
            let contents = std::fs::read_to_string(&self.path)?;
            Ok(toml::from_str(&contents)?)
        } else {
            Ok(FileData::default())
        }
    }

    fn write_data(&self, data: &FileData) -> Result<(), Error> {
        if let Some(parent) = self.path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let contents =
            toml::to_string_pretty(data).map_err(|e| Error::Storage(e.to_string()))?;
        std::fs::write(&self.path, contents)?;
        Ok(())
    }
}

#[async_trait]
impl Storage for FileStorage {
    async fn store_birthdate(&self, date: NaiveDate) -> Result<(), Error> {
        let mut data = self.read_data()?;
        data.birthdate = Some(date.to_string());
        self.write_data(&data)
    }

    async fn load_birthdate(&self) -> Result<Option<NaiveDate>, Error> {
        let data = self.read_data()?;
        match data.birthdate {
            Some(s) => {
                let date = s
                    .parse::<NaiveDate>()
                    .map_err(|_| Error::InvalidDate(s))?;
                Ok(Some(date))
            }
            None => Ok(None),
        }
    }

    async fn store_default_jurisdiction(&self, name: &str) -> Result<(), Error> {
        let mut data = self.read_data()?;
        data.default_jurisdiction = Some(name.to_string());
        self.write_data(&data)
    }

    async fn load_default_jurisdiction(&self) -> Result<Option<String>, Error> {
        let data = self.read_data()?;
        Ok(data.default_jurisdiction)
    }
}
