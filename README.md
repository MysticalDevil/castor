# Castor: Gemini Session Manager

`castor` is a secure, local session manager for the Gemini CLI, written in Rust. It provides a modern interface to inspect, manage, and clean up your local conversation history while prioritizing safety through a default dry-run policy and soft-delete mechanism.

## 🌟 Features

- **Project-Aware Scanning**: Automatically groups sessions by the project they originated from.
- **Modern TUI**: A rich Terminal User Interface powered by `ratatui` with three-pane navigation (Tree, File Status, Preview).
- **Rich Preview**: High-performance conversation preview with Markdown rendering and role-based coloring.
- **Deep Health Check**: Automatically identifies orphaned, corrupted, or potentially tampered (RISK) session files.
- **Safety First**: Implements a "Soft Delete" strategy by default, moving sessions to a trash folder with full audit logging.
- **High Performance**: Features asynchronous scanning and persistent metadata caching for instant loading of 1000+ sessions.
- **Customizable**: Supports multiple icon sets (NerdFont, Unicode, Emoji, ASCII) and popular TUI themes (TokyoNight, Gruvbox, OneDark, Catppuccin).

## 🚀 Installation

### Prerequisites
- **Rust Toolchain**: 1.85+ (supports Rust 2024 edition).
- **Just**: (Optional) For running project tasks.

### Build from source
```bash
git clone https://github.com/yourusername/castor.git
cd castor
cargo build --release
```

## 🛠 Usage

### CLI Commands
- `castor list`: List all detected sessions.
- `castor cat <ID>`: Render a specific conversation in your terminal.
- `castor prune --days 30`: Preview and cleanup sessions older than 30 days.
- `castor doctor`: Run a health check on your Gemini environment.
- `castor stats`: Show disk usage and session counts.

### TUI mode
```bash
castor tui
```
- **Navigation**: Use `j/k` or arrow keys to move through the tree.
- **Grouping**: Press `g` to toggle between Host and Month grouping.
- **Actions**: Press `d` to delete (moves to trash), `r` to reload.

## 📂 Project Structure

- `src/core/`: Domain logic (Session parsing, Scanner, Metadata Cache).
- `src/ops/`: Atomic business operations (Delete, Export, Grep, Doctor).
- `src/tui/`: TUI implementation and theme system.
- `src/audit/`: Structured audit logging.
- `src/utils/`: Shared utilities (Path formatting, Icons, Terminal rendering).

## 🧪 Development

Use `just` to run common tasks:
- `just test`: Run all unit and integration tests.
- `just check`: Run the full quality gate (fmt, clippy, test, coverage).
- `just test-tui`: Generate a rich test dataset and launch the TUI.

## 📄 License

This project is licensed under the **BSD 3-Clause License**. See the [LICENSE](LICENSE) file for details.
