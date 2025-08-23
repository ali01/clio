Simple feed aggregator written in Rust.

### Problem Statement

Today content is fragmented across dozens of platforms and formats. Users have to constantly switch between different apps, websites, and interfaces to keep up with content they follow – YouTube, Twitter/X, Substack, newsletters, RSS readers,  podcast apps, and individual blogs. There is no unified way to aggregate, prioritize content from all these diverse sources in a single, consistent interface.

### Vision

Create a simple, extensible command-line tool that serves as a unified inbox for all digital content. The MVP will focus on aggregating content from multiple source types into a single, chronological feed that users can scroll through on the command line. Future versions will allow the user to filter the feed; mark items as read; and might support web and mobile interfaces, but the core will remain a fast, reliable, and hackable content aggregation engine built in Rust.

### Target Users & Personas

**Primary Persona: Technical Power User**
- Developers, engineers, and technical users comfortable with command-line tools
- Follows 50+ content sources across multiple platforms
- Values efficiency, keyboard-driven interfaces, and scriptability
- Wants control over their data and the ability to extend/customize their tools
- Frustrated by algorithm-driven feeds and wants chronological, unfiltered access to content

### Scope
#### MVP Must-Haves
- Support for RSS/Atom feeds
- Extensible architecture to allow for other source types
- Configuration file (TOML) to define sources
- Basic fetch command to pull latest content from all sources
- Command-line interface to view feed headlines in chronological order
- Display title, source, date, and summary for each item
- Simple pagination through feed items
- Ability to open full content for an item on a browser

#### Post-MVP Features
- Persistent storage (SQLite database) to cache items between sessions
- Additional source types (YouTube, Twitter/X, Substack, podcasts, arbitrary websites)
- Mark items as read/unread
- Search functionality
- Superhuman-style splits that can be used to filter the feed
- Scheduling/automatic background fetching
- Web UI
- Mobile app
- Multi-device sync
- Content archiving
- Full-text content extraction

#### Out of Scope
- Social features (sharing, commenting, liking)
- Content creation or publishing
- Email client functionality
- Algorithm-based recommendations
- Paid subscription management
- Analytics or tracking
- Support for Linux or Windows

### Functional Requirements

**Configuration Management**
- System shall read source configurations from a TOML file
- Each source entry must specify type (RSS/Atom) and URL

**Content Fetching**
- `pull` command shall retrieve latest items from all configured sources
- System shall parse RSS/Atom feeds and extract title, date, summary, and link
- Duplicate items (same URL) shall not be stored multiple times
- System shall handle feed errors gracefully without stopping other fetches

**Data Storage**
- Fetched items shall be stored in memory during runtime
- Each item shall include: source, title, summary, link, publication date
- Data is not persisted between sessions (fresh fetch required each run)

**Content Display**
- `list` command shall display items in reverse chronological order (newest first)
- Each item shall show: title, source name, publication date, and summary (truncated to terminal width)
- System shall support pagination with keyboard navigation

**Content Interaction**
- `open [item-id]` command shall launch the item's URL in the default browser
- System shall provide clear item identifiers for selection

**CLI Interface**
- All commands shall provide helpful error messages
- `--help` flag shall display usage information for each command
- System shall provide feedback during long-running operations (fetching)

**Logging System**
- System shall support two log levels: normal and debug
- Log output shall be written to a file in the configuration directory (~/.clio/clio.log)
- Normal level shall log important operations and errors
- Debug level shall log detailed operations, including network requests and parsing details
- `--debug` flag shall enable debug logging level
- Log entries shall include timestamp, level, and component/module name

### Non-Functional Requirements

**Performance**
- Fetch operations should complete within 30 seconds for up to 100 sources
- CLI should respond to user input within 100ms
- Memory usage should stay under 100MB during normal operation

**Reliability**
- System should handle network failures gracefully with appropriate retry logic
- Application should not crash due to malformed feed data

**Usability**
- Commands should follow Unix conventions (short and long flags)
- Error messages should be clear and actionable
- Terminal output should respect terminal width
- Help text should include examples

**Portability**
- Must compile and run on macOS
- Configuration and data files should use platform-appropriate directories

**Extensibility**
- Architecture should allow easy addition of new source types
- Core fetching logic should be decoupled from source-specific parsers
- Data structures should accommodate future fields without breaking changes

**Security**
- HTTPS should be used for all network requests when available
- No credentials or sensitive data should be logged
- Configuration files should have appropriate file permissions (user-only access)

### Architecture
#### Components

**CLI Layer**
- Command parser using clap or similar crate
- Subcommands: `pull`, `list`, `open`
- Handles user input validation and formatting output

**Source Manager**
- Trait-based source abstraction (`Source` trait)
- RSS/Atom feed parser implementation
- Parallel fetching with async/await (tokio)
- Error handling and retry logic

**Data Layer**
- In-memory storage using `Vec<Item>` or similar structure
- Simple deduplication using HashSet for URLs
- Models: Item, Source

**Configuration Loader**
- TOML parser using toml crate
- Config file discovery (~/.clio/config.toml)
- Config validation and defaults

**Display Engine**
- Terminal UI components (possibly using ratatui for pagination)
- Item formatting and truncation
- Color output support

#### Configuration

Example config.toml structure:
```toml
[[sources.rss]]
name = "Hacker News"
url = "https://news.ycombinator.com/rss"

[[sources.rss]]
name = "Julia Evans"
url = "https://jvns.ca/atom.xml"
```
