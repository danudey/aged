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
        let contents = toml::to_string_pretty(data).map_err(|e| Error::Storage(e.to_string()))?;
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
                let date = s.parse::<NaiveDate>().map_err(|_| Error::InvalidDate(s))?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::storage::Storage;

    fn test_storage(dir: &std::path::Path) -> FileStorage {
        FileStorage {
            path: dir.join("data.toml"),
        }
    }

    #[tokio::test]
    async fn load_birthdate_empty() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        assert_eq!(s.load_birthdate().await.unwrap(), None);
    }

    #[tokio::test]
    async fn store_and_load_birthdate() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        let date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
        s.store_birthdate(date).await.unwrap();
        assert_eq!(s.load_birthdate().await.unwrap(), Some(date));
    }

    #[tokio::test]
    async fn store_birthdate_overwrites() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        let d1 = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        let d2 = NaiveDate::from_ymd_opt(2000, 6, 15).unwrap();
        s.store_birthdate(d1).await.unwrap();
        s.store_birthdate(d2).await.unwrap();
        assert_eq!(s.load_birthdate().await.unwrap(), Some(d2));
    }

    #[tokio::test]
    async fn load_default_jurisdiction_empty() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        assert_eq!(s.load_default_jurisdiction().await.unwrap(), None);
    }

    #[tokio::test]
    async fn store_and_load_default_jurisdiction() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        s.store_default_jurisdiction("US/California").await.unwrap();
        assert_eq!(
            s.load_default_jurisdiction().await.unwrap(),
            Some("US/California".to_string())
        );
    }

    #[tokio::test]
    async fn store_both_fields() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        let date = NaiveDate::from_ymd_opt(1985, 12, 25).unwrap();
        s.store_birthdate(date).await.unwrap();
        s.store_default_jurisdiction("US/Colorado").await.unwrap();

        assert_eq!(s.load_birthdate().await.unwrap(), Some(date));
        assert_eq!(
            s.load_default_jurisdiction().await.unwrap(),
            Some("US/Colorado".to_string())
        );
    }

    #[tokio::test]
    async fn written_file_is_valid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let s = test_storage(dir.path());
        let date = NaiveDate::from_ymd_opt(1990, 5, 15).unwrap();
        s.store_birthdate(date).await.unwrap();

        let contents = std::fs::read_to_string(dir.path().join("data.toml")).unwrap();
        let data: FileData = toml::from_str(&contents).unwrap();
        assert_eq!(data.birthdate, Some("1990-05-15".to_string()));
    }

    #[tokio::test]
    async fn load_from_manually_written_file() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("data.toml");
        std::fs::write(
            &path,
            "birthdate = \"2001-09-11\"\ndefault_jurisdiction = \"US/California\"\n",
        )
        .unwrap();

        let s = test_storage(dir.path());
        assert_eq!(
            s.load_birthdate().await.unwrap(),
            Some(NaiveDate::from_ymd_opt(2001, 9, 11).unwrap())
        );
        assert_eq!(
            s.load_default_jurisdiction().await.unwrap(),
            Some("US/California".to_string())
        );
    }

    #[tokio::test]
    async fn creates_parent_directories() {
        let dir = tempfile::tempdir().unwrap();
        let s = FileStorage {
            path: dir.path().join("sub").join("dir").join("data.toml"),
        };
        let date = NaiveDate::from_ymd_opt(1990, 1, 1).unwrap();
        s.store_birthdate(date).await.unwrap();
        assert_eq!(s.load_birthdate().await.unwrap(), Some(date));
    }
}
