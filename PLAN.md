# Clio MVP Implementation Plan

## Overview

This document outlines the implementation plan for Clio, a command-line feed aggregator written in Rust. The MVP focuses on RSS/Atom feed aggregation with an extensible architecture for future source types.

## Architecture

### Module Structure

```
src/
├── main.rs           # Entry point, CLI setup
├── cli.rs            # Command-line interface using clap
├── config.rs         # Configuration management
├── source/           # Source abstraction and implementations
│   ├── mod.rs        # Source trait definition
│   └── rss.rs        # RSS/Atom feed implementation
├── fetcher.rs        # Parallel content fetching orchestration
├── storage.rs        # In-memory storage management
├── display.rs        # Terminal UI and pagination
├── browser.rs        # Browser launching functionality
└── error.rs          # Error types and handling
```

## Core Types and Interfaces

### Data Structures

#### `Item` (storage.rs)
```rust
struct Item {
    id: String,           // Unique identifier for the session
    source_name: String,  // Name from configuration
    title: String,        // Article title
    link: String,         // URL to the article
    summary: Option<String>, // Article summary/description
    pub_date: Option<DateTime<Utc>>, // Publication date
}
```

#### `Config` (config.rs)
```rust
struct Config {
    sources: Sources,
}

struct Sources {
    rss: Vec<RssSource>,
}

struct RssSource {
    name: String,
    url: String,
}
```

#### `FetchResult` (fetcher.rs)
```rust
enum FetchResult {
    Success { source_name: String, items: Vec<Item> },
    Error { source_name: String, error: String },
}
```

### Traits

#### `Source` (source/mod.rs)
```rust
#[async_trait]
trait Source {
    async fn fetch(&self) -> Result<Vec<Item>, SourceError>;
    fn name(&self) -> &str;
    fn url(&self) -> &str;
}
```

### Error Types

#### `ClioError` (error.rs)
```rust
enum ClioError {
    ConfigError(String),
    NetworkError(String),
    ParseError(String),
    IoError(String),
    BrowserError(String),
}
```

## Dependencies

### Required Crates
- `clap` (v4) - Command-line argument parsing
- `tokio` (v1) - Async runtime for parallel fetching
- `reqwest` (v0.11) - HTTP client with async support
- `rss` (v2) - RSS feed parsing
- `atom_syndication` (v0.12) - Atom feed parsing
- `toml` (v0.8) - Configuration file parsing
- `serde` (v1) - Serialization/deserialization
- `chrono` (v0.4) - Date/time handling
- `crossterm` (v0.27) - Terminal manipulation for pagination
- `html-escape` (v0.2) - HTML entity decoding
- `unicode-width` (v0.1) - Terminal text width calculation
- `directories` (v5) - Platform-specific directory paths

### Testing Dependencies (dev-dependencies)
- `mockito` (v1) - HTTP mocking for network tests
- `tempfile` (v3) - Temporary file/directory creation for tests
- `proptest` (v1) - Property-based testing
- `assert_cmd` (v2) - CLI integration testing
- `predicates` (v3) - Assertions for command testing
- `insta` (v1) - Snapshot testing for output formatting
- `test-case` (v3) - Parameterized test cases
- `criterion` (v0.5) - Performance benchmarking
- `wiremock` (v0.6) - Advanced HTTP mocking
- `fake` (v2) - Test data generation

## Implementation Stages

### Stage 1: Foundation
**Goal**: Set up project structure and basic CLI with testing infrastructure

1. **Set up dependencies** (Cargo.toml)
   - Add all required crates with specific versions
   - Add dev-dependencies for testing
   - Configure edition and basic package metadata

2. **Create test infrastructure**
   - Set up `tests/` directory structure
   - Create `tests/fixtures/` for test data
   - Add test utilities module for common test helpers

3. **Create error types** (error.rs)
   - Define `ClioError` enum with all error variants
   - Implement `Display` and `From` traits for error conversion
   - Create helper functions for common error scenarios
   - **Tests**: Unit tests for error conversion and display

4. **Implement CLI structure** (cli.rs)
   - Define `Cli` struct with clap derive macros
   - Create subcommands: `Pull`, `List`, `Open`
   - Add global flag: `--quiet`
   - Implement help text and usage examples
   - **Tests**: Command parsing tests, help text tests

5. **Wire up main.rs**
   - Parse CLI arguments
   - Route to appropriate command handlers (stubbed)
   - Set up basic error handling and exit codes
   - **Integration tests**: CLI invocation with various arguments

### Stage 2: Configuration Management
**Goal**: Handle configuration file loading and validation with comprehensive testing

1. **Create config types** (config.rs)
   - Define `Config`, `Sources`, and `RssSource` structs with serde
   - Add validation methods for URLs and names
   - **Tests**: Serialization/deserialization tests, validation tests

