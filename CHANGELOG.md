# Changelog

All notable changes to this project will be documented in this file.

## [0.1.0] - 2026-03-08

### Added
- Tree-mode TUI with keyboard navigation, grouping, and fold/unfold controls.
- Metadata cache plus streaming session scan for large datasets.
- Background quick/deep preview pipeline for large session files.
- CLI operations for list/export/search/stats/doctor/prune/restore/delete.
- CI workflow with test run and coverage gate.
- Unit/integration/performance/stress test suites.

### Changed
- TUI large-file preview now supports bounded deep preview instead of fully skipping content.
- TUI event and render paths were hardened to avoid selection/index instability under heavy updates.
- Coverage gate baseline was raised and stabilized for release quality.

### Notes
- This is the first tagged stable release.

