# Clio MVP Requirements

This document defines the requirements for the Minimum Viable Product (MVP) of Clio, a command-line feed aggregator. All requirements are written using Easy Approach to Requirements Syntax (EARS) and focus exclusively on MVP functionality.

## 1. Configuration Management

### CFG-001: Configuration File Location
The system shall read configuration from a TOML file located at `~/.clio/config.toml`.
<!-- If the file doesn't exist, the system should provide clear guidance on creating it -->

### CFG-002: Configuration Directory Creation
When the configuration directory `~/.clio/` does not exist, the system shall create it with user-only permissions (700).

### CFG-003: Configuration File Permissions
When creating the configuration file, the system shall set file permissions to user-only access (600).

### CFG-004: Configuration File Format
The system shall parse configuration files in TOML format with the following structure:
- Array of sources under `[[sources.rss]]`
- Each source containing required fields: `name` (string) and `url` (string)
<!-- Supabase configuration handled via environment variables only -->

### CFG-005: Configuration Validation
When loading configuration, the system shall validate that:
- Each source has a non-empty `name` field
- Each source has a valid URL in the `url` field
- The URL uses HTTP or HTTPS protocol
<!-- Supabase validation happens separately via environment variables -->

### CFG-006: Missing Configuration Handling
When the configuration file does not exist, the system shall display an error message with:
- The expected configuration file path
- An example configuration snippet
- Instructions for creating the configuration file

### CFG-007: Invalid Configuration Handling
When the configuration file contains invalid TOML syntax, the system shall:
- Display a clear error message indicating the parsing error
- Show the line number and nature of the syntax error
- Exit with a non-zero status code

### CFG-008: Empty Configuration Handling
When the configuration file exists but contains no sources, the system shall display a warning message and continue operation.

### CFG-009: Environment Variable Requirements
The system shall require the following environment variables for database connection:
- `SUPABASE_URL`: The Supabase project URL (must be HTTPS)
- `SUPABASE_SECRET_KEY`: The secret key for full database access

### CFG-010: Missing Environment Variables
When required environment variables are not set, the system shall:
- Display a clear error message listing the missing variables
- Provide instructions for setting them
- Exit with a non-zero status code

### CFG-011: Environment Variable Validation
When validating environment variables, the system shall:
- Verify SUPABASE_URL is a valid HTTPS URL
- Verify SUPABASE_SECRET_KEY is not empty and starts with 'sb_secret_'
- Never log or display the secret key value

## 2. Source Management

### SRC-001: Source Abstraction
The system shall implement a trait-based abstraction for content sources to allow future extensibility.
<!-- This ensures the architecture can support additional source types post-MVP -->

### SRC-002: RSS Source Support
The system shall support RSS 2.0 feed sources.

### SRC-003: Atom Source Support
The system shall support Atom 1.0 feed sources.

### SRC-004: Source Identification
Each source shall be uniquely identified by its configured name within the application.

### SRC-005: Duplicate Source Names
When the configuration contains multiple sources with the same name, the system shall display an error and refuse to proceed.

### SRC-006: Source URL Validation
When validating source URLs, the system shall accept only HTTP and HTTPS protocols.

### SRC-007: Maximum Sources
The system shall support at least 100 configured sources.
<!-- Based on performance requirement of handling 100 sources within 30 seconds -->

## 3. Content Fetching

### FET-001: Pull Command
The system shall provide a `pull` command that fetches the latest content from all configured sources.

### FET-002: Parallel Fetching
When executing the pull command, the system shall fetch from multiple sources in parallel to improve performance.

### FET-003: Feed Parsing
When fetching RSS/Atom feeds, the system shall extract the following fields for each item:
- Title (required)
- Publication date (required if available in feed)
- Summary/description (optional)
- Link/URL (required)
- Source name (from configuration)

### FET-004: Network Timeout
When fetching a feed, if the request does not complete within 10 seconds, the system shall timeout and proceed with other sources.
<!-- Individual timeout ensures one slow source doesn't block others -->

### FET-005: HTTPS Preference
When making network requests, the system shall use HTTPS when available.

### FET-006: Network Error Handling
When a network error occurs while fetching a source, the system shall:
- Report the error with the source name
- Continue fetching other sources
- Not crash or exit

### FET-007: Malformed Feed Handling
When a feed contains malformed XML or invalid RSS/Atom structure, the system shall:
- Report an error message identifying the problematic source
- Skip the malformed source
- Continue processing other sources

### FET-008: Partial Feed Parsing
When a feed is partially malformed, the system shall extract and store all valid items that can be parsed successfully.

