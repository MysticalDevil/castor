use crate::config::PreviewConfig;
use crate::core::Session;
use crate::error::Result;
use regex::Regex;
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static MSG_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{"type"\s*:\s*"([^"]+)"\s*,\s*"content"\s*:\s*([^}]+)\}"#)
        .unwrap_or_else(|e| panic!("invalid MSG_BLOCK_REGEX: {e}"))
});

static RE_TEXT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#""text"\s*:\s*"([^"]+)""#).unwrap_or_else(|e| panic!("invalid RE_TEXT regex: {e}"))
});

pub fn session_to_markdown(session: &Session, preview: &PreviewConfig) -> Result<String> {
    session_to_markdown_limited(session, usize::MAX, preview)
}

pub fn session_to_markdown_limited(
    session: &Session,
    limit: usize,
    preview: &PreviewConfig,
) -> Result<String> {
    // Preferred path: structured JSON preview from the head messages.
    // This aligns with "preview first few segments" semantics.
    if session.size <= preview.deep_preview_max_bytes {
        if let Ok((mut head_markdown, count, truncated)) =
            build_markdown_from_json_file(session, limit, preview.deep_preview_char_budget / 2)
            && count > 0
        {
            head_markdown.insert_str(0, "-- [ Preview from session head ] --\n\n");
            if truncated {
                head_markdown.push_str("\n\n-- [ Preview limited to first few messages ] --");
            }
            return Ok(head_markdown);
        }
    }

    let file = std::fs::File::open(&session.path)?;
    let mut reader = BufReader::new(file);
    let head_bytes = read_window_from_start(&mut reader, session.size, preview.head_bytes)?;
    let head_content = String::from_utf8_lossy(&head_bytes);
    let (mut markdown, mut count) = extract_markdown_from_content(&head_content, limit);
    let mut used_tail_preview = false;

    // Fallback path: if head window has no hits, probe a tail window to find recent messages.
    if count == 0 && session.size > preview.head_bytes {
        let tail_bytes = read_window_from_end(&mut reader, session.size, preview.tail_bytes)?;
        let tail_content = String::from_utf8_lossy(&tail_bytes);
        let (tail_markdown, tail_count) = extract_markdown_from_content(&tail_content, limit);
        if tail_count > 0 {
            markdown = tail_markdown;
            count = tail_count;
            used_tail_preview = true;
        }
    }

    if count == 0 && session.size > 0 {
        if session.size <= preview.small_full_parse_bytes {
            return session_to_markdown_full(session);
        }
        return Ok(
            "-- [ Large session: no quick preview hit. Use `castor cat` to view full content ] --"
                .to_string(),
        );
    }

    if used_tail_preview {
        markdown.insert_str(
            0,
            "-- [ Preview from recent messages (tail window) ] --\n\n",
        );
    }

    if count >= limit {
        markdown.push_str("\n\n-- [ Preview limited to first few messages ] --");
    }

    Ok(markdown)
}

pub fn session_to_markdown_deep_limited(
    session: &Session,
    limit: usize,
    preview: &PreviewConfig,
) -> Result<String> {
    if session.size > preview.deep_preview_max_bytes {
        let mut fallback = session_to_markdown_limited(session, limit, preview)?;
        fallback.push_str(
            "\n\n-- [ Deep preview skipped: file exceeds configured deep_preview_max_bytes ] --",
        );
        return Ok(fallback);
    }

    let (mut markdown, count, truncated) =
        match build_markdown_from_json_file(session, limit, preview.deep_preview_char_budget) {
            Ok(v) => v,
            Err(_) => (String::new(), 0, false),
        };
    if count == 0 {
        return session_to_markdown_limited(session, limit, preview);
    }

    if truncated {
        markdown.push_str("\n\n-- [ Deep preview truncated by limits ] --");
    }
    markdown.insert_str(0, "-- [ Deep preview ] --\n\n");
    Ok(markdown)
}

fn build_markdown_from_json_file(
    session: &Session,
    limit: usize,
    char_budget: usize,
) -> Result<(String, usize, bool)> {
    let file = std::fs::File::open(&session.path)?;
    let reader = BufReader::new(file);
    let json: serde_json::Value = serde_json::from_reader(reader)?;
    let mut markdown = String::new();
    let mut count = 0usize;
    let mut used_chars = 0usize;
    let mut truncated = false;

    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        let mut last_role = String::new();
        for msg in messages {
            if count >= limit || used_chars >= char_budget {
                truncated = true;
                break;
            }

            let role = msg
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_uppercase();
            let display_role = if role == "ASSISTANT" {
                "GEMINI".to_string()
            } else {
                role
            };

            let mut text = String::new();
            let content_val = msg.get("content").unwrap_or(&serde_json::Value::Null);
            if let Some(t) = content_val.as_str() {
                text = t.to_string();
            } else if let Some(arr) = content_val.as_array() {
                for item in arr {
                    if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                        text.push_str(t);
                        text.push('\n');
                    }
                }
            }

            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            let available = char_budget.saturating_sub(used_chars);
            if available == 0 {
                truncated = true;
                break;
            }

            let clipped_text = if trimmed.chars().count() > available {
                let mut clipped = trimmed.chars().take(available).collect::<String>();
                clipped.push_str("…");
                truncated = true;
                clipped
            } else {
                trimmed.to_string()
            };

            used_chars += clipped_text.chars().count();
            count += 1;

            if display_role == last_role {
                markdown.push_str(&format!("{}\n\n", clipped_text));
            } else {
                markdown.push_str(&format!("## {}\n{}\n\n", display_role, clipped_text));
                last_role = display_role;
            }
        }
    }

    Ok((markdown, count, truncated))
}

