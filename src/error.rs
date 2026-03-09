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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display_messages() {
        assert_eq!(Error::NoBirthdate.to_string(), "no birthdate stored");
        assert_eq!(
            Error::InvalidDate("bad".into()).to_string(),
            "invalid date format: bad"
        );
        assert_eq!(
            Error::UnknownJurisdiction("US/TX".into()).to_string(),
            "unknown jurisdiction: US/TX"
        );
        assert_eq!(
            Error::NoDefaultJurisdiction.to_string(),
            "no default jurisdiction set"
        );
        assert_eq!(
            Error::NoBrackets("X".into()).to_string(),
            "no brackets configured for jurisdiction: X"
        );
        assert_eq!(
            Error::Storage("oops".into()).to_string(),
            "storage error: oops"
        );
        assert_eq!(Error::Config("bad".into()).to_string(), "config error: bad");
        assert_eq!(
            Error::GeoClue("fail".into()).to_string(),
            "geoclue error: fail"
        );
    }

    #[test]
    fn error_to_zbus_fdo() {
        let err: zbus::fdo::Error = Error::NoBirthdate.into();
        match err {
            zbus::fdo::Error::Failed(msg) => assert_eq!(msg, "no birthdate stored"),
            other => panic!("expected Failed, got {other:?}"),
        }
    }

    #[test]
    fn io_error_converts() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "gone");
        let err: Error = io_err.into();
        assert!(err.to_string().contains("gone"));
    }
}