### FET-009: Missing Required Fields
When a feed item lacks a title or link, the system shall skip that item and continue processing other items.

### FET-010: Fetch Progress Feedback
While fetching feeds, the system shall display progress information including:
- Number of sources being fetched
- Current progress (e.g., "Fetching 5 of 20 sources...")

### FET-011: Fetch Completion Feedback
When the pull command completes, the system shall display:
- Total number of sources fetched successfully
- Total number of new items retrieved
- Number of sources that failed (if any)

### FET-012: Duplicate Detection
When fetching content, the system shall:
- Check existing items in the database before inserting
- Skip items with URLs that already exist (using unique constraint)
- Update the updated_at timestamp for existing items if content has changed
<!-- Database-level deduplication ensures consistency -->

### FET-013: HTTP Status Handling
When a feed returns an HTTP error status (4xx or 5xx), the system shall:
- Report the HTTP status code and source name
- Continue with other sources
- Include the source in the failed count

### FET-014: Redirect Handling
When a feed URL returns a redirect (3xx status), the system shall follow up to 5 redirects before failing.

### FET-015: Empty Feed Handling
When a feed contains no items, the system shall note this and continue normally.

## 4. Data Storage

### STO-001: Persistent Storage with Supabase
The system shall store all fetched items in a Supabase PostgreSQL database for persistence between sessions.
<!-- Using Supabase for cloud-based persistent storage -->

### STO-002: Item Data Structure
Each stored item shall contain:
- Id (UUID, primary key, generated)
- Source name (string, required)
- Title (string, required)
- Summary (string, optional)
- Link (string, required, unique constraint)
- Publication date (timestamp, optional)
- Is_read (boolean, default false)
- Created_at (timestamp, auto-generated)
- Updated_at (timestamp, auto-updated)

### STO-003: Database Connection
The system shall establish a connection to Supabase using:
- SUPABASE_URL environment variable for the project URL
- SUPABASE_SECRET_KEY environment variable for authentication
<!-- Secret key provides full access, bypassing RLS -->

### STO-004: Basic Database Operations
The system shall support basic database operations without optimization requirements in the MVP.
<!-- Memory and database efficiency optimizations can be added post-MVP -->

### STO-005: Item Identifier Persistence
The system shall use database-generated UUIDs as item identifiers that persist across sessions.
<!-- Enables reliable item references across sessions -->

### STO-006: Chronological Ordering
The system shall use database indexes on publication_date and created_at for efficient chronological sorting.

### STO-007: Deduplication
The system shall prevent duplicate items using a unique constraint on the link column in the database.

### STO-008: Read Status Tracking
The system shall track read/unread status for each item, persisting this state across sessions.

### STO-009: Database Schema Migration
The system shall check and create the required database schema on first run if tables don't exist.
<!-- Service role key allows schema modifications -->

### STO-010: Connection Error Handling
When the database connection fails, the system shall:
- Display a clear error message
- Suggest checking network and configuration
- Exit gracefully without data corruption

## 5. Content Display

### DSP-001: List Command
The system shall provide a `list` command that displays fetched items.

### DSP-002: Default Sort Order
When displaying items, the system shall show them in reverse chronological order (newest first).

### DSP-003: Items Without Dates
When an item lacks a publication date, the system shall display it after all dated items.

### DSP-004: Item Display Format
Each displayed item shall show:
- Item identifier (for selection)
- Title (truncated if necessary)
- Source name
- Publication date (formatted as relative time if within 7 days, otherwise as date)
- Summary (first line, truncated to terminal width)

### DSP-005: Terminal Width Respect
The system shall detect terminal width and truncate text to prevent line wrapping.

### DSP-006: Pagination
When displaying more items than fit on one screen, the system shall provide pagination with:
- Items per page based on terminal height
- Current page indicator
- Total pages indicator

### DSP-007: Pagination Navigation
While viewing paginated results, the system shall support:
- Next page (arrow down or 'j')
- Previous page (arrow up or 'k')
- First page (Home or 'g')
- Last page (End or 'G')
- Quit (q or Escape)

### DSP-008: Empty Feed Display
When no unread items are available to display, the system shall:
- Show a message indicating no unread items
- Offer option to show all items including read ones
- Suggest running the `pull` command if no items exist at all

### DSP-009: Color Support
When the terminal supports color, the system shall use colors to differentiate:
- Source names (dimmed)
- Dates (dimmed)
- Titles (normal/bright)
- Item identifiers (cyan or similar)

### DSP-010: Unicode Handling
The system shall correctly display Unicode characters in titles and summaries.

### DSP-011: Long Title Truncation
When a title exceeds 80% of terminal width, the system shall truncate it with an ellipsis (...).

