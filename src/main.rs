mod cli;
mod error;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    run(cli).await
}

async fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::Pull => execute_pull().await,
        Command::List => execute_list().await,
        Command::Open { item_id } => execute_open(&item_id).await,
    }
}

async fn execute_pull() -> Result<()> {
    println!("Fetching content from configured sources...");
    println!("Note: Pull command implementation coming in Stage 6");
    Ok(())
}

async fn execute_list() -> Result<()> {
    println!("Listing items...");
    println!("Note: List command implementation coming in Stage 8");
    Ok(())
}

async fn execute_open(item_id: &str) -> Result<()> {
    println!("Opening item {item_id}...");
    println!("Note: Open command implementation coming in Stage 9");
    Ok(())
}
