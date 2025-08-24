use assert_cmd::Command;
use predicates::prelude::*;
use std::sync::Mutex;

// Use a mutex to ensure tests that manipulate ~/.clio don't run in parallel
static CONFIG_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn test_clio_without_args_shows_help() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.assert()
        .failure()
        .stderr(predicate::str::contains("Usage:"))
        .stderr(predicate::str::contains("Commands:"));
}

#[test]
fn test_clio_help_flag() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("command-line"))
        .stdout(predicate::str::contains("Commands:"))
        .stdout(predicate::str::contains("pull"))
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("open"));
}

#[test]
fn test_clio_version_flag() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("clio"));
}

#[test]
fn test_pull_command() {
    let _guard = CONFIG_MUTEX.lock().unwrap();

    // Clean up any existing config
    let config_dir = dirs::home_dir().unwrap().join(".clio");
    let _ = std::fs::remove_dir_all(&config_dir);

    // Pull command should create config silently and work with default config
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("pull")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Fetching content from 2 configured sources",
        ));

    // Verify config was created
    assert!(config_dir.join("config.toml").exists());

    // Clean up after test
    let _ = std::fs::remove_dir_all(&config_dir);
}

#[test]
fn test_pull_command_with_quiet() {
    let _guard = CONFIG_MUTEX.lock().unwrap();

    // Clean up any existing config
    let config_dir = dirs::home_dir().unwrap().join(".clio");
    let _ = std::fs::remove_dir_all(&config_dir);

    // Pull command should create config silently and work even with --quiet
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("--quiet")
        .arg("pull")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Fetching content from 2 configured sources",
        ));

    // Verify config was created
    assert!(config_dir.join("config.toml").exists());

    // Clean up after test
    let _ = std::fs::remove_dir_all(&config_dir);
}

#[test]
fn test_list_command() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("list")
        .assert()
        .success()
        .stdout(predicate::str::contains("Listing items"));
}

#[test]
fn test_open_command_without_id() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("open")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required"));
}

#[test]
fn test_open_command_with_id() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("open")
        .arg("test-id")
        .assert()
        .success()
        .stdout(predicate::str::contains("Opening item test-id"));
}

#[test]
fn test_pull_help() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("pull")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Fetch"));
}

#[test]
fn test_list_help() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("list")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("fetched items"));
}

#[test]
fn test_open_help() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("open")
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Open"));
}

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("invalid")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized subcommand"));
}

#[test]
fn test_global_flags_order() {
    // Global flags should work before the subcommand
    // Using 'list' instead of 'pull' since pull requires config
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("--quiet").arg("list").assert().success();

    // Global flags should also work after the subcommand
    let mut cmd = Command::cargo_bin("clio").unwrap();
    cmd.arg("list").arg("--quiet").assert().success();
}