## 6. Content Interaction

### INT-001: Open Command
The system shall provide an `open` command that accepts an item identifier as an argument.

### INT-002: Browser Launch
When the open command is executed with a valid item identifier, the system shall:
- Launch the item's URL in the system's default web browser
- Mark the item as read in the database
- Update the item's updated_at timestamp

### INT-003: Invalid Identifier Handling
When the open command is executed with an invalid identifier, the system shall display an error message indicating the item was not found.

### INT-004: Browser Launch Failure
When the system cannot launch the default browser, the system shall:
- Display an error message
- Show the URL for manual copying

### INT-005: Open Command Without Arguments
When the open command is executed without an item identifier, the system shall display an error message requesting an item identifier.

### INT-006: macOS Browser Integration
On macOS, the system shall use the `open` system command to launch URLs in the default browser.

### INT-007: Mark as Read Command
The system shall provide a `mark-read` command that accepts an item identifier to mark items as read without opening them.

### INT-008: Mark as Unread Command
The system shall provide a `mark-unread` command that accepts an item identifier to mark items as unread.

### INT-009: Mark All as Read
The system shall provide a `mark-all-read` command to mark all items as read in a single operation.

## 7. CLI Interface

### CLI-001: Command Structure
The system shall use subcommands for different operations: `pull`, `list`, `open`, `mark-read`, `mark-unread`, and `mark-all-read`.

### CLI-002: Help Flag
Each command shall support a `--help` flag that displays:
- Command description
- Available options and flags
- Usage examples

### CLI-003: Version Flag
The system shall support a `--version` flag that displays the application version.

### CLI-004: Exit Codes
The system shall use standard exit codes:
- 0 for successful operation
- 1 for general errors
- 2 for command line usage errors

### CLI-005: Argument Parsing Errors
When invalid arguments are provided, the system shall:
- Display a clear error message
- Show the correct usage
- Exit with status code 2

### CLI-006: Global Help
When run without any command or with `--help`, the system shall display:
- Application description
- List of available commands
- Basic usage examples

### CLI-007: Command Aliases
The system shall not provide command aliases in the MVP.
<!-- Keeping CLI simple for MVP -->

### CLI-008: Quiet Flag
The system shall support a `--quiet` flag that suppresses all non-error output.

## 8. Error Handling

### ERR-001: Unrecoverable Error Display
When an unrecoverable error occurs, the system shall:
- Display a clear error message to stderr
- Include context about what operation failed
- Exit with a non-zero status code

### ERR-002: Recoverable Error Handling
When a recoverable error occurs, the system shall:
- Continue operation
- Include the error in summary statistics if applicable

### ERR-003: Network Error Messages
Network error messages shall include:
- The source name that failed
- The type of network error (timeout, connection refused, DNS failure, etc.)

### ERR-004: File System Error Messages
File system error messages shall include:
- The file path involved
- The operation that failed (read, write, create directory)
- The system error if available

### ERR-005: Panic Prevention
The system shall not panic on any user input or external data.
<!-- Ensuring robustness and user-friendly error handling -->

### ERR-006: Error Message Formatting
Error messages shall follow the format: "Error: [context]: [specific error]"

## 9. Performance Requirements

### PRF-001: Fetch Operation Timeout
The pull command shall complete within 30 seconds for up to 100 sources under normal network conditions.

### PRF-002: CLI Responsiveness
Interactive CLI commands shall respond to user input within 100 milliseconds.

### PRF-003: Memory Usage
The system shall operate within reasonable memory constraints for a CLI application.
<!-- Specific memory optimization targets can be defined post-MVP -->

### PRF-004: Startup Time
The application shall start and be ready for commands within 1 second.

### PRF-005: List Command Performance
The list command shall display the first page of results within 500 milliseconds when items are already in memory.

## 10. Security Requirements

### SEC-001: HTTPS Usage
The system shall use HTTPS for all network requests when the source URL specifies HTTPS.

### SEC-002: Certificate Validation
The system shall validate SSL/TLS certificates for HTTPS connections.

### SEC-003: No Credential Storage
The system shall not store any authentication credentials in the MVP.
<!-- RSS/Atom feeds are typically public -->

### SEC-004: No Credential Exposure
The system shall not expose any URLs containing authentication tokens or credentials in output.

### SEC-005: Configuration File Permissions
The system shall ensure configuration files are readable only by the owner (permissions 600).

### SEC-006: No Code Execution
The system shall not execute any code from fetched content.

### SEC-007: URL Scheme Restriction
The system shall only accept HTTP and HTTPS URL schemes, rejecting file://, ftp://, and other schemes.

