use std::io;

#[derive(Debug, thiserror::Error)]
#[expect(dead_code, reason = "These variants will be used in future stages")]
pub enum ClioError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Browser error: {0}")]
    BrowserError(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),

    #[error("Source error: {0}: {1}")]
    SourceError(String, String),

    #[error("Storage error: {0}")]
    StorageError(String),

    #[error("{0}")]
    Other(String),
}

impl From<io::Error> for ClioError {
    fn from(err: io::Error) -> Self {
        ClioError::IoError(err.to_string())
    }
}

impl From<toml::de::Error> for ClioError {
    fn from(err: toml::de::Error) -> Self {
        ClioError::ConfigError(format!("Failed to parse TOML: {err}"))
    }
}

impl From<reqwest::Error> for ClioError {
    fn from(err: reqwest::Error) -> Self {
        ClioError::NetworkError(err.to_string())
    }
}

impl From<rss::Error> for ClioError {
    fn from(err: rss::Error) -> Self {
        ClioError::ParseError(format!("RSS parsing error: {err}"))
    }
}

impl From<atom_syndication::Error> for ClioError {
    fn from(err: atom_syndication::Error) -> Self {
        ClioError::ParseError(format!("Atom parsing error: {err}"))
    }
}

impl From<chrono::ParseError> for ClioError {
    fn from(err: chrono::ParseError) -> Self {
        ClioError::ParseError(format!("Date parsing error: {err}"))
    }
}

#[expect(
    dead_code,
    reason = "Will be used when we need ClioError-specific results"
)]
pub type Result<T> = std::result::Result<T, ClioError>;

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "Will be used in Stage 2: Configuration Management"
    )
)]
pub fn config_missing_error(path: &str) -> ClioError {
    ClioError::ConfigError(format!(
        "Configuration file not found at: {path}\n\n\
         Please create a configuration file with the following format:\n\n\
         [[sources.rss]]\n\
         name = \"Hacker News\"\n\
         url = \"https://news.ycombinator.com/rss\"\n\n\
         [[sources.rss]]\n\
         name = \"My Blog\"\n\
         url = \"https://example.com/feed.xml\""
    ))
}

#[expect(
    dead_code,
    reason = "Will be used in Stage 2: Configuration Management"
)]
pub fn invalid_url_error(url: &str) -> ClioError {
    ClioError::ConfigError(format!(
        "Invalid URL: {url}. URLs must use HTTP or HTTPS protocol."
    ))
}

#[expect(
    dead_code,
    reason = "Will be used in Stage 2: Configuration Management"
)]
pub fn duplicate_source_name_error(name: &str) -> ClioError {
    ClioError::ConfigError(format!(
        "Duplicate source name found: '{name}'. Each source must have a unique name."
    ))
}

#[expect(dead_code, reason = "Will be used in Stage 9: Browser Integration")]
pub fn item_not_found_error(id: &str) -> ClioError {
    ClioError::InvalidInput(format!("Item with ID '{id}' not found."))
}

#[expect(dead_code, reason = "Will be used in Stage 9: Browser Integration")]
pub fn browser_launch_error(url: &str, err: &str) -> ClioError {
    ClioError::BrowserError(format!(
        "Failed to open browser for URL: {url}\nError: {err}\n\n\
         You can manually open the URL by copying it to your browser."
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = ClioError::ConfigError("test error".to_string());
        assert_eq!(err.to_string(), "Configuration error: test error");
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
        let clio_err: ClioError = io_err.into();
        assert!(matches!(clio_err, ClioError::IoError(_)));
    }

    #[test]
    fn test_config_missing_error() {
        let err = config_missing_error("/home/user/.clio/config.toml");
        assert!(err.to_string().contains("Configuration file not found"));
        assert!(err.to_string().contains("[[sources.rss]]"));
    }

    #[test]
    fn test_source_error() {
        let err = ClioError::SourceError("Test Feed".to_string(), "Connection timeout".to_string());
        assert_eq!(
            err.to_string(),
            "Source error: Test Feed: Connection timeout"
        );
    }
}
