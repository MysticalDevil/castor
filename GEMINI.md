# Castor: Gemini Session Manager

Castor is a local session manager for the Gemini CLI, written in Rust. It provides a secure, transparent, and modern way to manage, inspect, and prune your local Gemini conversation history.

## Project Overview

- **Core Functionality**: Recursive scanning of Gemini session files (`~/.gemini/tmp/`), project-aware session grouping, and automated cleanup (pruning).
- **Safety First**: Implements a "Soft Delete" mechanism (moving to trash) and a default "Dry-run" policy for destructive operations. Every action is recorded in an atomic audit log.
- **Modern Interface**: Offers both a rich CLI with colored tables and a Terminal User Interface (TUI) powered by `ratatui`.
- **High Technical Standards**: Built with Rust 2024 Edition, 100% Safe Rust (no `unsafe` blocks), and robust Unicode support for CJK character alignment.

## Building and Running

### Prerequisites
- Rust Toolchain (1.85+ for 2024 edition support)
- Python 3 (optional, for generating test data)

### Key Commands (using `just`)
- **List Tasks**: `just`
- **Build**: `just build`
- **Run TUI (Real)**: `just tui`
- **Run TUI (Test Data)**: `just test-tui`
- **List Sessions**: `just list`
- **Test**: `just test`
- **Coverage**: `just coverage`
- **Lint/Check**: `just check`

### Cargo Fallbacks
- **Build**: `cargo build`
...

### CLI Usage Examples
- `castor list --group`: List sessions grouped by host project.
- `castor cat <ID>`: Render conversation history with role-based colors.
- `castor prune --days 30`: Preview and cleanup sessions older than 30 days.
- `castor delete <ID> --confirm --dry-run false`: Perform a real soft delete.

## Development Conventions

### Architecture
- `src/core/`: Domain logic (Session parsing, Scanner, Registry).
- `src/ops/`: Atomic operations (Executor for Delete/Restore).
- `src/audit/`: Audit logging and history tracking.
- `src/tui/`: Ratatui-based terminal UI implementation.
- `src/utils/`: Shared utilities (Path formatting, terminal alignment).
- `scripts/`: All utility and helper scripts (Python, Bash, etc.). **Do not place scripts in the root directory.**

### Coding Style
- **100% Safe Rust**: The use of `unsafe` blocks is strictly prohibited. All logic must be implemented using Safe Rust to leverage the compiler's full safety guarantees.
- **Latest Dependencies**: ALWAYS use the latest stable versions of all dependencies. When introducing new dependencies, first query for the most recent stable release via `cargo search`.
- **No Deprecated APIs**: The use of deprecated APIs is strictly prohibited. Always favor the latest recommended methods (e.g., use `frame.area()` instead of `frame.size()` in `ratatui`).
- **No Clippy Allows**: The use of `#[allow(clippy::...)]` attributes is strictly prohibited. All code must be refactored to satisfy Clippy lints or remain idiomatic.
- **English Comments**: All code documentation and inline comments must be in English.
- **Dependency Injection**: Use dependency injection for system-level mocks (like HOME path) in tests instead of modifying global state via `unsafe` functions.
- **Visual Integrity**: Use `unicode-width` for all table cell calculations to ensure alignment with CJK characters.
- **Error Handling**: Use the custom `CastorError` type defined in `src/error.rs`.

### Testing Practices
- **Isolation**: Always use `tempfile` for file system tests to avoid touching real Gemini data.
- **Coverage**: Aim for high coverage in `core` and `utils` modules. **A minimum total coverage of 40% is required for CI to pass.**
- **Quality Gate**: Use `just check` to run formatting, linting, and coverage verification in one go.
- **Verification**: ALWAYS run `just check` after any code modification to ensure the build is successful and all tests pass. **Changes are not considered complete until verified.**
- **Simulated Data**: Use `scripts/generate_test_data.py` to recreate complex scenarios for manual TUI/CLI verification.
