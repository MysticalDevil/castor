# Troubleshooting

This page covers common issues and practical fixes.

## TUI is laggy with many files

Symptoms:
- Scroll/input delay in tree pane.
- Preview refresh feels behind selection.

Checks and fixes:
1. Use release build: `cargo build --release` and run `./target/release/castor tui`.
2. Keep metadata cache path on local fast storage (`cache_path` in config).
3. Reduce preview load by tuning `preview` values:
   - Lower `head_bytes` and `tail_bytes`.
   - Lower `deep_preview_char_budget`.
4. Trigger deep preview only when needed (`p`).

## Large files do not show full preview

Expected behavior:
- Castor intentionally applies size and character budgets to protect TUI responsiveness.

If you need more content:
1. Increase `deep_preview_max_bytes`.
2. Increase `deep_preview_char_budget`.
3. Re-run TUI and press `p` for deep preview.

If still insufficient:
- Use `castor cat <SESSION_ID>` for non-TUI terminal rendering.

## Preview is empty or shows fallback text

Possible reasons:
- Session JSON is malformed.
- File is truncated/corrupted.
- File exceeds configured limits.

What to do:
1. Run `castor doctor`.
2. Open with `castor cat <SESSION_ID>`.
3. Check session health/status in TUI right pane.

## Icons look broken

Cause:
- Terminal font does not support current icon set.

Fix:
1. Set `icon_set` to `Unicode` or `Ascii` in config.
2. Or install and enable a Nerd Font and keep `NerdFont`.

## Theme or colors look wrong

Checks:
1. Confirm terminal supports truecolor.
2. Switch to another built-in theme in config (`Gruvbox`, `OneDark`, `Catppuccin`, `DefaultDark`).
3. Remove custom theme overrides and retry.

## Delete/prune does not physically remove files

Cause:
- Dry-run or safety confirmation not satisfied.

Fix:
```bash
./target/release/castor prune --days 30 --dry-run false --confirm
```

Notes:
- Soft-delete moves data to trash path. Use restore if needed.

## Config changes seem ignored

Checks:
1. Verify config file path is correct.
2. Validate JSON syntax.
3. Restart Castor process after changing config.

Default Linux path:
- `~/.config/castor/config.json`

