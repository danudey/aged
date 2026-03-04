use std::sync::Arc;

use chrono::NaiveDate;
use zbus::interface;

use crate::age;
use crate::error::Error;
use crate::jurisdiction::JurisdictionRegistry;
use crate::storage::Storage;

pub struct AgedDbus {
    pub storage: Arc<dyn Storage>,
    pub jurisdictions: Arc<JurisdictionRegistry>,
}

#[interface(name = "org.aged.Daemon")]
impl AgedDbus {
    async fn set_birthdate(&self, date_str: &str) -> Result<(), zbus::fdo::Error> {
        let date = date_str
            .parse::<NaiveDate>()
            .map_err(|_| Error::InvalidDate(date_str.to_string()))?;
        self.storage.store_birthdate(date).await?;
        tracing::info!("birthdate stored");
        Ok(())
    }

    async fn get_age_bracket(
        &self,
        jurisdiction: &str,
    ) -> Result<String, zbus::fdo::Error> {
        let birthdate = self
            .storage
            .load_birthdate()
            .await?
            .ok_or(Error::NoBirthdate)?;

        let jurisdiction_name = if jurisdiction.is_empty() {
            self.storage
                .load_default_jurisdiction()
                .await?
                .ok_or(Error::NoDefaultJurisdiction)?
        } else {
            jurisdiction.to_string()
        };

        let age = age::calculate_age(birthdate);
        let bracket = self.jurisdictions.lookup_bracket(&jurisdiction_name, age)?;
        Ok(bracket)
    }

    async fn list_jurisdictions(&self) -> Vec<String> {
        self.jurisdictions.list_names()
    }

    async fn get_default_jurisdiction(&self) -> Result<String, zbus::fdo::Error> {
        let name = self
            .storage
            .load_default_jurisdiction()
            .await?
            .unwrap_or_default();
        Ok(name)
    }

    async fn set_default_jurisdiction(
        &self,
        name: &str,
    ) -> Result<(), zbus::fdo::Error> {
        // Validate the jurisdiction exists
        if self.jurisdictions.get(name).is_none() {
            return Err(Error::UnknownJurisdiction(name.to_string()).into());
        }
        self.storage.store_default_jurisdiction(name).await?;
        tracing::info!(jurisdiction = name, "default jurisdiction set");
        Ok(())
    }

    async fn detect_jurisdiction(&self) -> Result<String, zbus::fdo::Error> {
        #[cfg(feature = "geoclue")]
        {
            let name = crate::geoclue::detect_jurisdiction(&self.jurisdictions).await?;
            Ok(name)
        }
        #[cfg(not(feature = "geoclue"))]
        {
            Err(zbus::fdo::Error::NotSupported(
                "geoclue feature not enabled".to_string(),
            ))
        }
    }
}
