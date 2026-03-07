use crate::core::Session;
use crate::core::session::SessionHealth;
use colored::Colorize;
use std::io::Write;
use unicode_width::UnicodeWidthStr;

/// Truncates a string to a maximum visual width, adding ".." if truncated.
pub fn truncate_visual(s: &str, max_width: usize) -> String {
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
pub fn format_cell_raw(text: &str, width: usize) -> (String, usize) {
    let truncated = truncate_visual(text, width);
    let visual_w = truncated.width();
    (truncated, width.saturating_sub(visual_w))
}

/// Core table cell formatter with color support.
pub fn render_cell(
    text: &str,
    width: usize,
    is_header: bool,
    health: Option<&SessionHealth>,
) -> String {
    let (truncated, pad_count) = format_cell_raw(text, width);
    let padding = " ".repeat(pad_count);

    if is_header {
        format!("{}{}", truncated.cyan().bold(), padding)
    } else if let Some(h) = health {
        let colored = match h {
            SessionHealth::Unknown => truncated.dimmed(),
            SessionHealth::Ok => truncated.green(),
            SessionHealth::Warn => truncated.yellow(),
            SessionHealth::Error => truncated.red().bold(),
            SessionHealth::Risk => truncated.magenta().bold(),
        };
        format!("{}{}", colored, padding)
    } else {
        format!("{}{}", truncated, padding)
    }
}

pub const ID_W: usize = 10;
pub const UPDATE_W: usize = 17;
pub const HOST_W: usize = 30;
pub const HEALTH_W: usize = 8;
pub const HEAD_W: usize = 30;

/// Renders the list header to the provided writer.
pub fn write_list_header<W: Write>(mut w: W) -> std::io::Result<()> {
    writeln!(
        w,
        "{} {} {} {} {}",
        render_cell("ID", ID_W, true, None),
        render_cell("UPDATE", UPDATE_W, true, None),
        render_cell("HOST", HOST_W, true, None),
        render_cell("HEALTH", HEALTH_W, true, None),
        render_cell("HEAD", HEAD_W, true, None)
    )
}

/// Renders a single session row to the provided writer.
pub fn write_session_row<W: Write>(
    mut w: W,
    s: &Session,
    home: Option<&str>,
) -> std::io::Result<()> {
    let display_id =
        s.id.strip_suffix(".json")
            .unwrap_or(&s.id)
            .split('-')
            .next_back()
            .unwrap_or(&s.id);

    let host_raw = if let Some(path) = &s.host_path {
        crate::utils::fs::format_host(path, home)
    } else {
        s.project_id.clone()
    };

    let head_raw = s.name.as_deref().unwrap_or("---");
    let updated = s.updated_at.format("%Y-%m-%d %H:%M").to_string();
    let health = &s.health;

    writeln!(
        w,
        "{} {} {} {} {}",
        render_cell(display_id, ID_W, false, None),
        render_cell(&updated, UPDATE_W, false, None),
        render_cell(&host_raw, HOST_W, false, None),
        render_cell(&health.to_string(), HEALTH_W, false, Some(health)),
        render_cell(head_raw, HEAD_W, false, None)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use std::path::PathBuf;

    #[test]
    fn test_write_session_row() {
        let mut buf = Vec::new();
        let s = Session {
            id: "session-2026-03-08T12-00-aaaa1111.json".into(),
            project_id: "p1".into(),
            host_path: Some(PathBuf::from("/home/user/proj")),
            name: Some("Test Name".into()),
            path: PathBuf::from("fake"),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            size: 100,
            health: crate::core::session::SessionHealth::Unknown,
            validation_notes: Vec::new(),
        };

        write_session_row(&mut buf, &s, Some("/home/user")).unwrap();
        let output = String::from_utf8_lossy(&buf);
        assert!(output.contains("aaaa1111"));
        assert!(output.contains("~/proj"));
        assert!(output.contains("Test Name"));
    }

    #[test]
    fn test_render_cell_logic() {
        use crate::core::session::SessionHealth;

        let header = render_cell("ID", 5, true, None);
        assert!(header.contains("ID"));

        let ok_cell = render_cell("OK", 5, false, Some(&SessionHealth::Ok));
        assert!(ok_cell.contains("OK"));

        let risk_cell = render_cell("RISK", 5, false, Some(&SessionHealth::Risk));
        assert!(risk_cell.contains("RISK"));
    }
}
