use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;

pub fn create_test_config(content: &str) -> (TempDir, PathBuf) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let config_dir = temp_dir.path().join(".clio");
    fs::create_dir_all(&config_dir).expect("Failed to create config dir");

    let config_path = config_dir.join("config.toml");
    fs::write(&config_path, content).expect("Failed to write config");

    (temp_dir, config_path)
}

pub fn sample_rss_config() -> String {
    r#"
[[sources.rss]]
name = "Test Feed"
url = "https://example.com/rss"

[[sources.rss]]
name = "Another Feed"
url = "https://test.com/feed"
"#
    .to_string()
}

pub fn invalid_toml_config() -> String {
    r#"
[[sources.rss]]
name = "Test Feed
url = "https://example.com/rss"
"#
    .to_string()
}

pub fn empty_config() -> String {
    "".to_string()
}

pub fn duplicate_names_config() -> String {
    r#"
[[sources.rss]]
name = "Test Feed"
url = "https://example.com/rss"

[[sources.rss]]
name = "Test Feed"
url = "https://test.com/feed"
"#
    .to_string()
}
