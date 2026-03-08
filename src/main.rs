mod audit;
mod cli;
mod config;
mod core;
mod error;
mod ops;
mod tui;
mod utils;

use crate::config::Config;
use crate::core::Registry;
use crate::error::Result;
use crate::ops::{
    doctor::DoctorReport, executor::Executor, export, grep, prune, stats::StorageStats,
};
use clap::{CommandFactory, Parser};
use cli::{Cli, Commands};

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    config.ensure_dirs()?;

    let executor = Executor::new(config);
    let mut registry = Registry::new(
        &executor.config.gemini_sessions_path,
        &executor.config.cache_path,
    );

    if let Some(command) = cli.command {
        match command {
            Commands::List {
                group,
                json,
                page_size: _,
            } => {
                registry.reload()?;
                let sessions = registry.list();

                if json {
                    let raw_sessions: Vec<_> = sessions.iter().map(|s| &**s).collect();
                    println!("{}", serde_json::to_string_pretty(&raw_sessions)?);
                } else if group {
                    utils::term::print_sessions_grouped(sessions, &executor.config);
                } else {
                    utils::term::print_sessions_table(sessions, &executor.config);
                }
            }
            Commands::Cat { id, raw } => {
                registry.reload()?;
                if let Some(session) = registry.find(&id) {
                    if raw {
                        println!("{}", session.get_content()?);
                    } else {
                        println!("{}", export::session_to_markdown(&session)?);
                    }
                } else {
                    println!("Session {} not found.", id);
                }
            }
            Commands::Grep {
                pattern,
                ignore_case,
            } => {
                registry.reload()?;
                let matches = grep::search_sessions(registry.list(), &pattern, ignore_case)?;
                utils::term::print_sessions_table(&matches, &executor.config);
            }
            Commands::Export { id, output } => {
                registry.reload()?;
                if let Some(session) = registry.find(&id) {
                    let path = export::export_session(&session, output.as_deref())?;
                    println!("Exported session to {:?}", path);
                } else {
                    println!("Session {} not found.", id);
                }
            }
            Commands::Stats => {
                registry.reload()?;
                let s = StorageStats::calculate(registry.list(), &executor.config);
                println!("Sessions: {}", s.total_sessions);
                println!(
                    "Total Size: {:.2} MB",
                    s.total_size_bytes as f64 / 1024.0 / 1024.0
                );
                println!(
                    "Trash Size: {:.2} MB",
                    s.trash_size_bytes as f64 / 1024.0 / 1024.0
                );
            }
            Commands::Prune {
                days,
                confirm,
                dry_run,
                hard,
            } => {
                registry.reload()?;
                let to_prune = prune::find_sessions_to_prune(registry.list(), days);

                if to_prune.is_empty() {
                    println!("No sessions older than {} days found.", days);
                    return Ok(());
                }

                println!("Found {} sessions to prune:", to_prune.len());
                utils::term::print_sessions_table(&to_prune, &executor.config);

                if confirm {
                    for session in to_prune {
                        if hard {
                            executor.delete_hard(&session, dry_run)?;
                        } else {
                            executor.delete_soft(&session, dry_run)?;
                        }
                    }
                    if dry_run {
                        println!("\nDry-run complete. No files were moved.");
                    } else {
                        println!("\nPrune complete.");
                    }
                } else {
                    println!("\nRun with --confirm to perform the operation.");
                }
            }
            Commands::Delete {
                id,
                confirm,
                dry_run,
                hard,
            } => {
                registry.reload()?;
                if let Some(session) = registry.find(&id) {
                    if !confirm {
                        println!("Deleting session {}:", id);
                        utils::term::print_sessions_table(&[session], &executor.config);
                        println!("\nRun with --confirm to proceed.");
                        return Ok(());
                    }

                    if hard {
                        executor.delete_hard(&session, dry_run)?;
                    } else {
                        executor.delete_soft(&session, dry_run)?;
                    }

                    if dry_run {
                        println!("Dry-run: Session {} would be deleted.", id);
                    } else {
                        println!("Session {} deleted.", id);
                    }
                } else {
                    println!("Session {} not found.", id);
                }
            }
            Commands::Restore { id, dry_run } => {
                let batch_id = executor.restore(&id, dry_run)?;
                if dry_run {
                    println!(
                        "Dry-run: Session {} would be restored (Batch: {}).",
                        id, batch_id
                    );
                } else {
                    println!("Session {} restored successfully.", id);
                }
            }
            Commands::ClearTrash { confirm } => {
                if confirm {
                    let count = executor.clear_trash()?;
                    println!("Trash cleared. Removed {} sessions.", count);
                } else {
                    println!("Run with --confirm to permanently empty the trash.");
                }
            }
            Commands::History { limit } => {
                let history = executor.logger.load_history()?;
                println!("{:<30} {:<15} {:<40}", "Time", "Action", "Session ID");
                println!("{}", "-".repeat(85));
                for entry in history.iter().rev().take(limit) {
                    println!(
                        "{:<30} {:<15} {:<40}",
                        entry.timestamp.format("%Y-%m-%d %H:%M:%S"),
                        format!("{:?}", entry.op_type),
                        entry.session_id
                    );
                }
            }
            Commands::Doctor => {
                registry.reload()?;
                let report = DoctorReport::generate(registry.list(), &executor.config);
                println!("Doctor Report:");
                println!("  Total Sessions: {}", report.total_sessions);
                println!("  Corrupted:      {}", report.corrupted_count);
                println!("  Orphaned:       {}", report.orphaned_count);
                println!("  High Risk:      {}", report.high_risk_count);
                println!("\nSuggestions:");
                for s in report.suggestions {
                    println!("  - {}", s);
                }
            }
            Commands::Tui => {
                tui::run(registry, executor)?;
            }
            Commands::Completions { shell } => {
                let mut cmd = Cli::command();
                let name = cmd.get_name().to_string();
                clap_complete::generate(shell, &mut cmd, name, &mut std::io::stdout());
            }
        }
    } else {
        tui::run(registry, executor)?;
    }

    Ok(())
}
