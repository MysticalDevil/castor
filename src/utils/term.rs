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
    const ID_W: usize = 12;
    const PROJECT_W: usize = 28;
    const UPDATED_W: usize = 19;
    const SIZE_W: usize = 10;
    const HEALTH_W: usize = 12;

    let icons = Icons::get(config.icon_set);
    let home = std::env::var("HOME").ok();
    println!(
        "{} {} {} {} {}",
        render_cell("ID", ID_W).bold(),
        render_cell("Project", PROJECT_W).bold(),
        render_cell("Updated", UPDATED_W).bold(),
        render_cell("Size", SIZE_W).bold(),
        render_cell("Health", HEALTH_W).bold()
    );
    println!(
        "{}",
        "-".repeat(ID_W + PROJECT_W + UPDATED_W + SIZE_W + HEALTH_W + 4)
    );

    for s in sessions {
        let health_plain = format!("{} {}", icon_for_health(&icons, &s.health), s.health);
        let health_text = match &s.health {
            SessionHealth::Ok => health_plain.green(),
            SessionHealth::Warn => health_plain.yellow(),
            SessionHealth::Error => health_plain.red(),
            SessionHealth::Risk => health_plain.magenta(),
            SessionHealth::Unknown => health_plain.dimmed(),
        };

        println!(
            "{} {} {} {} {}",
            render_cell(&s.display_id, ID_W).cyan(),
            render_cell(&project_display(s, home.as_deref()), PROJECT_W),
            render_cell(
                &s.updated_at.format("%Y-%m-%d %H:%M:%S").to_string(),
                UPDATED_W
            )
            .blue(),
            render_cell(&format_size(s.size), SIZE_W),
            render_cell(&health_text.to_string(), HEALTH_W)
        );
    }
}

pub fn print_sessions_table_paginated(
    sessions: &[Arc<Session>],
    config: &Config,
    page_size: usize,
) {
    if page_size == 0 || sessions.len() <= page_size {
        print_sessions_table(sessions, config);
        return;
    }

    let total_pages = sessions.len().div_ceil(page_size);
    for (idx, chunk) in sessions.chunks(page_size).enumerate() {
        println!(
            "Page {}/{} ({} sessions)",
            idx + 1,
            total_pages,
            chunk.len()
        );
        print_sessions_table(chunk, config);
        if idx + 1 < total_pages {
            println!();
        }
    }
}

fn icon_for_health<'a>(icons: &'a Icons, health: &SessionHealth) -> &'a str {
    match health {
        SessionHealth::Ok => icons.ok,
        SessionHealth::Warn => icons.warn,
        SessionHealth::Error => icons.error,
        SessionHealth::Risk => icons.risk,
        SessionHealth::Unknown => icons.unknown,
    }
}

fn project_display(session: &Session, home: Option<&str>) -> String {
    if let Some(host) = &session.host_path {
        return crate::utils::fs::format_host(host, home);
    }
    session.project_id.clone()
}

fn format_size(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;

    let b = bytes as f64;
    if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.2} KB", b / KB)
    } else {
        format!("{} B", bytes)
    }
}

pub fn print_sessions_grouped_paginated(
    sessions: &[Arc<Session>],
    config: &Config,
    page_size: usize,
) {
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
        if page_size == 0 {
            print_sessions_table(&groups[key], config);
            continue;
        }
        let group_sessions = &groups[key];
        if group_sessions.len() <= page_size {
            print_sessions_table(group_sessions, config);
            continue;
        }
        let total_pages = group_sessions.len().div_ceil(page_size);
        for (idx, chunk) in group_sessions.chunks(page_size).enumerate() {
            println!("Group page {}/{}", idx + 1, total_pages);
            print_sessions_table(chunk, config);
            if idx + 1 < total_pages {
                println!();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
            display_id: "abc".into(),
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

    #[test]
    fn test_project_display_prefers_host_path() {
        let now = chrono::Utc::now();
        let s = Session {
            id: "session-2026-03-08T12-00-abcdef01.json".into(),
            display_id: "abcdef01".into(),
            project_id: "gemini-sm".into(),
            host_path: Some(PathBuf::from("/home/omega/Projects/gemini-sm")),
            name: None,
            path: PathBuf::from("p.json"),
            created_at: now,
            updated_at: now,
            size: 1024,
            health: SessionHealth::Ok,
            validation_notes: Vec::new(),
        };
        assert_eq!(
            project_display(&s, Some("/home/omega")),
            "~/Projects/gemini-sm"
        );
    }

    #[test]
    fn test_format_size_human_readable() {
        assert_eq!(format_size(111), "111 B");
        assert_eq!(format_size(2048), "2.00 KB");
        assert_eq!(format_size(5 * 1024 * 1024), "5.00 MB");
    }

    #[test]
    fn test_paginated_table_with_zero_page_size_falls_back() {
        let config = Config::default();
        let now = chrono::Utc::now();
        let s = Arc::new(Session {
            id: "abc".into(),
            display_id: "abc".into(),
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
        print_sessions_table_paginated(&[s], &config, 0);
    }

    #[test]
    fn test_grouped_pagination_smoke() {
        let config = Config::default();
        let now = chrono::Utc::now();
        let a = Arc::new(Session {
            id: "a".into(),
            display_id: "a".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: "a.json".into(),
            created_at: now,
            updated_at: now,
            size: 1024,
            health: SessionHealth::Ok,
            validation_notes: Vec::new(),
        });
        let b = Arc::new(Session {
            id: "b".into(),
            display_id: "b".into(),
            project_id: "p1".into(),
            host_path: None,
            name: None,
            path: "b.json".into(),
            created_at: now,
            updated_at: now,
            size: 1024,
            health: SessionHealth::Ok,
            validation_notes: Vec::new(),
        });
        print_sessions_grouped_paginated(&[a, b], &config, 1);
    }
}
