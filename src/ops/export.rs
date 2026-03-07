use crate::core::Session;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub fn session_to_markdown(session: &Session) -> Result<String> {
    let content = std::fs::read_to_string(&session.path)?;
    let json: serde_json::Value = serde_json::from_str(&content)?;
    let mut markdown = format!("# Session: {}\n\n", session.id);

    if let Some(messages) = json.get("messages").and_then(|m| m.as_array()) {
        for msg in messages {
            let role = msg
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
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
    fn test_markdown_generation() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(
            &path,
            r#"{"messages": [{"type": "user", "content": "hello"}]}"#,
        )
        .unwrap();

        let session = Session {
            id: "test".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 0,
            validation_notes: Vec::new(),
        };

        let md = session_to_markdown(&session).unwrap();
        assert!(md.contains("# Session: test"));
        assert!(md.contains("## USER"));
        assert!(md.contains("hello"));
    }

    #[test]
    fn test_export_file_writing() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("s.json");
        fs::write(&path, r#"{"messages": []}"#).unwrap();

        let session = Session {
            id: "test_export".into(),
            project_id: "p".into(),
            host_path: None,
            name: None,
            path,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            size: 0,
            validation_notes: Vec::new(),
        };

        let out_path = tmp.path().join("test.md");
        let result = export_session(&session, Some(&out_path)).unwrap();
        assert!(result.exists());
        assert!(
            fs::read_to_string(result)
                .unwrap()
                .contains("# Session: test_export")
        );
    }
}
