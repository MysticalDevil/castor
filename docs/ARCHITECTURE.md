# Castor Architecture

This document describes the high-level architecture of Castor.

## Design Principles

1. **Safety First**: Destructive operations like `delete` or `prune` are "soft" by default (moving files to a trash directory) and always support dry-run previews.
2. **Performance**: To handle large numbers of session files, Castor uses asynchronous scanning and a persistent metadata cache.
3. **Deep Validation**: Session health is assessed not just by file existence, but through structural JSON parsing, temporal anomaly detection, and statistical analysis.
4. **Decoupling**: Business logic is separated into `ops` modules, while data models reside in `core`. The CLI and TUI are thin layers on top of these.

## Module Breakdown

### `core` (Domain Logic)
- **`Session`**: The central data model representing a Gemini conversation. Handles lazy loading of content and health calculations.
- **`Scanner`**: Recursively traverses the Gemini tmp directory to discover sessions and project roots.
- **`Registry`**: Manages the collection of sessions in memory and coordinates with the cache.
- **`MetadataCache`**: A JSON-based persistent store that tracks file modification times (`mtime`) to avoid redundant deep parsing.

### `ops` (Atomic Operations)
- **`Executor`**: The "engine" that performs physical file moves, deletes, and restores.
- **`Doctor`**: Generates health reports for the environment.
- **`Export`**: Handles conversion of JSON sessions to Markdown, including role-based message merging.
- **`Grep`**: Implements content-based searching across sessions.
- **`Prune`**: Implements session cleanup strategies based on age.

### `tui` (Terminal Interface)
- **`App`**: A state machine managing navigation, grouping modes, and async event handling.
- **`UI`**: Pure rendering logic using `ratatui`.
- **`Theme`**: Customizable color schemes.

## Performance Strategies

- **Asynchronous Preview**: In the TUI, when a user selects a session, the file is read and parsed in a background thread to prevent UI stuttering.
- **Debouncing**: Rapidly scrolling through sessions won't trigger hundreds of IO requests; a short delay is required before a preview is loaded.
- **Quick + Deep Preview**: Quick preview first tries structured JSON extraction from session head messages, then falls back to head/tail window scan. Deep preview (`p`) is explicitly triggered and bounded by file-size and character budgets.
- **Limited Parsing**: Both quick and deep paths enforce message/character limits to keep UI latency bounded for multi-megabyte sessions.

## Security Model

Castor identifies three levels of concern:
- **WARN**: The session is orphaned (its original project directory is missing).
- **ERROR**: The session file is corrupted (unparsable JSON or 0-byte file).
- **RISK**: High-risk anomalies, such as a session ID claiming a future date or an abnormally large file size (potential injection).