2. **Implement config loading**
   - Check for ~/.clio/config.toml
   - Create directory with proper permissions if missing
   - Parse TOML file
   - Validate configuration (duplicate names, valid URLs)
   - Handle missing/invalid config with helpful error messages
   - **Tests**: File loading tests with tempfile, permission tests, invalid config tests

3. **Add example config generation**
   - Generate example configuration snippet
   - Display when config is missing
   - **Tests**: Example generation tests

4. **Create test fixtures**
   - Valid config files
   - Invalid config files (malformed TOML, missing fields, duplicate names)
   - Edge cases (empty config, very large config)

### Stage 3: Source Abstraction
**Goal**: Create extensible source system with comprehensive testing

1. **Define Source trait** (source/mod.rs)
   - Async fetch method returning items
   - Name and URL accessors
   - Error handling
   - **Tests**: Mock source implementation for testing

2. **Implement RSS/Atom source** (source/rss.rs)
   - HTTP client setup with timeouts
   - RSS 2.0 parsing
   - Atom 1.0 parsing
   - HTML entity decoding
   - Date parsing (multiple formats)
   - Field extraction and validation
   - **Tests**:
     - Parsing tests with fixture files
     - Network tests with mockito/wiremock
     - Date parsing property tests
     - Malformed feed handling tests
     - Timeout tests

3. **Create test fixtures**
   - Sample RSS 2.0 feeds (valid, minimal, complex)
   - Sample Atom 1.0 feeds
   - Malformed XML files
   - Feeds with various date formats
   - Feeds with HTML entities and Unicode

### Stage 4: Content Fetching
**Goal**: Implement parallel feed fetching with testing

1. **Create fetcher module** (fetcher.rs)
   - Set up tokio runtime
   - Spawn concurrent fetch tasks
   - Implement timeout handling (10s per source)
   - Collect results with error handling
   - Progress feedback during fetching
   - **Tests**:
     - Concurrent fetching with mock sources
     - Timeout behavior tests
     - Error handling with partial failures
     - Progress reporting tests

2. **Add fetch statistics**
   - Track successful/failed sources
   - Count total items fetched
   - Display summary after completion
   - **Tests**: Statistics calculation tests

3. **Performance tests**
   - Test with 100 mock sources
   - Verify 30-second completion
   - Memory usage monitoring

### Stage 5: Storage System
**Goal**: Implement in-memory item storage with testing

1. **Create storage module** (storage.rs)
   - Define `Storage` struct with `Vec<Item>`
   - Implement deduplication using HashSet of URLs
   - Generate unique IDs for items
   - Add sorting methods (chronological)
   - **Tests**:
     - Deduplication tests
     - ID generation uniqueness tests
     - Sorting tests with various date scenarios

2. **Add item management**
   - Store new items
   - Retrieve items by ID
   - Get sorted item list
   - Handle items without dates
   - **Tests**:
     - Item retrieval tests
     - Edge cases (empty storage, invalid IDs)
     - Concurrent access tests
     - Memory usage tests with large datasets

### Stage 6: Pull Command
**Goal**: Implement the full pull workflow with integration testing

1. **Implement pull command handler**
   - Load configuration
   - Create source instances
   - Execute parallel fetching
   - Store items with deduplication
   - Display progress and results
   - **Integration tests**:
     - End-to-end pull with mock HTTP
     - Configuration error handling
     - Progress display tests

2. **Add error recovery**
   - Continue on individual source failures
   - Report errors appropriately
   - Report statistics at completion
   - **Tests**:
     - Partial failure scenarios
     - Network error recovery
     - Statistics accuracy tests

3. **CLI integration tests**
   - Test `clio pull` with various configs
   - Verify output format
   - Test exit codes

### Stage 7: Display System
**Goal**: Create terminal UI for listing items with comprehensive testing

1. **Implement basic display** (display.rs)
   - Format items with truncation
   - Respect terminal width
   - Unicode support
   - **Tests**:
     - Truncation tests with property testing
     - Unicode width calculation tests
     - Terminal width detection tests

2. **Add pagination**
   - Detect terminal height
   - Calculate items per page
   - Implement keyboard navigation
   - Show page indicators
   - **Tests**:
     - Pagination calculation tests
     - Navigation state tests
     - Edge cases (empty list, single item)

3. **Format dates and text**
   - Relative time for recent items
   - Truncate long titles/summaries
   - Normalize whitespace
   - **Tests**:
     - Date formatting tests
     - Whitespace normalization tests
     - Snapshot tests for formatted output

### Stage 8: List Command
**Goal**: Display fetched items with pagination and testing

1. **Implement list command handler**
   - Check for available items
   - Sort items chronologically
   - Launch pagination display
   - Handle empty feed case
   - **Integration tests**:
     - List command with various item counts
     - Empty storage handling
     - Sorting verification

2. **Add navigation controls**
   - Arrow keys and vim keys
   - Home/End navigation
   - Quit handling
   - **Tests**:
     - Key input simulation tests
     - Navigation boundary tests
     - State management tests

### Stage 9: Browser Integration
**Goal**: Open items in default browser with testing

