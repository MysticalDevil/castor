# Configuration Guide

Castor uses a JSON configuration file located at `~/.config/castor/config.json` (on Linux).

## Default Configuration

```json
{
  "gemini_sessions_path": "~/.gemini/tmp",
  "trash_path": "~/.local/share/castor/trash",
  "audit_path": "~/.local/share/castor/audit",
  "cache_path": "~/.cache/castor/metadata.json",
  "dry_run_by_default": true,
  "icon_set": "NerdFont",
  "theme": "TokyoNight",
  "preview": {
    "head_bytes": 524288,
    "tail_bytes": 2097152,
    "small_full_parse_bytes": 2097152,
    "deep_preview_max_bytes": 67108864,
    "deep_preview_char_budget": 120000
  }
}
```

## Options

### `icon_set`
Determines the icons used in the TUI.
- `NerdFont`: Requires a NerdFont compatible terminal.
- `Unicode`: Standard symbols (●, 📁).
- `Emoji`: Expressive icons (📂, 💬).
- `Ascii`: Pure text fallback ([P], [S]).

### `theme`
Sets the TUI color scheme.
- `TokyoNight` (Default)
- `Gruvbox`
- `OneDark`
- `Catppuccin`
- `DefaultDark`

You can also provide a **Custom Theme** by passing an object:
```json
"theme": {
  "border": "Blue",
  "title": "Cyan",
  "selection_bg": "DarkGray",
  "selection_fg": "Yellow",
  "folder": "Cyan",
  "user_msg": "Blue",
  "gemini_msg": "Green",
  "key_hint": "Magenta",
  "key_desc": "DarkGray"
}
```

### `dry_run_by_default`
If set to `true`, all `delete` and `prune` operations will require an explicit `--confirm` flag and `--dry-run false` to perform physical changes.

### `cache_path`
Path to the metadata cache file (not a directory).

### `preview`
Controls preview behavior for large sessions.
- `head_bytes`: quick preview scan window from the file head.
- `tail_bytes`: fallback quick preview scan window from the file tail.
- `small_full_parse_bytes`: full parse threshold for small files when quick scan misses.
- `deep_preview_max_bytes`: hard cap for deep preview in TUI.
- `deep_preview_char_budget`: max rendered characters for deep preview.
