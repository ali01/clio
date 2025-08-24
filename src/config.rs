use crate::error::{ClioError, ErrorContext};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use url::Url;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub sources: Sources,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Sources {
    #[serde(default)]
    pub rss: Vec<RssSource>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RssSource {
    pub name: String,
    pub url: String,
}

impl Config {
    pub fn load() -> Result<Self, ClioError> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            Self::init_with_example()?;
        }

        let contents = fs::read_to_string(&config_path).clio_config_err(format!(
            "Failed to read configuration file at {}",
            config_path.display()
        ))?;

        let config: Self =
            toml::from_str(&contents).clio_config_err("Failed to parse configuration file")?;

        config.validate()?;

        Ok(config)
    }

    pub fn validate(&self) -> Result<(), ClioError> {
        if self.sources.rss.is_empty() {
            eprintln!("Warning: No sources configured");
        }

        let mut seen_names = HashSet::new();
        for source in &self.sources.rss {
            if source.name.trim().is_empty() {
                return Err(ClioError::Config("Source name cannot be empty".to_string()));
            }

            if !seen_names.insert(source.name.clone()) {
                return Err(ClioError::Config(format!(
                    "Duplicate source name: {}",
                    source.name
                )));
            }

            Self::validate_url(&source.url)?;
        }

        Ok(())
    }

    fn config_path() -> Result<PathBuf, ClioError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ClioError::Config("Could not determine home directory".to_string()))?;
        Ok(home_dir.join(".clio").join("config.toml"))
    }

    fn config_dir() -> Result<PathBuf, ClioError> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| ClioError::Config("Could not determine home directory".to_string()))?;
        Ok(home_dir.join(".clio"))
    }

    fn validate_url(url_str: &str) -> Result<(), ClioError> {
        let url = Url::parse(url_str)
            .map_err(|e| ClioError::Config(format!("Invalid URL '{url_str}': {e}")))?;

        match url.scheme() {
            "http" | "https" => Ok(()),
            scheme => Err(ClioError::Config(format!(
                "Invalid URL scheme '{scheme}': only HTTP and HTTPS are supported"
            ))),
        }
    }

    fn init_with_example() -> Result<(), ClioError> {
        Self::ensure_config_dir()?;
        let config_path = Self::config_path()?;

        let example_config = include_str!("../data/example_config.toml");

        fs::write(&config_path, example_config).clio_config_err({
            format!(
                "Failed to write example configuration to {}",
                config_path.display()
            )
        })?;

        let metadata = fs::metadata(&config_path).clio_config_err(format!(
            "Failed to get metadata for {}",
            config_path.display()
        ))?;
        let mut permissions = metadata.permissions();
        permissions.set_mode(0o600);
        fs::set_permissions(&config_path, permissions).clio_config_err(format!(
            "Failed to set permissions for {}",
            config_path.display()
        ))?;

        Ok(())
    }

    fn ensure_config_dir() -> Result<(), ClioError> {
        let config_dir = Self::config_dir()?;

        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).clio_config_err({
                format!(
                    "Failed to create configuration directory at {}",
                    config_dir.display()
                )
            })?;

            let metadata = fs::metadata(&config_dir).clio_config_err(format!(
                "Failed to get metadata for {}",
                config_dir.display()
            ))?;
            let mut permissions = metadata.permissions();
            permissions.set_mode(0o700);
            fs::set_permissions(&config_dir, permissions).clio_config_err({
                format!("Failed to set permissions for {}", config_dir.display())
            })?;
        }

        Ok(())
    }
}

impl RssSource {
    // Public for use in integration tests
    #[allow(dead_code)]
    pub fn new(name: String, url: String) -> Self {
        Self { name, url }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_serialization() {
        let config = Config {
            sources: Sources {
                rss: vec![
                    RssSource::new(
                        "Test Feed 1".to_string(),
                        "https://example.com/feed1.xml".to_string(),
                    ),
                    RssSource::new(
                        "Test Feed 2".to_string(),
                        "https://example.com/feed2.xml".to_string(),
                    ),
                ],
            },
        };

        let serialized = toml::to_string(&config).unwrap();
        let deserialized: Config = toml::from_str(&serialized).unwrap();
        assert_eq!(config, deserialized);
    }

    #[test]
    fn test_config_validation_empty_name() {
        let config = Config {
            sources: Sources {
                rss: vec![RssSource::new(
                    "".to_string(),
                    "https://example.com/feed.xml".to_string(),
                )],
            },
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("empty"));
    }

    #[test]
    fn test_config_validation_duplicate_names() {
        let config = Config {
            sources: Sources {
                rss: vec![
                    RssSource::new(
                        "Duplicate".to_string(),
                        "https://example.com/feed1.xml".to_string(),
                    ),
                    RssSource::new(
                        "Duplicate".to_string(),
                        "https://example.com/feed2.xml".to_string(),
                    ),
                ],
            },
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Duplicate"));
    }

    #[test]
    fn test_config_validation_invalid_url() {
        let config = Config {
            sources: Sources {
                rss: vec![RssSource::new("Test".to_string(), "not-a-url".to_string())],
            },
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Invalid URL"));
    }

    #[test]
    fn test_config_validation_invalid_scheme() {
        let config = Config {
            sources: Sources {
                rss: vec![RssSource::new(
                    "Test".to_string(),
                    "ftp://example.com/feed.xml".to_string(),
                )],
            },
        };

        let result = config.validate();
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid URL scheme")
        );
    }

    #[test]
    fn test_config_validation_valid() {
        let config = Config {
            sources: Sources {
                rss: vec![
                    RssSource::new(
                        "Feed 1".to_string(),
                        "https://example.com/feed1.xml".to_string(),
                    ),
                    RssSource::new(
                        "Feed 2".to_string(),
                        "http://example.com/feed2.xml".to_string(),
                    ),
                ],
            },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_config_validation_empty_sources() {
        let config = Config {
            sources: Sources { rss: vec![] },
        };

        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_url_validation() {
        assert!(Config::validate_url("https://example.com/feed.xml").is_ok());
        assert!(Config::validate_url("http://example.com/feed.xml").is_ok());
        assert!(Config::validate_url("https://example.com:8080/feed.xml").is_ok());

        assert!(Config::validate_url("ftp://example.com/feed.xml").is_err());
        assert!(Config::validate_url("file:///etc/passwd").is_err());
        assert!(Config::validate_url("not-a-url").is_err());
        assert!(Config::validate_url("").is_err());
    }

    #[test]
    fn test_parse_valid_toml() {
        let toml_content = r#"
[[sources.rss]]
name = "Test Feed"
url = "https://example.com/feed.xml"

[[sources.rss]]
name = "Another Feed"
url = "http://example.org/rss"
"#;

        let config: Config = toml::from_str(toml_content).unwrap();
        assert_eq!(config.sources.rss.len(), 2);
        assert_eq!(config.sources.rss[0].name, "Test Feed");
        assert_eq!(config.sources.rss[1].name, "Another Feed");
    }

    #[test]
    fn test_parse_empty_config() {
        let toml_content = "";
        let result: Result<Config, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_malformed_toml() {
        let toml_content = r#"
[[sources.rss]]
name = "Test Feed
url = "https://example.com/feed.xml"
"#;

        let result: Result<Config, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_missing_required_fields() {
        let toml_content = r#"
[[sources.rss]]
name = "Test Feed"
"#;

        let result: Result<Config, _> = toml::from_str(toml_content);
        assert!(result.is_err());
    }
}
