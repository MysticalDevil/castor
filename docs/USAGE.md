# Usage Guide

This guide covers common Castor workflows.

## Build and Run

```bash
cargo build --release
./target/release/castor --help
```

## Run After Installation

If installed via `cargo install`, use:

```bash
castor --help
castor list
castor tui
```

## Core Commands

### List sessions

```bash
./target/release/castor list
```

### Inspect one session

```bash
./target/release/castor cat <SESSION_ID>
```

### Search session content

```bash
./target/release/castor grep "keyword"
```

### Show health/system status

```bash
./target/release/castor doctor
```

### Show usage statistics

```bash
./target/release/castor stats
```

## Safe Cleanup Workflows

Castor uses soft-delete by default. Prefer dry-run first.

### Preview prune result (recommended)

```bash
./target/release/castor prune --days 30 --dry-run
```

### Execute prune physically

```bash
./target/release/castor prune --days 30 --dry-run false --confirm
```

### Restore from trash

```bash
./target/release/castor restore <SESSION_ID>
```

`restore` defaults to dry-run. To perform actual restore:

```bash
./target/release/castor restore <SESSION_ID> --dry-run false
```

## TUI Workflows

### Launch TUI

```bash
./target/release/castor tui
```

### Keybindings

- `j/k` or `Up/Down`: move selection
- `h/l` or `Left/Right`: fold/unfold group
- `Space`: toggle fold
- `g`: switch grouping mode
- `d`: delete selected session (soft-delete)
- `r`: reload session tree
- `p`: deep preview selected session
- `Enter`: open selected item
- `q`: quit

## Large-File Preview Behavior

For very large session files, preview uses bounded windows and budgets to keep TUI responsive.

- Quick preview reads from head window first, then tail window fallback.
- Deep preview (`p`) has max file-size and char-budget limits.
- If a file is too large for deep preview settings, content is truncated by design.

Tune these values in `preview` config fields:

- `head_bytes`
- `tail_bytes`
- `small_full_parse_bytes`
- `deep_preview_max_bytes`
- `deep_preview_char_budget`

See [CONFIGURATION.md](./CONFIGURATION.md) for exact meanings.
