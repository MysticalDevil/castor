use castor::cli::{Cli, Commands};
use castor::config::Config;
use castor::core::Registry;
use castor::error::Result;
use castor::ops::Executor;
use clap::Parser;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;
use std::collections::HashMap;

/// Truncates a string to a maximum visual width, adding ".." if truncated.
fn truncate_visual(s: &str, max_width: usize) -> String {
    if s.width() <= max_width {
        return s.to_string();
    }

    let mut result = String::new();
    let mut current_width = 0;
    for c in s.chars() {
        let char_width = unicode_width::UnicodeWidthChar::width(c).unwrap_or(0);
        if current_width + char_width + 2 > max_width {
            result.push_str("..");
            break;
        }
        result.push(c);
        current_width += char_width;
    }
    result
}

/// Formats a cell with fixed visual width and optional styling.
fn format_cell(text: &str, width: usize, is_header: bool) -> String {
    let truncated = truncate_visual(text, width);
    let visual_w = truncated.width();
    let padding = " ".repeat(width.saturating_sub(visual_w));
    
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

fn print_session(s: &castor::core::Session) {
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
        castor::utils::fs::format_host(path)
    } else {
        s.project_id.clone()
    };
    
    let head_raw = s.name.as_deref().unwrap_or("---");
    let updated = s.updated_at.format("%Y-%m-%d %H:%M").to_string();

    let truncate = |text: &str, max_len: usize| -> String {
        truncate_visual(text, max_len)
    };

    println!("{} {} {} {}", 
        format_cell(&truncate(display_id, 10), id_w, false),
        format_cell(&updated, update_w, false),
        format_cell(&truncate(&host_raw, 30), host_w, false),
        format_cell(&truncate(head_raw, 25), head_w, false));
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let config = Config::load(cli.config.as_deref())?;
    config.ensure_dirs()?;

    let mut registry = Registry::new(&config.gemini_sessions_path);
    let executor = Executor::new(config);

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
                        castor::utils::fs::format_host(path)
                    } else {
                        s.project_id.clone()
                    };
                    groups.entry(host).or_default().push(s);
                }

                for (host, group_sessions) in groups {
                    println!("\n{}", host.yellow().bold());
                    print_list_header();
                    for s in group_sessions {
                        print_session(s);
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
                        print_session(s);
                    }
                }
            } else {
                print_list_header();
                for s in sessions {
                    print_session(s);
                }
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
            
            // Check Gemini Home
            let home = std::env::var("HOME").map(std::path::PathBuf::from).unwrap_or_default();
            let gemini_base = home.join(".gemini");
            if gemini_base.exists() {
                println!("{} Gemini base directory: {:?}", "✓".green(), gemini_base);
            } else {
                println!("{} Gemini base directory NOT FOUND at {:?}", "✗".red(), gemini_base);
            }

            // Check Sessions Path
            if executor.config.gemini_sessions_path.exists() {
                println!("{} Sessions path: {:?}", "✓".green(), executor.config.gemini_sessions_path);
            } else {
                println!("{} Sessions path NOT FOUND: {:?}", "✗".red(), executor.config.gemini_sessions_path);
            }

            // Check Trash Path
            if executor.config.trash_path.exists() {
                println!("{} Trash directory: {:?}", "✓".green(), executor.config.trash_path);
            } else {
                println!("{} Trash directory NOT FOUND: {:?}", "✗".red(), executor.config.trash_path);
            }

            // Scan check
            registry.reload()?;
            let count = registry.list().len();
            println!("{} Detected sessions: {}", "✓".green(), count);

            if count == 0 {
                println!("{} Hint: Ensure you have used Gemini CLI at least once.", "ℹ".blue());
            }
        }
        None => {
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