1. **Create browser module** (browser.rs)
   - macOS-specific implementation using `open` command
   - Error handling for launch failures
   - **Tests**:
     - Mock command execution
     - Error handling tests
     - URL escaping tests

2. **Implement open command**
   - Parse item identifier
   - Retrieve item from storage
   - Launch browser with URL
   - Handle invalid identifiers
   - **Integration tests**:
     - Open command with valid/invalid IDs
     - Command output verification
     - Exit code tests

### Stage 10: Polish and Final Testing
**Goal**: Ensure all requirements are met with comprehensive test coverage

1. **Performance optimization and benchmarking**
   - Run criterion benchmarks
   - Verify memory usage under 100MB
   - Ensure 30-second timeout for 100 sources
   - Check CLI responsiveness
   - **Performance tests**:
     - Memory profiling with 100 sources
     - Response time measurements
     - Concurrent operation benchmarks

2. **Test coverage analysis**
   - Generate coverage reports
   - Ensure 80%+ coverage for business logic
   - Add missing tests
   - Document untested edge cases

3. **Integration test suite**
   - Full workflow tests (pull -> list -> open)
   - Error scenario testing
   - Cross-feature interaction tests

4. **Error handling review**
   - Ensure no panics on user input
   - Clear error messages
   - Proper exit codes
   - **Fuzz testing**: Random input testing

5. **Security review**
   - HTTPS preference
   - Certificate validation
   - No credential exposure
   - File permissions
   - **Security tests**: Permission tests, credential protection tests

## How Requirements Are Met

### Configuration Management (CFG-001 to CFG-008)
- Config module handles TOML parsing and validation
- Directory/file creation with proper permissions
- Clear error messages for missing/invalid configs

### Source Management (SRC-001 to SRC-007)
- Trait-based abstraction allows extensibility
- RSS and Atom support through dedicated parsers
- Validation of source names and URLs

### Content Fetching (FET-001 to FET-015)
- Parallel fetching with tokio
- Timeout handling per source
- Graceful error recovery
- Progress feedback
- Deduplication by URL

### Data Storage (STO-001 to STO-006)
- In-memory `Vec<Item>` storage
- Unique ID generation
- No persistence (MVP scope)
- Efficient chronological sorting

### Content Display (DSP-001 to DSP-012)
- Crossterm-based pagination
- Terminal-aware formatting
- Color support with NO_COLOR
- Proper Unicode handling

### Content Interaction (INT-001 to INT-006)
- Open command with browser launching
- macOS-specific implementation
- Error handling with URL display

### CLI Interface (CLI-001 to CLI-008)
- Clap-based command structure
- Comprehensive help text
- Standard exit codes
- Quiet flag for suppressing output

### Error Handling (ERR-001 to ERR-006)
- Structured error types
- Clear error messages
- No panics on user input
- Graceful degradation

### Performance (PRF-001 to PRF-005)
- Async parallel fetching
- Memory-efficient storage
- Fast CLI response
- Quick startup

### Security (SEC-001 to SEC-007)
- HTTPS enforcement
- Certificate validation
- No credential storage or exposure
- Restricted URL schemes

### Testing (TST-001 to TST-015)
- 80%+ unit test coverage for business logic
- Comprehensive integration tests
- Mock infrastructure for external dependencies
- Property-based testing for complex inputs
- Performance benchmarks with criterion
- Test fixtures in version control
- Snapshot testing for output formatting

## Testing Strategy

### Test Organization
- **Unit Tests**: In-module tests using `#[cfg(test)]` blocks
- **Integration Tests**: `tests/` directory for end-to-end testing
- **Test Fixtures**: `tests/fixtures/` for sample data
- **Benchmarks**: `benches/` directory for performance tests

### Testing Approach Per Module
- **CLI**: Command parsing, help text, exit codes
- **Config**: TOML parsing, validation, error handling
- **Source**: Feed parsing with fixtures, network mocking, timeout testing
- **Fetcher**: Concurrent operations, partial failures, progress tracking
- **Storage**: Deduplication, sorting, memory usage
- **Display**: Truncation, Unicode, pagination calculations
- **Browser**: Command execution mocking, URL handling

### Mock Infrastructure
- **Network**: `mockito` or `wiremock` for HTTP responses
- **Filesystem**: `tempfile` for temporary test directories
- **Time**: Deterministic clock for date testing
- **Browser**: Command execution capture

### Test Data Strategy
- Minimal but representative fixtures
- Edge cases (empty, malformed, very large)
- Various encoding and character sets
- Multiple date formats

### Continuous Testing
- Run tests with every commit
- Coverage reports with `cargo tarpaulin`
- Benchmark regression detection
- Property test with `proptest` for fuzzing

## Future Extensibility

The architecture supports post-MVP features:
- New source types (implement Source trait)
- Persistent storage (replace Storage implementation)
- Additional commands (extend CLI enum)
- Web UI (separate crate using core modules)
- Advanced filtering (extend Storage queries)
