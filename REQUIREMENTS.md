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

### CFG-005: Configuration Validation
When loading configuration, the system shall validate that:
- Each source has a non-empty `name` field
- Each source has a valid URL in the `url` field
- The URL uses HTTP or HTTPS protocol

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
- Log the error with the source name
- Continue fetching other sources
- Not crash or exit

### FET-007: Malformed Feed Handling
When a feed contains malformed XML or invalid RSS/Atom structure, the system shall:
- Log an error message identifying the problematic source
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
When fetching content, the system shall not store duplicate items with the same URL.
<!-- Deduplication based on URL ensures the same article isn't shown multiple times -->

### FET-013: HTTP Status Handling
When a feed returns an HTTP error status (4xx or 5xx), the system shall:
- Log the HTTP status code and source name
- Continue with other sources
- Include the source in the failed count

### FET-014: Redirect Handling
When a feed URL returns a redirect (3xx status), the system shall follow up to 5 redirects before failing.

### FET-015: Empty Feed Handling
When a feed contains no items, the system shall log this as informational and continue normally.

## 4. Data Storage

### STO-001: In-Memory Storage
The system shall store all fetched items in memory during runtime.
<!-- MVP does not include persistent storage between sessions -->

### STO-002: Item Data Structure
Each stored item shall contain:
- Source name (string, required)
- Title (string, required)
- Summary (string, optional)
- Link (string, required)
- Publication date (timestamp, optional)
- Unique identifier (generated, required)

### STO-003: Session Scope
When the application exits, all stored items shall be discarded.
<!-- No persistence in MVP -->

### STO-004: Memory Limit
The system shall not exceed 100MB of memory usage during normal operation with up to 100 sources.

### STO-005: Item Identifier Generation
The system shall generate a unique identifier for each item that remains consistent within a session.
<!-- Used for the `open` command to reference specific items -->

### STO-006: Chronological Ordering
The system shall maintain items in a structure that allows efficient chronological sorting.

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
When no items are available to display, the system shall show a message indicating the feed is empty and suggest running the `pull` command.

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
When the open command is executed with a valid item identifier, the system shall launch the item's URL in the system's default web browser.

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

## 7. CLI Interface

### CLI-001: Command Structure
The system shall use subcommands for different operations: `pull`, `list`, and `open`.

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

### CLI-008: Verbose Flag
The system shall support a `--verbose` flag for detailed logging output.

### CLI-009: Quiet Flag
The system shall support a `--quiet` flag that suppresses all non-error output.

## 8. Error Handling

### ERR-001: Unrecoverable Error Display
When an unrecoverable error occurs, the system shall:
- Display a clear error message to stderr
- Include context about what operation failed
- Exit with a non-zero status code

### ERR-002: Recoverable Error Logging
When a recoverable error occurs, the system shall:
- Log the error with context
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

### PRF-003: Memory Usage Limit
The system shall not exceed 100MB of memory usage during normal operation.

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

### SEC-004: No Credential Logging
The system shall not log any URLs containing authentication tokens or credentials.

### SEC-005: Configuration File Permissions
The system shall ensure configuration files are readable only by the owner (permissions 600).

### SEC-006: No Code Execution
The system shall not execute any code from fetched content.

### SEC-007: URL Scheme Restriction
The system shall only accept HTTP and HTTPS URL schemes, rejecting file://, ftp://, and other schemes.

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

## 13. Logging System

### LOG-001: Log Levels
The system shall support two log levels:
- Normal: logs important operations, errors, and warnings
- Debug: logs detailed operations including network requests, parsing details, and internal state changes

### LOG-002: Log File Location
The system shall write log output to a file located at `~/.clio/clio.log`.

### LOG-003: Debug Flag
The system shall support a `--debug` flag that enables debug-level logging for the current execution.

### LOG-004: Default Log Level
When the `--debug` flag is not specified, the system shall use normal log level.

### LOG-005: Log Entry Format
Each log entry shall include:
- ISO 8601 timestamp with millisecond precision
- Log level (INFO, DEBUG, WARN, ERROR)
- Component or module name
- Log message

### LOG-006: Log File Creation
When the log file does not exist, the system shall create it with user-only permissions (600).

### LOG-007: Normal Level Logging
At normal log level, the system shall log:
- Application startup and shutdown
- Configuration loading success/failure
- Pull command start and completion with statistics
- Network errors and feed parsing errors
- Fatal errors that cause application exit

### LOG-008: Debug Level Logging
At debug log level, the system shall log everything from normal level plus:
- Individual feed fetch start/completion
- HTTP request/response details (excluding sensitive data)
- Feed parsing details
- Item deduplication actions
- Configuration validation details
- Browser launch commands

### LOG-009: Concurrent Logging
When multiple operations write to the log concurrently, the system shall ensure log entries are not interleaved or corrupted.

### LOG-010: Log Write Failures
When the system cannot write to the log file, it shall:
- Continue operation without logging
- Display a warning to stderr on first failure only
- Not crash or exit due to logging failures

### LOG-011: Sensitive Data Exclusion
The system shall not log:
- Full URLs containing authentication tokens or credentials
- Any user credentials if encountered
- Personal data from feed content

### LOG-012: Performance Impact
Logging operations shall not add more than 5% overhead to normal operations.

### LOG-013: Log Buffer Flushing
The system shall flush log buffers:
- After each error or warning message
- When the application exits
- At least every 5 seconds during normal operation

## 14. Testing Requirements

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
- File system operations (in-memory config)
- System time (deterministic date handling)
- Browser launching (capture open commands)

### TST-004: Test Organization
Tests shall be organized as:
- Unit tests in the same file as the code being tested (in `#[cfg(test)]` modules)
- Integration tests in the `tests/` directory
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
- Malformed feed data
- File system permission errors
- Missing dependencies

### TST-007: Performance Testing
The system shall include performance tests to verify:
- Memory usage stays under 100MB with 100 sources
- Fetch operations complete within timeout
- CLI responsiveness requirements

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
- Concurrent logging
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
