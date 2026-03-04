pub mod file;
pub mod secret_service;

use async_trait::async_trait;
use chrono::NaiveDate;

use crate::config::Config;
use crate::error::Error;

#[async_trait]
pub trait Storage: Send + Sync {
    async fn store_birthdate(&self, date: NaiveDate) -> Result<(), Error>;
    async fn load_birthdate(&self) -> Result<Option<NaiveDate>, Error>;
    async fn store_default_jurisdiction(&self, name: &str) -> Result<(), Error>;
    async fn load_default_jurisdiction(&self) -> Result<Option<String>, Error>;
}

/// Create a storage backend based on configuration.
/// Tries Secret Service first if configured, falls back to file storage on failure.
pub async fn create_storage(config: &Config) -> Box<dyn Storage> {
    if config.storage.backend == "secret-service" {
        match secret_service::SecretServiceStorage::new().await {
            Ok(ss) => {
                tracing::info!("using Secret Service storage backend");
                return Box::new(ss);
            }
            Err(e) => {
                tracing::warn!("Secret Service unavailable ({e}), falling back to file storage");
            }
        }
    }

    tracing::info!("using file storage backend");
    Box::new(file::FileStorage::new())
}
