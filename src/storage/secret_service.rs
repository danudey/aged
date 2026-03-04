use std::collections::HashMap;

use async_trait::async_trait;
use chrono::NaiveDate;
use secret_service::EncryptionType;
use secret_service::SecretService;

use crate::error::Error;
use crate::storage::Storage;

const BIRTHDATE_LABEL: &str = "aged-birthdate";
const JURISDICTION_LABEL: &str = "aged-default-jurisdiction";
const SERVICE_ATTR: &str = "aged";

pub struct SecretServiceStorage {
    service: SecretService<'static>,
}

fn make_attributes(label: &str) -> HashMap<&str, &str> {
    HashMap::from([("service", SERVICE_ATTR), ("type", label)])
}

impl SecretServiceStorage {
    pub async fn new() -> Result<Self, Error> {
        let service = SecretService::connect(EncryptionType::Dh)
            .await
            .map_err(|e| Error::Storage(format!("Secret Service connection failed: {e}")))?;
        Ok(Self { service })
    }

    async fn store_secret(&self, label: &str, value: &str) -> Result<(), Error> {
        let collection = self
            .service
            .get_default_collection()
            .await
            .map_err(|e| Error::Storage(format!("get default collection: {e}")))?;

        collection
            .unlock()
            .await
            .map_err(|e| Error::Storage(format!("unlock collection: {e}")))?;

        let attributes = make_attributes(label);

        // Delete existing item with these attributes
        if let Ok(items) = collection.search_items(attributes.clone()).await {
            for item in items {
                let _ = item.delete().await;
            }
        }

        collection
            .create_item(label, attributes, value.as_bytes(), true, "text/plain")
            .await
            .map_err(|e| Error::Storage(format!("create item: {e}")))?;

        Ok(())
    }

    async fn load_secret(&self, label: &str) -> Result<Option<String>, Error> {
        let collection = self
            .service
            .get_default_collection()
            .await
            .map_err(|e| Error::Storage(format!("get default collection: {e}")))?;

        collection
            .unlock()
            .await
            .map_err(|e| Error::Storage(format!("unlock collection: {e}")))?;

        let attributes = make_attributes(label);
        let items = collection
            .search_items(attributes)
            .await
            .map_err(|e| Error::Storage(format!("search items: {e}")))?;

        match items.first() {
            Some(item) => {
                let secret = item
                    .get_secret()
                    .await
                    .map_err(|e| Error::Storage(format!("get secret: {e}")))?;
                let value = String::from_utf8(secret)
                    .map_err(|e| Error::Storage(format!("invalid UTF-8 in secret: {e}")))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

#[async_trait]
impl Storage for SecretServiceStorage {
    async fn store_birthdate(&self, date: NaiveDate) -> Result<(), Error> {
        self.store_secret(BIRTHDATE_LABEL, &date.to_string()).await
    }

    async fn load_birthdate(&self) -> Result<Option<NaiveDate>, Error> {
        match self.load_secret(BIRTHDATE_LABEL).await? {
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
        self.store_secret(JURISDICTION_LABEL, name).await
    }

    async fn load_default_jurisdiction(&self) -> Result<Option<String>, Error> {
        self.load_secret(JURISDICTION_LABEL).await
    }
}
