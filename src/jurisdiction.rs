use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;

use crate::error::Error;

#[derive(Debug, Deserialize, Clone)]
pub struct Bracket {
    pub max_age: Option<u32>,
    pub label: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Jurisdiction {
    pub name: String,
    #[allow(dead_code)]
    pub region: String,
    pub brackets: Vec<Bracket>,
}

#[derive(Debug, Deserialize)]
struct JurisdictionFile {
    jurisdiction: Vec<Jurisdiction>,
}

#[derive(Debug, Clone)]
pub struct JurisdictionRegistry {
    jurisdictions: HashMap<String, Jurisdiction>,
}

impl JurisdictionRegistry {
    pub fn new() -> Self {
        Self {
            jurisdictions: HashMap::new(),
        }
    }

    pub fn load_file(&mut self, path: &Path) -> Result<(), Error> {
        let contents = std::fs::read_to_string(path)?;
        let file: JurisdictionFile = toml::from_str(&contents)?;
        for j in file.jurisdiction {
            self.jurisdictions.insert(j.name.clone(), j);
        }
        Ok(())
    }

    pub fn load_builtin(&mut self) {
        let builtin: JurisdictionFile =
            toml::from_str(include_str!("../dist/jurisdictions.toml"))
                .expect("built-in jurisdictions.toml is valid");
        for j in builtin.jurisdiction {
            self.jurisdictions.insert(j.name.clone(), j);
        }
    }

    pub fn get(&self, name: &str) -> Option<&Jurisdiction> {
        self.jurisdictions.get(name)
    }

    pub fn list_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.jurisdictions.keys().cloned().collect();
        names.sort();
        names
    }

    pub fn lookup_bracket(&self, name: &str, age: u32) -> Result<String, Error> {
        let jurisdiction = self
            .get(name)
            .ok_or_else(|| Error::UnknownJurisdiction(name.to_string()))?;

        if jurisdiction.brackets.is_empty() {
            return Err(Error::NoBrackets(name.to_string()));
        }

        for bracket in &jurisdiction.brackets {
            match bracket.max_age {
                Some(max) if age < max => return Ok(bracket.label.clone()),
                None => return Ok(bracket.label.clone()),
                _ => continue,
            }
        }

        // Last bracket should be catch-all, but if somehow we get here,
        // return the last bracket's label
        Ok(jurisdiction.brackets.last().unwrap().label.clone())
    }

    /// Find a jurisdiction name by IANA timezone string.
    /// Matches against the `region` field using a simple timezone→region mapping.
    #[allow(dead_code)]
    pub fn find_by_timezone(&self, tz: &str) -> Option<String> {
        // Simple mapping from IANA timezone to region prefix
        let region = match tz {
            "America/Los_Angeles" => "US-CA",
            "America/Denver" => "US-CO",
            _ => return None,
        };

        self.jurisdictions
            .values()
            .find(|j| j.region == region)
            .map(|j| j.name.clone())
    }
}
