use castor::cli::{Cli, Commands};
use castor::config::Config;
use castor::core::Registry;
use castor::error::Result;
use castor::ops::Executor;
use castor::utils::term::format_cell_raw;
use clap::{Parser, CommandFactory};
use clap_complete::generate;
use colored::Colorize;
use std::collections::HashMap;
use std::io;
use chrono::{Utc, Duration};

/// Formats a cell with fixed visual width and optional styling for the CLI.
fn format_cell(text: &str, width: usize, is_header: bool) -> String {
    let (truncated, pad_count) = format_cell_raw(text, width);
    let padding = " ".repeat(pad_count);
    
    if is_header {
        format!("{}{}", truncated.cyan().bold(), padding)
    } else {
        format!("{}{}", truncated, padding)
    }
}

fn print_list_header() {
    let id_w = 10;
    let update_w = 17;
    let host_w = 30;
    let head_w = 30;
    println!("{} {} {} {}", 
        format_cell("ID", id_w, true),
        format_cell("UPDATE", update_w, true),
        format_cell("HOST", host_w, true),
        format_cell("HEAD", head_w, true));
}

fn print_session(s: &castor::core::Session, home: Option<&str>) {
    let id_w = 10;
    let update_w = 17;
    let host_w = 30;
    let head_w = 30;

    let display_id = s.id.strip_suffix(".json")
        .unwrap_or(&s.id)
        .split('-')
        .last()
        .unwrap_or(&s.id);
    
    let host_raw = if let Some(path) = &s.host_path {
        castor::utils::fs::format_host(path, home)
    } else {
        s.project_id.clone()
    };
    
    let head_raw = s.name.as_deref().unwrap_or("---");
    let updated = s.updated_at.format("%Y-%m-%d %H:%M").to_string();

    println!("{} {} {} {}", 
        format_cell(display_id, id_w, false),
        format_cell(&updated, update_w, false),
        format_cell(&host_raw, host_w, false),
        format_cell(head_raw, head_w, false));
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    config.ensure_dirs()?;

    let mut registry = Registry::new(&config.gemini_sessions_path);
    let executor = Executor::new(config);
    let home_dir = std::env::var("HOME").ok();

    match cli.command {
        Some(Commands::Tui) => {
            castor::tui::run(registry, executor)?;
        }
        Some(Commands::List { json, group, page_size }) => {
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
                    print_list_header();
                    for s in group_sessions {
                        print_session(s, home_dir.as_deref());
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
                    print_list_header();
                    for s in chunk {
                        print_session(s, home_dir.as_deref());
                    }
                }
            } else {
                print_list_header();
                for s in sessions {
                    print_session(s, home_dir.as_deref());
                }
            }
        }
        Some(Commands::Cat { id, raw }) => {
            registry.reload()?;
            let session = registry.find(&id).ok_or_else(|| {
                castor::error::CastorError::PathNotFound(std::path::PathBuf::from(id.clone()))
            })?;

            let content = std::fs::read_to_string(&session.path)?;
            if raw {
                println!("{}", content);
            } else {
                let json: serde_json::Value = serde_json::from_str(&content)?;
                if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                    for msg in messages {
                        let role = msg.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
                        let color_role = if role == "user" {
                            role.blue().bold()
                        } else {
                            role.green().bold()
                        };
                        
                        println!("\n--- {} ---", color_role);
                        
                        let content_val = msg.get("content").unwrap_or(&serde_json::Value::Null);
                        if let Some(text) = content_val.as_str() {
                            println!("{}", text);
                        } else if let Some(arr) = content_val.as_array() {
                            for item in arr {
                                if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                                    println!("{}", text);
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(Commands::Grep { pattern, ignore_case }) => {
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
                print_list_header();
                for s in matches {
                    print_session(s, home_dir.as_deref());
                }
            }
        }
        Some(Commands::Export { id, output }) => {
            registry.reload()?;
            let session = registry.find(&id).ok_or_else(|| {
                castor::error::CastorError::PathNotFound(std::path::PathBuf::from(id.clone()))
            })?;

            let content = std::fs::read_to_string(&session.path)?;
            let json: serde_json::Value = serde_json::from_str(&content)?;
            let mut markdown = format!("# Session: {}\n\n", session.id);

            if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
                for msg in messages {
                    let role = msg.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
                    markdown.push_str(&format!("## {}\n", role.to_uppercase()));
                    
                    let content_val = msg.get("content").unwrap_or(&serde_json::Value::Null);
                    if let Some(text) = content_val.as_str() {
                        markdown.push_str(&format!("{}\n\n", text));
                    } else if let Some(arr) = content_val.as_array() {
                        for item in arr {
                            if let Some(text) = item.get("text").and_then(|v| v.as_str()) {
                                markdown.push_str(&format!("{}\n\n", text));
                            }
                        }
                    }
                }
            }

            let out_path = output.unwrap_or_else(|| {
                let mut p = std::path::PathBuf::from(&session.id);
                p.set_extension("md");
                p
            });

            std::fs::write(&out_path, markdown)?;
            println!("Session exported to {}", out_path.display().to_string().green());
        }
        Some(Commands::Stats) => {
            registry.reload()?;
            let sessions = registry.list();
            let total_size: u64 = sessions.iter().map(|s| s.size).sum();
            
            println!("{}", "Castor Storage Statistics".cyan().bold());
            println!("{:<20} {}", "Total Sessions:", sessions.len());
            println!("{:<20} {:.2} MB", "Total Size:", total_size as f64 / 1024.0 / 1024.0);
            
            let mut trash_size = 0;
            if executor.config.trash_path.exists() {
                for entry in walkdir::WalkDir::new(&executor.config.trash_path) {
                    if let Ok(e) = entry {
                        if e.file_type().is_file() {
                            trash_size += e.metadata().map(|m| m.len()).unwrap_or(0);
                        }
                    }
                }
            }
            println!("{:<20} {:.2} MB", "Trash Size:", trash_size as f64 / 1024.0 / 1024.0);
        }
        Some(Commands::ClearTrash { confirm }) => {
            if !confirm {
                println!("{}", "Please provide --confirm to empty the trash.".yellow());
                return Ok(());
            }
            if executor.config.trash_path.exists() {
                std::fs::remove_dir_all(&executor.config.trash_path)?;
                std::fs::create_dir_all(&executor.config.trash_path)?;
                println!("{}", "Trash cleared successfully.".green());
            }
        }
        Some(Commands::Prune { days, hard, dry_run, confirm }) => {
            registry.reload()?;
            let sessions = registry.list();
            let threshold = Utc::now() - Duration::days(days as i64);
            let to_prune: Vec<_> = sessions.iter()
                .filter(|s| s.updated_at < threshold)
                .collect();

            if to_prune.is_empty() {
                println!("No sessions older than {} days found.", days);
                return Ok(());
            }

            println!("Found {} sessions to prune:", to_prune.len());
            for s in &to_prune {
                print_session(s, home_dir.as_deref());
            }

            let is_actually_dry = dry_run && !confirm;

            if is_actually_dry {
                println!("\n{}", "[DRY-RUN] Pruning would occur if confirmed.".cyan());
            } else {
                println!("\nPruning {} sessions...", to_prune.len());
                for s in to_prune {
                    if hard {
                        executor.delete_hard(s, false)?;
                    } else {
                        executor.delete_soft(s, false)?;
                    }
                }
                println!("{}", "Pruning complete.".green());
            }
        }
        Some(Commands::Delete { id, hard, dry_run, confirm }) => {
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
            println!("{}", "Castor Doctor - Environment Diagnostics".cyan().bold());
            let home = std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_default();
            
            // 1. Basic Directory Checks
            let gemini_base = home.join(".gemini");
            if gemini_base.exists() {
                println!("{} Gemini base directory: {:?}", "✓".green(), gemini_base);
            } else {
                println!("{} Gemini base directory NOT FOUND at {:?}", "✗".red(), gemini_base);
            }

            if executor.config.gemini_sessions_path.exists() {
                println!("{} Sessions path: {:?}", "✓".green(), executor.config.gemini_sessions_path);
            } else {
                println!("{} Sessions path NOT FOUND: {:?}", "✗".red(), executor.config.gemini_sessions_path);
            }

            if executor.config.trash_path.exists() {
                println!("{} Trash directory: {:?}", "✓".green(), executor.config.trash_path);
            } else {
                println!("{} Trash directory NOT FOUND: {:?}", "✗".red(), executor.config.trash_path);
            }

            // 2. Orphaned Session Detection
            registry.reload()?;
            let sessions = registry.list();
            let total = sessions.len();
            let mut orphaned = 0;
            let mut no_root_file = 0;

            for s in sessions {
                if let Some(path) = &s.host_path {
                    if !path.exists() {
                        orphaned += 1;
                    }
                } else {
                    no_root_file += 1;
                }
            }

            println!("\n{}", "Session Integrity:".yellow().bold());
            println!("{:<25} {}", "Total Sessions:", total);
            
            if orphaned > 0 {
                println!("{:<25} {}", "Orphaned Sessions:", orphaned.to_string().red().bold());
                println!("   (Hosts no longer exist on disk)");
            } else {
                println!("{:<25} {}", "Orphaned Sessions:", "0".green());
            }

            if no_root_file > 0 {
                println!("{:<25} {}", "Untracked Hosts:", no_root_file.to_string().yellow());
                println!("   (Missing .project_root metadata)");
            }

            if orphaned > 0 {
                println!("\n{} Hint: Use `castor list` to find orphaned sessions or `prune` to clean up.", "ℹ".blue());
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
