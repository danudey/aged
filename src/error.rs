use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("no birthdate stored")]
    NoBirthdate,

    #[error("invalid date format: {0}")]
    InvalidDate(String),

    #[error("unknown jurisdiction: {0}")]
    UnknownJurisdiction(String),

    #[error("no default jurisdiction set")]
    NoDefaultJurisdiction,

    #[error("no brackets configured for jurisdiction: {0}")]
    NoBrackets(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("config error: {0}")]
    Config(String),

    #[error("D-Bus error: {0}")]
    DBus(#[from] zbus::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),

    #[error("geoclue error: {0}")]
    GeoClue(String),
}

impl From<Error> for zbus::fdo::Error {
    fn from(e: Error) -> Self {
        zbus::fdo::Error::Failed(e.to_string())
    }
}
