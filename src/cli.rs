use clap::builder::styling::{AnsiColor, Styles};
use clap::{Parser, Subcommand};
use std::path::PathBuf;

// Define a professional Rust CLI style palette
const STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().bold())
    .usage(AnsiColor::Yellow.on_default().bold())
    .literal(AnsiColor::Green.on_default().bold())
    .placeholder(AnsiColor::Cyan.on_default());

#[derive(Parser)]
#[command(name = "castor", version, author, styles = STYLES)]
#[command(about = "Gemini Session Manager - Local session management for Gemini CLI", long_about = None)]
pub struct Cli {
    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Specify a custom config path
    #[arg(short, long, global = true, value_name = "FILE")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start the terminal user interface
    Tui,

    /// List all detected Gemini sessions
    List {
        /// Format output as JSON
        #[arg(long)]
        json: bool,

        /// Group sessions by host project
        #[arg(short, long)]
        group: bool,

        /// Number of sessions per page (0 for no paging)
        #[arg(short, long, default_value_t = 0)]
        page_size: usize,
    },

    /// Show the content of a specific session
    Cat {
        /// The ID or path of the session to display
        id: String,

        /// Display raw JSON instead of pretty-printed content
        #[arg(long)]
        raw: bool,
    },

    /// Remove old sessions based on a strategy
    Prune {
        /// Remove sessions older than N days
        #[arg(short, long, default_value_t = 30)]
        days: u64,

        /// Perform a hard delete (bypass trash)
        #[arg(long)]
        hard: bool,

        /// Dry-run mode: show what would be pruned
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,

        /// Confirm the operation
        #[arg(long)]
        confirm: bool,
    },

    /// Delete a specific session
    Delete {
        /// The ID or path of the session to delete
        id: String,

        /// Perform a hard delete (bypass trash)
        #[arg(long)]
        hard: bool,

        /// Dry-run mode: show what would be done without actually deleting
        #[arg(long, default_value_t = true, action = clap::ArgAction::Set)]
        dry_run: bool,

        /// Confirm the operation (required for actual deletion)
        #[arg(long)]
        confirm: bool,
    },

    /// Restore a session from the trash
    Restore {
        /// The ID of the session to restore
        id: String,

        /// Dry-run mode: show what would be restored
        #[arg(long, default_value_t = false)]
        dry_run: bool,
    },

    /// Show audit history
    History {
        /// Limit the number of entries
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },

    /// Check the health of the Gemini environment and configuration
    Doctor,
}
