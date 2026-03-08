# Changelog

All notable changes to this project are documented in this file.

## [0.1.1] - 2026-03-08

### Added
- `list` pagination support via `--page-size` in both normal and grouped views.
- CLI parsing tests for `restore --dry-run` defaults and override behavior.
- Additional table/pagination rendering tests.

### Changed
- CLI `list` readability improvements:
  - `ID` now uses true short session ID (`display_id`).
  - `Project` now prefers resolved host path display.
  - Better column widths/alignment and human-readable size units.
- `restore` dry-run behavior aligned with `delete`/`prune`:
  - Default is now dry-run (`true`).
  - Explicit execution requires `--dry-run false`.
- `--verbose` now initializes runtime logging.
- `cat`/`export`/`delete` now return non-zero exit on missing session.
- Host path formatting improved for non-home absolute paths.
- Minor internal refactors to satisfy strict clippy gate.

### Quality
- `cargo build --release` passed.
- `just check` passed (fmt, clippy `-D warnings`, tests, coverage gate).

## [0.1.0] - 2026-03-08

### Added
- Initial stable release.
- Modern TUI and CLI workflows for Gemini session management.
