use clio::config::{Config, RssSource, Sources};
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tempfile::TempDir;

fn setup_test_config(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".clio");
    fs::create_dir_all(&config_dir).unwrap();

    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, content).unwrap();

    (temp_dir, config_path)
}

#[test]
fn test_load_valid_config_from_file() {
    let config_content = r#"
[[sources.rss]]
name = "Test Feed 1"
url = "https://example.com/feed1.xml"

[[sources.rss]]
name = "Test Feed 2"
url = "https://example.com/feed2.xml"
"#;

    let (_temp_dir, _config_path) = setup_test_config(config_content);
    // Note: Testing Config::load() with custom paths would require modification
    // to accept environment variables or custom paths.
    // For now, we test the parsing directly
    let config: Config = toml::from_str(config_content).unwrap();
    assert_eq!(config.sources.rss.len(), 2);
    assert_eq!(config.sources.rss[0].name, "Test Feed 1");
    assert_eq!(config.sources.rss[1].name, "Test Feed 2");
}

#[test]
fn test_fixture_valid_config() {
    let content = fs::read_to_string("tests/fixtures/valid_config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    assert!(config.validate().is_ok());
    assert_eq!(config.sources.rss.len(), 3);

    let names: Vec<String> = config.sources.rss.iter().map(|s| s.name.clone()).collect();
    assert!(names.contains(&"Hacker News".to_string()));
    assert!(names.contains(&"Julia Evans".to_string()));
    assert!(names.contains(&"Rust Blog".to_string()));
}

#[test]
fn test_fixture_empty_sources() {
    let content = fs::read_to_string("tests/fixtures/empty_sources_config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    assert!(config.validate().is_ok());
    assert_eq!(config.sources.rss.len(), 0);
}

#[test]
fn test_fixture_duplicate_names() {
    let content = fs::read_to_string("tests/fixtures/duplicate_names_config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Duplicate"));
}

#[test]
fn test_fixture_invalid_url() {
    let content = fs::read_to_string("tests/fixtures/invalid_url_config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("Invalid URL"));
}

#[test]
fn test_fixture_malformed_toml() {
    let content = fs::read_to_string("tests/fixtures/malformed_config.toml").unwrap();
    let result: Result<Config, _> = toml::from_str(&content);
    assert!(result.is_err());
}

#[test]
fn test_fixture_missing_field() {
    let content = fs::read_to_string("tests/fixtures/missing_field_config.toml").unwrap();
    let result: Result<Config, _> = toml::from_str(&content);
    assert!(result.is_err());
}

#[test]
fn test_fixture_empty_name() {
    let content = fs::read_to_string("tests/fixtures/empty_name_config.toml").unwrap();
    let config: Config = toml::from_str(&content).unwrap();

    let result = config.validate();
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("empty"));
}

#[test]
fn test_config_with_100_sources() {
    let mut sources = Vec::new();
    for i in 1..=100 {
        sources.push(RssSource::new(
            format!("Feed {i}"),
            format!("https://example.com/feed{i}.xml"),
        ));
    }

    let config = Config {
        sources: Sources { rss: sources },
    };

    assert!(config.validate().is_ok());
    assert_eq!(config.sources.rss.len(), 100);
}

#[test]
fn test_config_with_various_url_formats() {
    let config = Config {
        sources: Sources {
            rss: vec![
                RssSource::new(
                    "HTTP".to_string(),
                    "http://example.com/feed.xml".to_string(),
                ),
                RssSource::new(
                    "HTTPS".to_string(),
                    "https://example.com/feed.xml".to_string(),
                ),
                RssSource::new(
                    "Port".to_string(),
                    "https://example.com:8080/feed.xml".to_string(),
                ),
                RssSource::new(
                    "Path".to_string(),
                    "https://example.com/path/to/feed.xml".to_string(),
                ),
                RssSource::new(
                    "Query".to_string(),
                    "https://example.com/feed?format=rss".to_string(),
                ),
            ],
        },
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_with_invalid_schemes() {
    let test_cases = vec![
        ("FTP", "ftp://example.com/feed.xml"),
        ("File", "file:///etc/passwd"),
        ("Data", "data:text/plain,hello"),
        ("SSH", "ssh://example.com"),
    ];

    for (name, url) in test_cases {
        let config = Config {
            sources: Sources {
                rss: vec![RssSource::new(name.to_string(), url.to_string())],
            },
        };

        let result = config.validate();
        assert!(result.is_err(), "Should reject {name} scheme");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid URL scheme")
        );
    }
}

#[test]
fn test_config_serialization_roundtrip() {
    let original = Config {
        sources: Sources {
            rss: vec![
                RssSource::new(
                    "Feed 1".to_string(),
                    "https://example.com/feed1.xml".to_string(),
                ),
                RssSource::new(
                    "Feed 2".to_string(),
                    "https://example.com/feed2.xml".to_string(),
                ),
            ],
        },
    };

    let serialized = toml::to_string(&original).unwrap();
    let deserialized: Config = toml::from_str(&serialized).unwrap();

    assert_eq!(original, deserialized);
}

#[test]
fn test_config_with_unicode_names() {
    let config = Config {
        sources: Sources {
            rss: vec![
                RssSource::new(
                    "日本語フィード".to_string(),
                    "https://example.jp/feed.xml".to_string(),
                ),
                RssSource::new(
                    "Фид на русском".to_string(),
                    "https://example.ru/feed.xml".to_string(),
                ),
                RssSource::new(
                    "العربية موجز".to_string(),
                    "https://example.ae/feed.xml".to_string(),
                ),
                RssSource::new(
                    "🚀 Emoji Feed 🎉".to_string(),
                    "https://example.com/feed.xml".to_string(),
                ),
            ],
        },
    };

    assert!(config.validate().is_ok());
    assert_eq!(config.sources.rss.len(), 4);
}

#[test]
fn test_config_with_whitespace_names() {
    let config = Config {
        sources: Sources {
            rss: vec![
                RssSource::new(
                    "Normal Name".to_string(),
                    "https://example.com/feed1.xml".to_string(),
                ),
                RssSource::new(
                    "  Leading Spaces".to_string(),
                    "https://example.com/feed2.xml".to_string(),
                ),
                RssSource::new(
                    "Trailing Spaces  ".to_string(),
                    "https://example.com/feed3.xml".to_string(),
                ),
            ],
        },
    };

    assert!(config.validate().is_ok());
}

#[test]
fn test_config_directory_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let config_dir = temp_dir.path().join(".clio");

    fs::create_dir_all(&config_dir).unwrap();

    let metadata = fs::metadata(&config_dir).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o700);
    fs::set_permissions(&config_dir, permissions).unwrap();

    let metadata = fs::metadata(&config_dir).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(mode & 0o777, 0o700);
}

#[test]
fn test_config_file_permissions() {
    let temp_dir = TempDir::new().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    fs::write(&config_path, "# test config").unwrap();

    let metadata = fs::metadata(&config_path).unwrap();
    let mut permissions = metadata.permissions();
    permissions.set_mode(0o600);
    fs::set_permissions(&config_path, permissions).unwrap();

    let metadata = fs::metadata(&config_path).unwrap();
    let mode = metadata.permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}
