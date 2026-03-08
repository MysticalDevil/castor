# Configuration Guide

Castor uses a JSON configuration file located at `~/.config/castor/config.json` (on Linux).

## Default Configuration

```json
{
  "gemini_sessions_path": "~/.gemini/tmp",
  "trash_path": "~/.local/share/castor/trash",
  "audit_path": "~/.local/share/castor/audit",
  "cache_path": "~/.cache/castor",
  "dry_run_by_default": true,
  "icon_set": "NerdFont",
  "theme": "TokyoNight"
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
