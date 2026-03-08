use crate::core::Session;
use crate::error::Result;
use regex::Regex;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static MSG_BLOCK_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"\{"type"\s*:\s*"([^"]+)"\s*,\s*"content"\s*:\s*([^}]+)\}"#).unwrap()
});

pub fn session_to_markdown(session: &Session) -> Result<String> {
    session_to_markdown_limited(session, usize::MAX)
}

pub fn session_to_markdown_limited(session: &Session, limit: usize) -> Result<String> {
    let file = std::fs::File::open(&session.path)?;
    let mut reader = BufReader::new(file);

    let mut preview_buffer_size = 512 * 1024;
    if session.size < preview_buffer_size {
        preview_buffer_size = session.size;
    }

    let mut buffer = vec![0; preview_buffer_size as usize];
    let n = reader.read(&mut buffer)?;
    buffer.truncate(n);
    let content = String::from_utf8_lossy(&buffer);

    let mut markdown = String::new();
    let mut last_role = String::new();
    let mut count = 0;
    let re_text = Regex::new(r#""text"\s*:\s*"([^"]+)""#).unwrap();

    for caps in MSG_BLOCK_REGEX.captures_iter(&content) {
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
            for t_cap in re_text.captures_iter(content_raw) {
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

    if count == 0 && session.size > 0 {
        if session.size < 2 * 1024 * 1024 {
            return session_to_markdown_full(session);
        }
        markdown.push_str("-- [ Large session: Use `castor cat` to view full content ] --");
    } else if count >= limit {
        markdown.push_str("\n\n-- [ Preview limited to first few messages ] --");
    }

    Ok(markdown)
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

        let md = session_to_markdown_limited(&session, 10).unwrap();
        assert!(md.contains("USER"));
        assert!(md.contains("hello"));
        assert!(md.contains("GEMINI"));
        assert!(md.contains("world"));
    }
}