fn read_window_from_start(
    reader: &mut BufReader<std::fs::File>,
    file_size: u64,
    max_bytes: u64,
) -> Result<Vec<u8>> {
    let read_size = std::cmp::min(file_size, max_bytes) as usize;
    let mut buffer = vec![0; read_size];
    let n = reader.read(&mut buffer)?;
    buffer.truncate(n);
    Ok(buffer)
}

fn read_window_from_end(
    reader: &mut BufReader<std::fs::File>,
    file_size: u64,
    max_bytes: u64,
) -> Result<Vec<u8>> {
    let read_size = std::cmp::min(file_size, max_bytes) as usize;
    if read_size == 0 {
        return Ok(Vec::new());
    }

    let start_offset = file_size.saturating_sub(read_size as u64);
    reader.seek(SeekFrom::Start(start_offset))?;

    let mut buffer = vec![0; read_size];
    let n = reader.read(&mut buffer)?;
    buffer.truncate(n);
    Ok(buffer)
}

fn extract_markdown_from_content(content: &str, limit: usize) -> (String, usize) {
    let mut markdown = String::new();
    let mut last_role = String::new();
    let mut count = 0;

    for caps in MSG_BLOCK_REGEX.captures_iter(content) {
        if count >= limit {
            break;
        }

        let role_raw = &caps[1];
        let content_raw = &caps[2];

        let display_role = match role_raw {
            "user" => "USER",
            "assistant" | "gemini" => "GEMINI",
            other => other,
        }
        .to_uppercase();

        let mut text = String::new();
        if content_raw.starts_with('"') {
            text = content_raw
                .trim_matches('"')
                .replace("\\n", "\n")
                .to_string();
        } else if content_raw.contains("\"text\"") {
            for t_cap in RE_TEXT.captures_iter(content_raw) {
                text.push_str(&t_cap[1].replace("\\n", "\n"));
                text.push('\n');
            }
        }

        if !text.trim().is_empty() {
            count += 1;
            if display_role == last_role {
                markdown.push_str(&format!("{}\n\n", text.trim()));
            } else {
                markdown.push_str(&format!("## {}\n{}\n\n", display_role, text.trim()));
                last_role = display_role;
            }
        }
    }

    (markdown, count)
}

fn session_to_markdown_full(session: &Session) -> Result<String> {
    let content = std::fs::read_to_string(&session.path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut markdown = String::new();

    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        let mut last_role = String::new();
        for msg in messages {
            let role = msg
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown")
                .to_uppercase();

            let display_role = if role == "ASSISTANT" {
                "GEMINI".to_string()
            } else {
                role
            };

            let mut text = String::new();
            let content_val = msg.get("content").unwrap_or(&serde_json::Value::Null);
            if let Some(t) = content_val.as_str() {
                text = t.to_string();
            } else if let Some(arr) = content_val.as_array() {
                for item in arr {
                    if let Some(t) = item.get("text").and_then(|v| v.as_str()) {
                        text.push_str(t);
                        text.push('\n');
                    }
                }
            }

            if !text.trim().is_empty() {
                if display_role == last_role {
                    markdown.push_str(&format!("{}\n\n", text.trim()));
                } else {
                    markdown.push_str(&format!("## {}\n{}\n\n", display_role, text.trim()));
                    last_role = display_role;
                }
            }
        }
    }
    Ok(markdown)
}

pub fn export_session(session: &Session, output: Option<&Path>) -> Result<PathBuf> {
    let markdown = session_to_markdown_full(session)?;
    let out_path = output.map(|p| p.to_path_buf()).unwrap_or_else(|| {
        let mut p = PathBuf::from(&session.id);
        p.set_extension("md");
        p
    });

    std::fs::write(&out_path, markdown)?;
    Ok(out_path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_markdown_generation_merging() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("s.json");
        let data = r#"{"messages":[{"type":"user","content":"hello"},{"type":"assistant","content":"world"}]}"#;
        fs::write(&path, data).unwrap();

        let session = Session {
            id: "test".into(),
            display_id: "test".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 100,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        };

        let md = session_to_markdown_limited(&session, 10, &PreviewConfig::default()).unwrap();
        assert!(md.contains("USER"));
        assert!(md.contains("hello"));
        assert!(md.contains("GEMINI"));
        assert!(md.contains("world"));
    }

    #[test]
    fn test_large_file_uses_tail_preview_fallback() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("large-tail-preview.json");

        // Create a large file where early bytes don't contain message blocks,
        // but recent bytes do.
        let mut data = "x".repeat((PreviewConfig::default().head_bytes + 128) as usize);
        data.push_str(r#"{"type":"user","content":"tail hello"}"#);
        data.push_str(r#"{"type":"assistant","content":"tail world"}"#);
        fs::write(&path, data.as_bytes()).unwrap();

        let session = Session {
            id: "large-tail".into(),
            display_id: "large-tail".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: data.len() as u64,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        };

        let md = session_to_markdown_limited(&session, 10, &PreviewConfig::default()).unwrap();
        assert!(md.contains("Preview from recent messages"));
        assert!(md.contains("tail hello"));
        assert!(md.contains("tail world"));
    }

    #[test]
    fn test_deep_preview_truncated_by_char_budget() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("deep.json");
        let data = r#"{"messages":[{"type":"user","content":"hello"},{"type":"assistant","content":"world world world"}]}"#;
        fs::write(&path, data).unwrap();

        let session = Session {
            id: "deep".into(),
            display_id: "deep".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: data.len() as u64,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        };

        let preview = PreviewConfig {
            deep_preview_char_budget: 8,
            ..PreviewConfig::default()
        };
        let md = session_to_markdown_deep_limited(&session, 20, &preview).unwrap();
        assert!(md.contains("Deep preview"));
        assert!(md.contains("truncated"));
    }
}
