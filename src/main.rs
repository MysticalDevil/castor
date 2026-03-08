use castor::cli::{Cli, Commands};
use castor::config::Config;
use castor::core::Registry;
use castor::error::Result;
use castor::ops::{
    doctor::DoctorReport, executor::Executor, export, grep, prune, stats::StorageStats,
};
use castor::utils::term::{write_list_header, write_session_row};
use clap::{CommandFactory, Parser};
use clap_complete::generate;
use colored::Colorize;
use std::collections::HashMap;
use std::io;

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    config.ensure_dirs()?;

    let mut registry = Registry::new(
        &config.gemini_sessions_path,
        &config.cache_path.join("metadata.json"),
    );
    let executor = Executor::new(config);
    let home_dir = std::env::var("HOME").ok();
    let stdout = io::stdout();
    let mut handle = stdout.lock();

    match cli.command {
        Some(Commands::Tui) => {
            castor::tui::run(registry, executor)?;
        }
        Some(Commands::List {
            json,
            group,
            page_size,
        }) => {
            registry.reload()?;
            let sessions = registry.list();

            if json {
                println!("{}", serde_json::to_string_pretty(sessions)?);
            } else if group {
                let mut groups: HashMap<String, Vec<&castor::core::Session>> = HashMap::new();
                for s in sessions {
                    let host = if let Some(path) = &s.host_path {
                        castor::utils::fs::format_host(path, home_dir.as_deref())
                    } else {
                        s.project_id.clone()
                    };
                    groups.entry(host).or_default().push(s);
                }

                for (host, group_sessions) in groups {
                    println!("\n{}", host.yellow().bold());
                    write_list_header(&mut handle)?;
                    for s in group_sessions {
                        write_session_row(&mut handle, s, home_dir.as_deref())?;
                    }
                }
            } else if page_size > 0 {
                let chunks = sessions.chunks(page_size);
                for (i, chunk) in chunks.enumerate() {
                    if i > 0 {
                        println!("\n--- Page {} (Press Enter for next) ---", i + 1);
                        let mut input = String::new();
                        std::io::stdin().read_line(&mut input).ok();
                    }
                    write_list_header(&mut handle)?;
                    for s in chunk {
                        write_session_row(&mut handle, s, home_dir.as_deref())?;
                    }
                }
            } else {
                write_list_header(&mut handle)?;
                for s in sessions {
                    write_session_row(&mut handle, s, home_dir.as_deref())?;
                }
            }
        }
        Some(Commands::Cat { id, raw }) => {
            registry.reload()?;
            let session = registry.find(&id).ok_or_else(|| {
                castor::error::CastorError::PathNotFound(std::path::PathBuf::from(id.clone()))
            })?;

            if raw {
                println!("{}", std::fs::read_to_string(&session.path)?);
            } else {
                println!("{}", export::session_to_markdown(session)?);
            }
        }
        Some(Commands::Grep {
            pattern,
            ignore_case,
        }) => {
            registry.reload()?;
            let matches = grep::search_sessions(registry.list(), &pattern, ignore_case)?;

            if matches.is_empty() {
                println!("No sessions found containing '{}'", pattern);
            } else {
                println!("Found {} sessions containing '{}':", matches.len(), pattern);
                write_list_header(&mut handle)?;
                for s in matches {
                    write_session_row(&mut handle, s, home_dir.as_deref())?;
                }
            }
        }
        Some(Commands::Export { id, output }) => {
            registry.reload()?;
            let session = registry.find(&id).ok_or_else(|| {
                castor::error::CastorError::PathNotFound(std::path::PathBuf::from(id.clone()))
            })?;

            let path = export::export_session(session, output.as_deref())?;
            println!("Session exported to {}", path.display().to_string().green());
        }
        Some(Commands::Stats) => {
            registry.reload()?;
            let s = StorageStats::calculate(registry.list(), &executor.config);

            println!("{}", "Castor Storage Statistics".cyan().bold());
            println!("{:<20} {}", "Total Sessions:", s.total_sessions);
            println!(
                "{:<20} {:.2} MB",
                "Total Size:",
                s.total_size_bytes as f64 / 1024.0 / 1024.0
            );
            println!(
                "{:<20} {:.2} MB",
                "Trash Size:",
                s.trash_size_bytes as f64 / 1024.0 / 1024.0
            );
        }
        Some(Commands::ClearTrash { confirm }) => {
            if !confirm {
                println!(
                    "{}",
                    "Please provide --confirm to empty the trash.".yellow()
                );
                return Ok(());
            }
            if executor.config.trash_path.exists() {
                std::fs::remove_dir_all(&executor.config.trash_path)?;
                std::fs::create_dir_all(&executor.config.trash_path)?;
                println!("{}", "Trash cleared successfully.".green());
            }
        }
        Some(Commands::Prune {
            days,
            hard,
            dry_run,
            confirm,
        }) => {
            registry.reload()?;
            let to_prune = prune::find_sessions_to_prune(registry.list(), days);

            if to_prune.is_empty() {
                println!("No sessions older than {} days found.", days);
                return Ok(());
            }

            println!("Found {} sessions to prune:", to_prune.len());
            write_list_header(&mut handle)?;
            for s in &to_prune {
                write_session_row(&mut handle, s, home_dir.as_deref())?;
            }

            let is_actually_dry = dry_run && !confirm;

            if is_actually_dry {
                println!("\n{}", "[DRY-RUN] Pruning would occur if confirmed.".cyan());
            } else {
                println!("\nPruning {} sessions...", to_prune.len());
                for s in to_prune {
                    if hard {
                        executor.delete_hard(&s, false)?;
                    } else {
                        executor.delete_soft(&s, false)?;
                    }
                }
                println!("{}", "Pruning complete.".green());
            }
        }
        Some(Commands::Delete {
            id,
            hard,
            dry_run,
            confirm,
        }) => {
            registry.reload()?;
            let session = registry.find(&id).ok_or_else(|| {
                castor::error::CastorError::PathNotFound(std::path::PathBuf::from(id.clone()))
            })?;

            let is_actually_dry = dry_run && !confirm;

            if hard {
                executor.delete_hard(session, is_actually_dry)?;
                if !is_actually_dry {
                    println!("Session {} permanently deleted.", session.id.green());
                }
            } else {
                executor.delete_soft(session, is_actually_dry)?;
                if !is_actually_dry {
                    println!("Session {} moved to trash.", session.id.yellow());
                }
            }
        }
        Some(Commands::Restore { id, dry_run }) => {
            executor.restore(&id, dry_run)?;
            if !dry_run {
                println!("Session {} restored.", id.green());
            }
        }
        Some(Commands::History { limit }) => {
            let history = executor.logger.load_history()?;
            for entry in history.iter().rev().take(limit) {
                println!(
                    "{:?} - {} - {:?} - {}",
                    entry.timestamp, entry.session_id, entry.op_type, entry.batch_id
                );
            }
        }
        Some(Commands::Doctor) => {
            registry.reload()?;
            let report = DoctorReport::generate(registry.list(), &executor.config);

            println!(
                "{}",
                "Castor Doctor - Environment Diagnostics".cyan().bold()
            );

            // 1. Basic Directory Checks
            let home = std::env::var("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_default();
            let gemini_base = home.join(".gemini");

            let check_mark = |exists: bool| {
                if exists { "✓".green() } else { "✗".red() }
            };

            println!(
                "{} Gemini base directory: {:?}",
                check_mark(report.gemini_base_exists),
                gemini_base
            );
            println!(
                "{} Sessions path: {:?}",
                check_mark(report.sessions_path_exists),
                executor.config.gemini_sessions_path
            );
            println!(
                "{} Trash directory: {:?}",
                check_mark(report.trash_path_exists),
                executor.config.trash_path
            );

            // 2. Integrity
            println!("\n{}", "Session Integrity:".yellow().bold());
            println!("{:<25} {}", "Total Sessions:", report.total_sessions);

            if report.orphaned_count > 0 {
                println!(
                    "{:<25} {}",
                    "Orphaned Sessions:",
                    report.orphaned_count.to_string().red().bold()
                );
                println!("   (Hosts no longer exist on disk)");
            } else {
                println!("{:<25} {}", "Orphaned Sessions:", "0".green());
            }

            if report.corrupted_count > 0 {
                println!(
                    "{:<25} {}",
                    "Corrupted Sessions:",
                    report.corrupted_count.to_string().red().bold()
                );
            }

            if report.untrusted_count > 0 {
                println!(
                    "{:<25} {}",
                    "Untrusted Sessions:",
                    report.untrusted_count.to_string().magenta().bold()
                );
            }

            if report.untracked_hosts_count > 0 {
                println!(
                    "{:<25} {}",
                    "Untracked Hosts:",
                    report.untracked_hosts_count.to_string().yellow()
                );
            }

            if report.orphaned_count > 0 || report.corrupted_count > 0 || report.untrusted_count > 0
            {
                println!(
                    "\n{} Hint: Use `castor list` to find unhealthy sessions or `prune` to clean up.",
                    "ℹ".blue()
                );
            }
        }
        Some(Commands::Completions { shell }) => {
            let mut cmd = Cli::command();
            let name = cmd.get_name().to_string();
            generate(shell, &mut cmd, name, &mut io::stdout());
        }
        None => {
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
