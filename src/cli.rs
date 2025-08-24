use clap::{Parser, Subcommand};

/// A simple command-line feed aggregator
///
/// Clio is a command-line tool for aggregating content from RSS and Atom feeds.
/// It fetches content from configured sources and displays them in a unified,
/// chronological feed that you can browse from your terminal.
#[derive(Parser, Debug)]
#[command(name = "clio", version, author)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Suppress all non-error output
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Fetch latest content from all configured sources
    ///
    /// Fetches the latest content from all configured RSS and Atom feeds.
    /// Sources are fetched in parallel with a 10-second timeout per source.
    /// Failed sources will be reported but won't stop other sources from being fetched.
    Pull,

    /// List fetched items in chronological order
    ///
    /// Displays all fetched items in reverse chronological order (newest first).
    /// Use arrow keys or j/k to navigate, q to quit.
    /// If no items are available, run 'clio pull' first to fetch content.
    List,

    /// Open an item in your default browser
    ///
    /// Opens the specified item in your system's default web browser.
    /// Use the item ID shown in the 'list' command output.
    Open {
        /// The ID of the item to open
        #[arg(value_name = "ITEM_ID")]
        item_id: String,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parse_pull() {
        let cli = Cli::parse_from(["clio", "pull"]);
        assert!(matches!(cli.command, Command::Pull));
    }

    #[test]
    fn test_cli_parse_list() {
        let cli = Cli::parse_from(["clio", "list"]);
        assert!(matches!(cli.command, Command::List));
    }

    #[test]
    fn test_cli_parse_open() {
        let cli = Cli::parse_from(["clio", "open", "item-123"]);
        match cli.command {
            Command::Open { item_id } => assert_eq!(item_id, "item-123"),
            _ => panic!("Expected Open command"),
        }
    }

    #[test]
    fn test_cli_parse_quiet_flag() {
        let cli = Cli::parse_from(["clio", "--quiet", "pull"]);
        assert!(cli.quiet);
    }

    #[test]
    fn test_help_text() {
        let result = Cli::try_parse_from(["clio", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let help_text = err.to_string();
        assert!(help_text.contains("command-line feed aggregator") || help_text.contains("Usage:"));
    }

    #[test]
    fn test_subcommand_help() {
        let result = Cli::try_parse_from(["clio", "pull", "--help"]);
        assert!(result.is_err());
        let err = result.unwrap_err();
        let help_text = err.to_string();
        assert!(help_text.contains("Fetch") || help_text.contains("configured sources"));
    }
}
