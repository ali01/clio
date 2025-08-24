# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Documentation

- **PRD.md**: Product requirements, vision, and target users
- **REQUIREMENTS.md**: Detailed functional and non-functional requirements (EARS format)
- **PLAN.md**: Implementation plan with architecture, stages, and testing strategy

## Development Commands

```bash
# Build & Run
cargo build                   # Debug build
cargo build --release         # Release build
cargo run -- pull             # Fetch content from all sources
cargo run -- list             # Display items with pagination
cargo run -- open [item-id]   # Open item in browser

# Testing
cargo test                    # Run all tests
cargo test -- --nocapture     # Show test output
cargo test test_name          # Run specific test
cargo test module_name::      # Run module tests
cargo test --test '*'         # Integration tests only
cargo test --lib              # Unit tests only

# Code Quality
cargo fmt                     # Format code
cargo fmt -- --check          # Check formatting
cargo clippy -- -D warnings   # Run linter
cargo check                   # Quick compilation check
```

## Key Architectural Notes

- **Source trait**: All content sources implement the `Source` trait in `source/mod.rs` for extensibility
- **Async fetching**: Uses tokio for parallel feed fetching with 10s per-source timeout
- **Storage**: In-memory only for MVP, no persistence between sessions
- **Config location**: `~/.clio/config.toml`
- **Log location**: `~/.clio/clio.log`
