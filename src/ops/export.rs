use crate::core::Session;
use crate::error::Result;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

pub fn session_to_markdown(session: &Session) -> Result<String> {
    session_to_markdown_limited(session, usize::MAX)
}

/// Specialized version for TUI previews that uses buffered reading and limits data processed.
pub fn session_to_markdown_limited(session: &Session, limit: usize) -> Result<String> {
    let file = std::fs::File::open(&session.path)?;
    let mut reader = BufReader::new(file);

    // 1MB is a good balance for background parsing while staying responsive.
    let preview_threshold = 1024 * 1024;

    let content = if limit < 100 && session.size > preview_threshold {
        let mut buffer = vec![0; preview_threshold as usize];
        let n = reader.read(&mut buffer)?;
        buffer.truncate(n);
        String::from_utf8_lossy(&buffer).into_owned()
    } else {
        let mut s = String::new();
        reader.read_to_string(&mut s)?;
        s
    };

    // Attempt to parse.
    let json_val: serde_json::Value = if content.ends_with('}') {
        serde_json::from_str(&content).unwrap_or(serde_json::Value::Null)
    } else {
        // Best-effort partial JSON fix for truncated data
        let mut fixed = content.clone();
        if !fixed.ends_with(']') {
            fixed.push_str("]}");
        }
        if !fixed.ends_with('}') {
            fixed.push('}');
        }
        // Try parsing the fixed version, fallback to Null if still broken
        serde_json::from_str(&fixed).unwrap_or(serde_json::Value::Null)
    };

    let mut markdown = String::new();
    if let Some(messages) = json_val.get("messages").and_then(|m| m.as_array()) {
        let mut last_role = String::new();
        let mut count = 0;

        for msg in messages {
            if count >= limit {
                markdown.push_str("\n\n-- [ Preview limited to first few messages ] --");
                break;
            }

            let role = msg
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
            let display_role = match role {
                "user" => "USER",
                "assistant" => "GEMINI",
                other => other,
            }
            .to_uppercase();

            let mut text_parts = Vec::new();
            let content_val = msg.get("content").unwrap_or(&serde_json::Value::Null);

            if let Some(text) = content_val.as_str() {
                if !text.trim().is_empty() {
                    text_parts.push(text.to_string());
                }
            } else if let Some(arr) = content_val.as_array() {
                for item in arr {
                    if let Some(text) = item.get("text").and_then(|v| v.as_str())
                        && !text.trim().is_empty()
                    {
                        text_parts.push(text.to_string());
                    }
                }
            }

            if !text_parts.is_empty() {
                let joined_text = text_parts.join("\n\n");
                count += 1;

                if display_role == last_role {
                    markdown.push_str(&format!("{}\n\n", joined_text));
                } else {
                    markdown.push_str(&format!("## {}\n{}\n\n", display_role, joined_text));
                    last_role = display_role;
                }
            }
        }
    } else if session.size > preview_threshold {
        // If we failed to parse even with 1MB, something is weird or the header is too deep.
        markdown.push_str("-- [ Large file: content header is too deep for quick preview ] --\n");
        markdown.push_str("-- [ Use `castor cat` in CLI to view full content ] --");
    } else {
        markdown.push_str("-- [ Error: Session content is unreadable or malformed ] --");
    }

    Ok(markdown)
}

pub fn export_session(session: &Session, output: Option<&Path>) -> Result<PathBuf> {
    let markdown = session_to_markdown(session)?;
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
        let data = r#"{
            "messages": [
                {"type": "user", "content": "hello"},
                {"type": "assistant", "content": "part 1"},
                {"type": "assistant", "content": "part 2"}
            ]
        }"#;
        fs::write(&path, data).unwrap();

        let session = Session {
            id: "test".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 0,
            health: crate::core::session::SessionHealth::Ok,
            validation_notes: Vec::new(),
        };

        let md = session_to_markdown(&session).unwrap();
        assert!(md.contains("## USER\nhello"));
        assert!(md.contains("## GEMINI\npart 1\n\npart 2"));
    }
}