### SEC-008: Secret Key Protection
The system shall:
- Only accept the secret key via environment variable
- Never log, display, or echo the secret key value
- Never store the secret key in any file
- Mask the key in any debug output (show only first/last 4 chars)
- Exit immediately if the key is attempted to be logged

### SEC-009: Environment Variable Security
The system shall:
- Document that SUPABASE_SECRET_KEY must be kept secret
- Recommend using secure secret management tools (e.g., 1Password CLI, macOS Keychain)
- Never accept secret key via command-line arguments
- Validate environment variables are set before any database operations

## 11. Platform Requirements

### PLT-001: macOS Support
The system shall compile and run on macOS (version 12.0 or later).

### PLT-002: Rust Version
The system shall compile with stable Rust (version 1.70 or later).

### PLT-003: Platform Paths
The system shall use platform-appropriate paths for configuration (~/.clio/ on macOS).

## 12. Data Validation

### VAL-001: HTML Entity Decoding
The system shall decode HTML entities in titles and summaries (e.g., &amp; to &, &lt; to <).

### VAL-002: Whitespace Normalization
The system shall normalize excessive whitespace in titles and summaries to single spaces.

### VAL-003: Date Parsing
The system shall parse dates in RFC 822, RFC 3339, and ISO 8601 formats commonly used in RSS/Atom feeds.

### VAL-004: Invalid Date Handling
When a date cannot be parsed, the system shall treat the item as having no date rather than failing.

### VAL-005: URL Validation
The system shall validate that item URLs are well-formed before storing them.

### VAL-006: Empty Title Handling
When an item has an empty title after trimming whitespace, the system shall skip that item.

## 13. Testing Requirements

### TST-001: Unit Test Coverage
The system shall maintain at least 80% unit test coverage for all business logic modules.

### TST-002: Integration Tests
The system shall include integration tests for:
- Configuration loading and validation
- Feed fetching and parsing
- Command execution workflows
- Error handling paths

### TST-003: Mock Infrastructure
The system shall provide mock implementations for:
- Network requests (mock HTTP responses)
- Database operations (mock Supabase client)
- Environment variables (test-specific values)
- File system operations (in-memory config)
- System time (deterministic date handling)
- Browser launching (capture open commands)

### TST-004: Test Organization
Tests shall be organized as:
- Unit tests in the same file as the code being tested (in `#[cfg(test)]` modules)
- Integration tests in the `tests/` directory
- Database tests using mocked environment variables and client
- Test fixtures in `tests/fixtures/` for sample feeds and configs

### TST-005: Feed Parser Testing
The system shall include test fixtures for:
- Valid RSS 2.0 feeds
- Valid Atom 1.0 feeds
- Malformed XML
- Feeds with missing required fields
- Feeds with various date formats
- Feeds with HTML entities
- Unicode content

### TST-006: Error Scenario Testing
The system shall include tests for:
- Network timeouts
- HTTP error responses (404, 500, etc.)
- Invalid configuration files
- Missing environment variables
- Invalid environment variable values
- Malformed feed data
- File system permission errors
- Database connection failures

### TST-007: Performance Testing
The system shall include performance tests to verify:
- Fetch operations complete within timeout
- CLI responsiveness requirements
<!-- Memory usage testing can be added when optimization targets are defined -->

### TST-008: CLI Testing
The system shall include tests for:
- Command parsing and validation
- Help text generation
- Exit codes for various scenarios
- Flag combinations

### TST-009: Display Testing
The system shall include tests for:
- Text truncation logic
- Unicode width calculations
- Date formatting
- Pagination calculations

### TST-010: Concurrent Operation Testing
The system shall include tests for:
- Parallel feed fetching
- Race conditions in deduplication

### TST-011: Test Execution
All tests shall be executable via `cargo test` and shall:
- Run without network access (except specific integration tests)
- Complete within 30 seconds total
- Provide clear failure messages

### TST-012: Documentation Tests
The system shall include documentation tests for:
- All public API examples in doc comments
- README code examples
- Configuration examples

### TST-013: Property-Based Testing
The system shall use property-based testing for:
- Date parsing with various formats
- Text truncation with various Unicode inputs
- URL validation

### TST-014: Regression Testing
The system shall include regression tests for any bugs fixed post-release to prevent reoccurrence.

### TST-015: Test Data Management
Test data shall be:
- Stored in version control for reproducibility
- Minimal but representative
- Documented with source and purpose

### TST-016: Environment Variable Testing
The system shall include tests that:
- Mock environment variables for all test scenarios
- Verify proper error handling when variables are missing
- Ensure secret key is never exposed in logs or errors
- Test with invalid URLs and empty values
