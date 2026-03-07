use castor::cli::{Cli, Commands};
use castor::config::Config;
use castor::core::Registry;
use castor::error::Result;
use castor::ops::Executor;
use clap::Parser;
use colored::Colorize;
use unicode_width::UnicodeWidthStr;

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
/// Styling is applied only to the text, not the padding, to ensure alignment.
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
        Some(Commands::List { json }) => {
            registry.reload()?;
            let sessions = registry.list();
            if json {
                println!("{}", serde_json::to_string_pretty(sessions)?);
            } else {
                // Fixed column widths
                let id_w = 10;
                let update_w = 17;
                let host_w = 30;
                let head_w = 30;

                // Print Headers
                println!("{} {} {} {}", 
                    format_cell("ID", id_w, true),
                    format_cell("UPDATE", update_w, true),
                    format_cell("HOST", host_w, true),
                    format_cell("HEAD", head_w, true));

                for s in sessions {
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
                    
                    println!("{} {} {} {}", 
                        format_cell(display_id, id_w, false),
                        format_cell(&updated, update_w, false),
                        format_cell(&host_raw, host_w, false),
                        format_cell(head_raw, head_w, false));
                }
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

            let is_dry = dry_run && !confirm;

            if hard {
                executor.delete_hard(session, is_dry)?;
                if !is_dry {
                    println!("Session {} permanently deleted.", session.id.green());
                }
            } else {
                executor.delete_soft(session, is_dry)?;
                if !is_dry {
                    println!("Session {} moved to trash.", session.id.yellow());
                }
            }
        }
        Some(Commands::Restore { id }) => {
            executor.restore(&id, false)?;
            println!("Session {} restored.", id.green());
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
        None => {
            println!("Use --help for available commands");
        }
    }

    Ok(())
}
