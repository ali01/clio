use std::fmt::Display;

/// Extension trait for adding context to errors
pub trait ErrorContext<T> {
    /// Add a simple string context to the error with a specific error variant
    fn clio_config_err(self, msg: impl Display) -> std::result::Result<T, ClioError>;

    #[allow(dead_code)]
    fn clio_network_err(self, msg: impl Display) -> std::result::Result<T, ClioError>;

    #[allow(dead_code)]
    fn clio_parse_err(self, msg: impl Display) -> std::result::Result<T, ClioError>;
}

impl<T, E: Display> ErrorContext<T> for std::result::Result<T, E> {
    fn clio_config_err(self, msg: impl Display) -> std::result::Result<T, ClioError> {
        self.map_err(|e| ClioError::Config(format!("{msg}: {e}")))
    }

    fn clio_network_err(self, msg: impl Display) -> std::result::Result<T, ClioError> {
        self.map_err(|e| ClioError::Network(format!("{msg}: {e}")))
    }

    fn clio_parse_err(self, msg: impl Display) -> std::result::Result<T, ClioError> {
        self.map_err(|e| ClioError::Parse(format!("{msg}: {e}")))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ClioError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Network error: {0}")]
    Network(String),

    #[error("Parse error: {0}")]
    Parse(String),
}

impl From<toml::de::Error> for ClioError {
    fn from(err: toml::de::Error) -> Self {
        ClioError::Config(format!("Failed to parse TOML: {err}"))
    }
}

impl From<reqwest::Error> for ClioError {
    fn from(err: reqwest::Error) -> Self {
        ClioError::Network(err.to_string())
    }
}

impl From<rss::Error> for ClioError {
    fn from(err: rss::Error) -> Self {
        ClioError::Parse(format!("RSS parsing error: {err}"))
    }
}

impl From<atom_syndication::Error> for ClioError {
    fn from(err: atom_syndication::Error) -> Self {
        ClioError::Parse(format!("Atom parsing error: {err}"))
    }
}

impl From<chrono::ParseError> for ClioError {
    fn from(err: chrono::ParseError) -> Self {
        ClioError::Parse(format!("Date parsing error: {err}"))
    }
}
