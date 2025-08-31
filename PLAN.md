# Clio MVP Implementation Plan

## Progress Summary

- ✅ **Stage 1: Foundation** - CLI structure, error handling, testing infrastructure
- ✅ **Stage 2: Configuration Management** - TOML config loading and validation (needs Supabase config addition)
- ✅ **Stage 3: Source Abstraction** - Source trait and RSS/Atom implementation
- ✅ **Stage 4: Content Fetching** - Parallel feed fetching with timeout and stats
- ⏳ **Stage 5: Database Setup** - Supabase connection and schema
- ⏳ **Stage 6: Storage System** - Supabase-backed persistent storage
- ⏳ **Stage 7: Pull Command** - Full pull workflow with database persistence
- ⏳ **Stage 8: Display System** - Terminal UI and pagination with read/unread filtering
- ⏳ **Stage 9: List Command** - Display fetched items from database
- ⏳ **Stage 10: Browser Integration** - Open items and mark as read
- ⏳ **Stage 11: Read Status Commands** - Mark read/unread functionality
- ⏳ **Stage 12: Polish and Final Testing** - Performance and coverage

## Overview

This document outlines the implementation plan for Clio, a command-line feed aggregator written in Rust. The MVP focuses on RSS/Atom feed aggregation with persistent storage using Supabase PostgreSQL, providing an extensible architecture for future source types and enabling cross-session state management.

## Architecture

### Module Structure

```
src/
├── main.rs           # Entry point, CLI setup
├── cli.rs            # Command-line interface using clap
├── config.rs         # Configuration management
├── source.rs         # Source trait definition (modern Rust 2025 style)
├── source/           # Source implementations
│   └── rss.rs        # RSS/Atom feed implementation
├── fetcher.rs        # Parallel content fetching orchestration (not yet implemented)
├── storage.rs        # Supabase-backed persistent storage (not yet implemented)
├── database.rs       # Database connection and schema management (not yet implemented)
├── display.rs        # Terminal UI and pagination (not yet implemented)
├── browser.rs        # Browser launching functionality (not yet implemented)
├── lib.rs            # Library root, module declarations
└── error.rs          # Error types and handling
```

## Core Types and Interfaces

### Data Structures

