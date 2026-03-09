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
        let builtin: JurisdictionFile = toml::from_str(include_str!("../dist/jurisdictions.toml"))
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

#[cfg(test)]
mod tests {
    use super::*;

    fn registry_with_builtins() -> JurisdictionRegistry {
        let mut r = JurisdictionRegistry::new();
        r.load_builtin();
        r
    }

    #[test]
    fn new_is_empty() {
        let r = JurisdictionRegistry::new();
        assert!(r.list_names().is_empty());
        assert!(r.get("US/California").is_none());
    }

    #[test]
    fn load_builtin_populates_jurisdictions() {
        let r = registry_with_builtins();
        let names = r.list_names();
        assert!(names.contains(&"US/California".to_string()));
        assert!(names.contains(&"US/Colorado".to_string()));
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn list_names_is_sorted() {
        let r = registry_with_builtins();
        let names = r.list_names();
        assert_eq!(names, vec!["US/California", "US/Colorado"]);
    }

    #[test]
    fn get_existing() {
        let r = registry_with_builtins();
        let j = r.get("US/California").unwrap();
        assert_eq!(j.name, "US/California");
        assert_eq!(j.region, "US-CA");
        assert_eq!(j.brackets.len(), 3);
    }

    #[test]
    fn get_nonexistent() {
        let r = registry_with_builtins();
        assert!(r.get("US/Texas").is_none());
    }

    // California brackets: child (<13), minor (<18), adult (catch-all)
    #[test]
    fn california_child() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/California", 0).unwrap(), "child");
        assert_eq!(r.lookup_bracket("US/California", 5).unwrap(), "child");
        assert_eq!(r.lookup_bracket("US/California", 12).unwrap(), "child");
    }

    #[test]
    fn california_minor() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/California", 13).unwrap(), "minor");
        assert_eq!(r.lookup_bracket("US/California", 15).unwrap(), "minor");
        assert_eq!(r.lookup_bracket("US/California", 17).unwrap(), "minor");
    }

    #[test]
    fn california_adult() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/California", 18).unwrap(), "adult");
        assert_eq!(r.lookup_bracket("US/California", 35).unwrap(), "adult");
        assert_eq!(r.lookup_bracket("US/California", 99).unwrap(), "adult");
    }

    // Colorado brackets: minor (<18), adult (<65), senior (catch-all)
    #[test]
    fn colorado_minor() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/Colorado", 0).unwrap(), "minor");
        assert_eq!(r.lookup_bracket("US/Colorado", 17).unwrap(), "minor");
    }

    #[test]
    fn colorado_adult() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/Colorado", 18).unwrap(), "adult");
        assert_eq!(r.lookup_bracket("US/Colorado", 64).unwrap(), "adult");
    }

    #[test]
    fn colorado_senior() {
        let r = registry_with_builtins();
        assert_eq!(r.lookup_bracket("US/Colorado", 65).unwrap(), "senior");
        assert_eq!(r.lookup_bracket("US/Colorado", 90).unwrap(), "senior");
    }

    #[test]
    fn lookup_unknown_jurisdiction() {
        let r = registry_with_builtins();
        let err = r.lookup_bracket("US/Texas", 25).unwrap_err();
        assert!(err.to_string().contains("US/Texas"));
    }

    #[test]
    fn lookup_empty_brackets() {
        let mut r = JurisdictionRegistry::new();
        r.jurisdictions.insert(
            "Empty".to_string(),
            Jurisdiction {
                name: "Empty".to_string(),
                region: "XX".to_string(),
                brackets: vec![],
            },
        );
        let err = r.lookup_bracket("Empty", 25).unwrap_err();
        assert!(err.to_string().contains("Empty"));
    }

    #[test]
    fn load_file_works() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test.toml");
        std::fs::write(
            &path,
            r#"
[[jurisdiction]]
name = "Test/Place"
region = "TP"
brackets = [
    { max_age = 21, label = "youth" },
    { label = "grown" },
]
"#,
        )
        .unwrap();

        let mut r = JurisdictionRegistry::new();
        r.load_file(&path).unwrap();
        assert_eq!(r.lookup_bracket("Test/Place", 20).unwrap(), "youth");
        assert_eq!(r.lookup_bracket("Test/Place", 21).unwrap(), "grown");
    }

    #[test]
    fn load_file_overrides_existing() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("override.toml");
        std::fs::write(
            &path,
            r#"
[[jurisdiction]]
name = "US/California"
region = "US-CA"
brackets = [
    { max_age = 21, label = "young" },
    { label = "old" },
]
"#,
        )
        .unwrap();

        let mut r = registry_with_builtins();
        r.load_file(&path).unwrap();
        // Should use the overridden brackets
        assert_eq!(r.lookup_bracket("US/California", 18).unwrap(), "young");
    }

    #[test]
    fn load_file_invalid_path() {
        let mut r = JurisdictionRegistry::new();
        assert!(r.load_file(Path::new("/nonexistent/file.toml")).is_err());
    }

    #[test]
    fn load_file_invalid_toml() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("bad.toml");
        std::fs::write(&path, "not valid toml {{{{").unwrap();

        let mut r = JurisdictionRegistry::new();
        assert!(r.load_file(&path).is_err());
    }

    #[test]
    fn find_by_timezone_california() {
        let r = registry_with_builtins();
        assert_eq!(
            r.find_by_timezone("America/Los_Angeles"),
            Some("US/California".to_string())
        );
    }

    #[test]
    fn find_by_timezone_colorado() {
        let r = registry_with_builtins();
        assert_eq!(
            r.find_by_timezone("America/Denver"),
            Some("US/Colorado".to_string())
        );
    }

    #[test]
    fn find_by_timezone_unknown() {
        let r = registry_with_builtins();
        assert_eq!(r.find_by_timezone("Europe/London"), None);
    }

    #[test]
    fn find_by_timezone_empty_registry() {
        let r = JurisdictionRegistry::new();
        assert_eq!(r.find_by_timezone("America/Los_Angeles"), None);
    }

    #[test]
    fn lookup_bracket_no_catchall_falls_through() {
        let mut r = JurisdictionRegistry::new();
        r.load_file(Path::new("tests/data/no_catchall.toml")).unwrap();

        // Age within defined brackets
        assert_eq!(r.lookup_bracket("Test/NoCatchAll", 10).unwrap(), "child");
        assert_eq!(r.lookup_bracket("Test/NoCatchAll", 15).unwrap(), "minor");
        assert_eq!(r.lookup_bracket("Test/NoCatchAll", 30).unwrap(), "adult");

        // Age exceeds ALL max_age values — triggers the fall-through on line 85
        assert_eq!(r.lookup_bracket("Test/NoCatchAll", 65).unwrap(), "adult");
        assert_eq!(r.lookup_bracket("Test/NoCatchAll", 99).unwrap(), "adult");
    }
}
