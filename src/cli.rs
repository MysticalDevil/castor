use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "castor")]
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
    },

    /// Delete a specific session
    Delete {
        /// The ID or path of the session to delete
        id: String,

        /// Perform a hard delete (bypass trash)
        #[arg(long)]
        hard: bool,

        /// Dry-run mode: show what would be done without actually deleting
        #[arg(long, default_value_t = true)]
        dry_run: bool,

        /// Confirm the operation (required for actual deletion)
        #[arg(long)]
        confirm: bool,
    },

    /// Restore a session from the trash
    Restore {
        /// The ID of the batch or session to restore
        id: String,
    },

    /// Show audit history
    History {
        /// Limit the number of entries
        #[arg(short, long, default_value_t = 10)]
        limit: usize,
    },
}