#### `Item` (storage.rs)
```rust
struct Item {
    id: Uuid,             // Unique identifier (database-generated)
    source_name: String,  // Name from configuration
    title: String,        // Article title
    link: String,         // URL to the article (unique constraint)
    summary: Option<String>, // Article summary/description
    pub_date: Option<DateTime<Utc>>, // Publication date
    is_read: bool,        // Read/unread status
    created_at: DateTime<Utc>, // First fetched timestamp
    updated_at: DateTime<Utc>, // Last updated timestamp
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

// Supabase connection handled separately via environment
struct SupabaseClient {
    url: String,         // From SUPABASE_URL env var
    secret_key: String,  // From SUPABASE_SECRET_KEY env var
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

#### `Source` (source.rs)
```rust
#[async_trait]
pub trait Source: Send + Sync + Debug {
    async fn pull(&self) -> Result<Vec<Item>, ClioError>;
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
    DatabaseError(String),
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
- `postgrest` (v1) - Supabase PostgREST client
- `uuid` (v1) - UUID generation and handling
- `sqlx` (v0.7) - Alternative: Direct PostgreSQL access if needed

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

### Stage 1: Foundation ✅ COMPLETED
**Goal**: Set up project structure and basic CLI with testing infrastructure

1. **Set up dependencies** (Cargo.toml) ✅
   - Added all required crates with specific versions
   - Added dev-dependencies for testing
   - Configured edition 2024 and basic package metadata

2. **Create test infrastructure** ✅
   - Set up `tests/` directory structure
   - Created `tests/fixtures/` with sample RSS and Atom feeds
   - Added test utilities module (`tests/common.rs`) for common test helpers
   - Need to add: Database test utilities and mock Supabase client

3. **Create error types** (error.rs) ✅
   - Defined `ClioError` enum with all error variants
   - Implemented `Display` and `From` traits for error conversion
   - Created helper functions for common error scenarios
   - Added unit tests for error conversion and display
   - Used `#[expect(dead_code)]` for future-use items

4. **Implement CLI structure** (cli.rs) ✅
   - Defined `Cli` struct with clap derive macros
   - Created subcommands: `Pull`, `List`, `Open`
   - Added global flag: `--quiet`
   - Implemented help text using doc comments
   - Added command parsing tests and help text tests

5. **Wire up main.rs** ✅
   - Parse CLI arguments using `anyhow::Result`
   - Route to appropriate command handlers (stubbed)
   - Set up basic error handling with automatic exit codes
   - Added 13 integration tests for CLI invocation

**Implementation Notes:**
- Removed logging infrastructure to keep the MVP simple
- Using Rust edition 2024
- Using `anyhow::Result` for cleaner error handling in main
- Using doc comments for clap commands instead of attributes
- Using `#[expect(dead_code)]` to track unused code that will be used later
- Configured rustfmt with stable-only options

### Stage 2: Configuration Management ⚠️ NEEDS UPDATE
**Goal**: Handle configuration file loading and validation with comprehensive testing
**Update Required**: Add environment variable validation for Supabase

1. **Create config types** (config.rs) ⚠️ NEEDS UPDATE
   - Defined `Config`, `Sources`, and `RssSource` structs with serde
   - Config file contains only source definitions (no secrets)
   - Added validation methods for URLs and names
   - Implemented comprehensive validation (duplicate names, empty fields, URL format)
   - Added 30+ unit tests for all validation scenarios

2. **Implement config loading** ⚠️ NEEDS UPDATE
   - Check for ~/.clio/config.toml with proper platform paths
   - Create directory with proper permissions if missing
   - Parse TOML file with detailed error messages
   - Validate configuration (duplicate names, valid URLs, empty fields)
   - Handle missing/invalid config with helpful error messages
   - Added integration tests for file loading with tempfile

3. **Environment variable validation** ⚠️ TODO
   - Check for SUPABASE_URL and SUPABASE_SECRET_KEY
   - Validate URL format (must be HTTPS)
   - Validate secret key format (must start with 'sb_secret_')
   - Never log or display the secret key
   - Clear error messages if variables are missing
   - Exit gracefully if not configured

3. **Add example config generation** ✅
   - Created example configuration in data/example_config.toml
   - Display helpful message when config is missing
   - Provide clear instructions for config setup

4. **Create test fixtures** ✅
   - Valid config files (valid_config.toml)
   - Invalid config files:
     - malformed_config.toml (invalid TOML syntax)
     - missing_field_config.toml (missing required fields)
     - duplicate_names_config.toml (duplicate source names)
     - empty_name_config.toml (empty source name)
     - empty_sources_config.toml (no sources defined)
     - invalid_url_config.toml (malformed URL)
   - Edge cases (large_config.toml with many sources)
   - No Supabase credentials in any config files

**Implementation Notes:**
- Using `directories` crate for platform-specific config paths
- Config validation happens at parse time for immediate feedback
- Supabase credentials strictly from environment variables
- Secret key never stored in files or logged
- All error messages include context and suggested fixes
- Comprehensive test coverage with 292 lines of integration tests
- Using `#[expect(dead_code)]` for methods that will be used in later stages

### Stage 3: Source Abstraction ✅ COMPLETED
**Goal**: Create extensible source system with comprehensive testing

1. **Define Source trait** (source.rs) ✅
   - Async pull method returning items (renamed from fetch for consistency)
   - Name and URL accessors
   - Error handling with ClioError
   - Send + Sync + Debug bounds for async usage
   - **Tests**: Mock source implementation for testing

2. **Implement RSS/Atom source** (source/rss.rs) ✅
   - HTTP client setup with 10-second timeout
   - RSS 2.0 parsing with `rss` crate
   - Atom 1.0 parsing with `atom_syndication` crate
   - HTML entity decoding with `html_escape`
   - Date parsing (RFC 2822, RFC 3339, ISO 8601, and variants)
   - Field extraction and validation
   - Whitespace normalization
   - UUID generation for item IDs
   - **Tests**:
     - Unit tests for all parsing functions
     - Integration tests with wiremock
     - Property-based testing with proptest
     - Malformed feed handling tests
     - HTTP error handling tests

3. **Create test fixtures** ✅
   - Sample RSS 2.0 feeds (detailed, with entities, date formats)
   - Sample Atom 1.0 feeds (detailed)
   - Malformed XML files
   - Feeds with various date formats
   - Feeds with HTML entities and Unicode

**Implementation Notes:**
- Used modern Rust 2025 style with `source.rs` instead of `source/mod.rs`
- Renamed `RssFeedSource` to `RssSource` for brevity
- Renamed `fetch()` method to `pull()` for consistency with CLI command
- Used `unwrap_or_default()` for safer HTTP client initialization
- Fixed all clippy lints (inline format strings, no unnecessary allocations)
- Comprehensive test coverage with 27 unit tests and 18 integration tests

### Stage 4: Content Fetching ✅ COMPLETED
**Goal**: Implement parallel feed fetching with testing

1. **Create fetcher module** (fetcher.rs) ✅
   - Set up tokio runtime with futures::join_all
   - Spawn concurrent fetch tasks using tokio::spawn
   - Implement timeout handling (10s per source, configurable)
   - Collect results with error handling
   - Progress feedback during fetching
   - **Tests**:
     - 10 unit tests for concurrent fetching with mock sources
     - Timeout behavior tests with custom delays
     - Error handling with partial failures
     - Progress reporting tests

2. **Add fetch statistics** ✅
   - FetchStats struct tracks successful/failed sources
   - Count total items fetched
   - Display summary after completion
   - Collect error messages for failed sources
   - **Tests**: Statistics calculation and display tests

3. **Performance tests** ✅
   - Created benchmarks with 100 mock sources
   - Verify 30-second completion requirement
   - Memory usage monitoring (<10MB for 100 sources)
   - Integration tests with up to 50 sources

**Implementation Notes:**
- Renamed `pull()` method to `fetch()` throughout codebase
- Used `join_all` for cleaner async handling
- Fetcher uses `fetch_one()` internally to avoid code duplication
- Added `futures = "0.3"` dependency for join_all support
- Total: 10 fetcher unit tests + 9 integration tests + 5 benchmarks

### Stage 5: Database Setup
**Goal**: Establish Supabase connection and database schema

1. **Create database module** (database.rs)
   - Load SUPABASE_URL and SUPABASE_SECRET_KEY from environment
   - Validate environment variables (exit if missing)
   - Define Supabase client initialization with secret key
   - Connection pool management
   - Schema migration/verification on startup
   - Create items table if not exists:
     ```sql
     CREATE TABLE IF NOT EXISTS items (
       id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
       source_name TEXT NOT NULL,
       title TEXT NOT NULL,
       link TEXT NOT NULL UNIQUE,
       summary TEXT,
       pub_date TIMESTAMPTZ,
       is_read BOOLEAN DEFAULT FALSE,
       created_at TIMESTAMPTZ DEFAULT NOW(),
       updated_at TIMESTAMPTZ DEFAULT NOW()
     );
     CREATE INDEX idx_items_pub_date ON items(pub_date DESC);
     CREATE INDEX idx_items_created_at ON items(created_at DESC);
     CREATE INDEX idx_items_is_read ON items(is_read);
     ```
     <!-- Note: Secret key allows DDL operations -->
   - **Tests**:
     - Connection establishment tests with mocked env vars
     - Schema creation tests
     - Missing environment variable tests
     - Invalid URL format tests
     - Connection failure handling

2. **Add Supabase client wrapper**
   - Authentication with secret key (full access, bypasses RLS)
   - Error handling for network issues
   - Retry logic for transient failures
   - Secret key protection (never log or display)
   - **Tests**:
     - Mock client for testing with fake credentials
     - Environment variable mocking
     - Auth failure tests
     - Network timeout tests
     - Key protection tests (ensure no leaks)

### Stage 6: Storage System
**Goal**: Implement Supabase-backed persistent storage with testing

1. **Create storage module** (storage.rs)
   - Define `Storage` struct wrapping Supabase client
   - Implement CRUD operations:
     - Insert new items (with duplicate check)
     - Update existing items
     - Query items with filters (read/unread, date range)
     - Mark items as read/unread
   - Pagination support for large result sets
   - **Tests**:
     - CRUD operation tests with mock database
     - Deduplication tests (unique constraint)
     - Pagination tests
     - Concurrent access tests

2. **Add item management**
   - Batch insert for efficiency
   - Query items by ID (UUID)
   - Get sorted item list with filters
   - Update read status
   - **Tests**:
     - Batch operation tests
     - Filter and sorting tests
     - Transaction tests
     - Performance tests with large datasets

### Stage 7: Pull Command
**Goal**: Implement the full pull workflow with database persistence

1. **Implement pull command handler**
   - Load configuration (sources only)
   - Validate environment variables for Supabase
   - Initialize database connection with secret key
   - Create source instances
   - Execute parallel fetching
   - Store items in database with deduplication
   - Update existing items' updated_at timestamp
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

### Stage 8: Display System
**Goal**: Create terminal UI for listing items with read/unread filtering

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

### Stage 9: List Command
**Goal**: Display fetched items from database with pagination and filtering

1. **Implement list command handler**
   - Validate environment variables
   - Connect to database with secret key
   - Query items (default: unread only)
   - Add --all flag to show read items
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

### Stage 10: Browser Integration
**Goal**: Open items in default browser and mark as read

1. **Create browser module** (browser.rs)
   - macOS-specific implementation using `open` command
   - Error handling for launch failures
   - **Tests**:
     - Mock command execution
     - Error handling tests
     - URL escaping tests

2. **Implement open command**
   - Validate environment variables
   - Parse item identifier (UUID)
   - Retrieve item from database using secret key
   - Launch browser with URL
   - Mark item as read in database
   - Handle invalid identifiers
   - **Integration tests**:
     - Open command with valid/invalid IDs
     - Command output verification
     - Exit code tests

### Stage 11: Read Status Commands
**Goal**: Implement mark read/unread functionality

1. **Implement mark-read command**
   - Parse item identifier(s)
   - Update is_read status in database
   - Provide feedback on success/failure
   - **Tests**:
     - Single item marking
     - Multiple item marking
     - Invalid ID handling

2. **Implement mark-unread command**
   - Parse item identifier(s)
   - Update is_read status to false
   - Provide feedback
   - **Tests**: Similar to mark-read

3. **Implement mark-all-read command**
   - Update all items in database
   - Optional filter by source
   - Confirmation prompt
   - **Tests**:
     - Bulk update tests
     - Filter tests
     - Confirmation bypass flag

### Stage 12: Polish and Final Testing
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
   - Full workflow tests (pull -> list -> open -> mark-read)
   - Database persistence tests across sessions
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
   - Supabase key protection
   - No credential exposure in logs
   - File permissions (especially for config with keys)
   - **Security tests**: Permission tests, credential protection tests, key redaction tests

## How Requirements Are Met

### Configuration Management (CFG-001 to CFG-011)
- Config module handles TOML parsing and validation
- Environment variable validation for Supabase connection
- Directory/file creation with proper permissions
- Clear error messages for missing/invalid configs or env vars

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

### Data Storage (STO-001 to STO-010)
- Supabase PostgreSQL persistent storage
- UUID generation by database
- Persistence across sessions
- Database-indexed chronological sorting
- Read/unread status tracking
- Connection error handling

### Content Display (DSP-001 to DSP-012)
- Crossterm-based pagination
- Terminal-aware formatting
- Color support with NO_COLOR
- Proper Unicode handling
- Read/unread status indicators

### Content Interaction (INT-001 to INT-009)
- Open command with browser launching
- Automatic mark as read on open
- Mark read/unread commands
- Mark all as read functionality
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

### Security (SEC-001 to SEC-009)
- HTTPS enforcement
- Certificate validation
- Secret key protection (env vars only)
- Never store or log secret key
- Key masking in debug output
- Restricted URL schemes
- Documented security best practices

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
- **CLI**: Command parsing, help text, exit codes
- **Config**: TOML parsing, validation, error handling
- **Environment**: Variable validation, missing vars, key protection
- **Source**: Feed parsing with fixtures, network mocking, timeout testing
- **Fetcher**: Concurrent operations, partial failures, progress tracking
- **Storage**: Database operations with mocked client, deduplication
- **Display**: Truncation, Unicode, pagination calculations
- **Browser**: Command execution mocking, URL handling

### Mock Infrastructure
- **Network**: `mockito` or `wiremock` for HTTP responses
- **Database**: Mock Supabase client with fake credentials
- **Environment**: Mock environment variables for testing
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
- Advanced search with Supabase full-text search
- Real-time sync using Supabase Realtime subscriptions
- Multi-user support with Supabase Auth
- Additional commands (extend CLI enum)
- Web UI (using Supabase JS client)
- Advanced filtering (database queries)
- Analytics dashboard (database aggregations)
- Collaborative features (shared feeds, annotations)
