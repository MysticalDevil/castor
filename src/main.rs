use castor::cli::{Cli, Commands};
use castor::config::Config;
use castor::core::Registry;
use castor::core::session::SessionHealth;
use castor::error::Result;
use castor::ops::{Executor, export, prune, stats::StorageStats};
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
            let sessions = registry.list();
            let mut matches = Vec::new();
            let pattern_lower = pattern.to_lowercase();

            for s in sessions {
                let content = std::fs::read_to_string(&s.path)?;
                let is_match = if ignore_case {
                    content.to_lowercase().contains(&pattern_lower)
                } else {
                    content.contains(&pattern)
                };

                if is_match {
                    matches.push(s);
                }
            }

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
            println!(
                "{}",
                "Castor Doctor - Environment Diagnostics".cyan().bold()
            );
            let home = std::env::var("HOME")
                .map(std::path::PathBuf::from)
                .unwrap_or_default();

            // 1. Basic Directory Checks
            let gemini_base = home.join(".gemini");
            if gemini_base.exists() {
                println!("{} Gemini base directory: {:?}", "✓".green(), gemini_base);
            } else {
                println!(
                    "{} Gemini base directory NOT FOUND at {:?}",
                    "✗".red(),
                    gemini_base
                );
            }

            if executor.config.gemini_sessions_path.exists() {
                println!(
                    "{} Sessions path: {:?}",
                    "✓".green(),
                    executor.config.gemini_sessions_path
                );
            } else {
                println!(
                    "{} Sessions path NOT FOUND: {:?}",
                    "✗".red(),
                    executor.config.gemini_sessions_path
                );
            }

            if executor.config.trash_path.exists() {
                println!(
                    "{} Trash directory: {:?}",
                    "✓".green(),
                    executor.config.trash_path
                );
            } else {
                println!(
                    "{} Trash directory NOT FOUND: {:?}",
                    "✗".red(),
                    executor.config.trash_path
                );
            }

            // 2. Integrity Detection
            registry.reload()?;
            let sessions = registry.list();
            let total = sessions.len();
            let mut orphaned = 0;
            let mut errors = 0;
            let mut risks = 0;
            let mut no_root_file = 0;

            for s in sessions {
                match s.calculate_health() {
                    SessionHealth::Warn => orphaned += 1,
                    SessionHealth::Error => errors += 1,
                    SessionHealth::Risk => risks += 1,
                    SessionHealth::Unknown | SessionHealth::Ok => {}
                }
                if s.host_path.is_none() {
                    no_root_file += 1;
                }
            }

            println!("\n{}", "Session Integrity:".yellow().bold());
            println!("{:<25} {}", "Total Sessions:", total);

            if orphaned > 0 {
                println!(
                    "{:<25} {}",
                    "Orphaned Sessions:",
                    orphaned.to_string().red().bold()
                );
            } else {
                println!("{:<25} {}", "Orphaned Sessions:", "0".green());
            }

            if errors > 0 {
                println!(
                    "{:<25} {}",
                    "Corrupted Sessions:",
                    errors.to_string().red().bold()
                );
            }

            if risks > 0 {
                println!(
                    "{:<25} {}",
                    "Untrusted Sessions:",
                    risks.to_string().magenta().bold()
                );
            }

            if no_root_file > 0 {
                println!(
                    "{:<25} {}",
                    "Untracked Hosts:",
                    no_root_file.to_string().yellow()
                );
            }

            if orphaned > 0 || errors > 0 || risks > 0 {
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
