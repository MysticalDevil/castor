use crate::config::Config;
use crate::core::session::{Session, SessionHealth};
use crate::utils::icons::Icons;
use colored::*;
use std::collections::HashMap;
use std::sync::Arc;
use unicode_width::UnicodeWidthStr;

pub fn render_cell(text: &str, width: usize) -> String {
    let text_width = UnicodeWidthStr::width(text);
    if text_width <= width {
        return format!(
            "{:<width$}",
            text,
            width = width + (text.len() - text_width)
        );
    }

    let mut result = String::new();
    let mut current_width = 0;
    for c in text.chars() {
        let c_width = UnicodeWidthStr::width(c.to_string().as_str());
        if current_width + c_width + 1 > width {
            result.push('…');
            current_width += 1;
            break;
        }
        result.push(c);
        current_width += c_width;
    }

    // Pad to exact width if needed
    if current_width < width {
        result.push_str(&" ".repeat(width - current_width));
    }
    result
}

pub fn print_sessions_table(sessions: &[Arc<Session>], config: &Config) {
    let icons = Icons::get(config.icon_set);
    println!(
        "{:<10} {:<15} {:<30} {:<20} {:<10}",
        "ID".bold(),
        "Project".bold(),
        "Updated".bold(),
        "Size".bold(),
        "Health".bold()
    );
    println!("{}", "-".repeat(90));

    for s in sessions {
        let health_color = match s.health {
            SessionHealth::Ok => icons.ok.green(),
            SessionHealth::Warn => icons.warn.yellow(),
            SessionHealth::Error => icons.error.red(),
            SessionHealth::Risk => icons.risk.magenta(),
            SessionHealth::Unknown => icons.unknown.dimmed(),
        };

        println!(
            "{:<10} {:<15} {:<30} {:<20} {:<10}",
            s.id.chars().take(8).collect::<String>().cyan(),
            render_cell(&s.project_id, 14),
            s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string().blue(),
            format!("{:.2} KB", s.size as f64 / 1024.0),
            health_color
        );
    }
}

pub fn print_sessions_grouped(sessions: &[Arc<Session>], config: &Config) {
    let home = std::env::var("HOME").ok();
    let mut groups: HashMap<String, Vec<Arc<Session>>> = HashMap::new();

    for s in sessions {
        let key = if let Some(host) = &s.host_path {
            crate::utils::fs::format_host(host, home.as_deref())
        } else {
            s.project_id.clone()
        };
        groups.entry(key).or_default().push(s.clone());
    }

    let mut keys: Vec<_> = groups.keys().collect();
    keys.sort();

    for key in keys {
        println!(
            "\n{} {}",
            Icons::get(config.icon_set).folder.yellow(),
            key.bold().underline()
        );
        print_sessions_table(&groups[key], config);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_cell_logic() {
        assert_eq!(
            UnicodeWidthStr::width(render_cell("hello", 10).as_str()),
            10
        );
        let long_cjk = render_cell("这是一个很长的标题项", 10);
        assert_eq!(UnicodeWidthStr::width(long_cjk.as_str()), 10);
    }

    #[test]
    fn test_write_session_row() {
        // Just verify it doesn't panic
        let config = Config::default();
        let now = chrono::Utc::now();
        let s = Arc::new(Session {
            id: "abc".into(),
            project_id: "proj".into(),
            host_path: None,
            name: None,
            path: "p.json".into(),
            created_at: now,
            updated_at: now,
            size: 1024,
            health: SessionHealth::Ok,
            validation_notes: Vec::new(),
        });
        print_sessions_table(&[s], &config);
    }
}
