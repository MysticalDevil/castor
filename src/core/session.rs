use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::error::{Result, CastorError};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionHealth {
    Ok,
    Warn,
    Error,
}

impl std::fmt::Display for SessionHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionHealth::Ok => write!(f, "OK"),
            SessionHealth::Warn => write!(f, "WARN"),
            SessionHealth::Error => write!(f, "ERROR"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub project_id: String,
    pub host_path: Option<PathBuf>,
    pub name: Option<String>,
    pub path: PathBuf,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub size: u64,
}

#[derive(Deserialize)]
struct GeminiSessionFile {
    messages: Option<Vec<GeminiMessage>>,
}

#[derive(Deserialize)]
struct GeminiMessage {
    #[serde(rename = "type")]
    msg_type: String,
    content: serde_json::Value,
}

impl Session {
    pub fn from_path(path: &Path, project_id: String, host_path: Option<PathBuf>) -> Result<Self> {
        let id = path.file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CastorError::InvalidSession("Invalid session filename".to_string()))?
            .to_string();

        let metadata = std::fs::metadata(path)?;
        let created_at: DateTime<Utc> = metadata.created().unwrap_or_else(|_| metadata.modified().unwrap()).into();
        let updated_at: DateTime<Utc> = metadata.modified().unwrap().into();
        
        let size = if path.is_dir() {
            Self::calculate_dir_size(path)?
        } else {
            metadata.len()
        };

        let name = Self::extract_name(path);

        Ok(Self {
            id,
            project_id,
            host_path,
            name,
            path: path.to_path_buf(),
            created_at,
            updated_at,
            size,
        })
    }

    /// Checks the health of the session.
    pub fn check_health(&self) -> SessionHealth {
        // 1. Check if session file itself exists
        if !self.path.exists() {
            return SessionHealth::Error;
        }

        // 2. Check if file is empty (usually means corrupted or interrupted)
        if self.size == 0 {
            return SessionHealth::Error;
        }

        // 3. Check if host exists (if known)
        if let Some(host) = &self.host_path {
            if !host.exists() {
                return SessionHealth::Warn;
            }
        }

        SessionHealth::Ok
    }

    fn extract_name(path: &Path) -> Option<String> {
        if path.is_dir() {
            return None;
        }

        let file = std::fs::File::open(path).ok()?;
        let reader = std::io::BufReader::new(file);
        let session_data: GeminiSessionFile = serde_json::from_reader(reader).ok()?;

        if let Some(messages) = session_data.messages {
            for msg in messages {
                if msg.msg_type == "user" {
                    let raw_text = if let Some(text) = msg.content.as_str() {
                        text
                    } else if let Some(arr) = msg.content.as_array() {
                        arr.first().and_then(|f| f.get("text")).and_then(|v| v.as_str()).unwrap_or("")
                    } else {
                        ""
                    };

                    if !raw_text.is_empty() {
                        let single_line = raw_text.replace('\n', " ").replace('\r', " ");
                        return Some(single_line.trim().chars().take(100).collect::<String>());
                    }
                }
            }
        }

        None
    }

    fn calculate_dir_size(path: &Path) -> Result<u64> {
        let mut total_size = 0;
        for entry in walkdir::WalkDir::new(path) {
            let entry = entry?;
            if entry.file_type().is_file() {
                total_size += entry.metadata()?.len();
            }
        }
        Ok(total_size)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_extract_name_simple() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("session.json");
        let data = r#"{"messages": [{"type": "user", "content": "Hello world"}]}"#;
        fs::write(&file_path, data).unwrap();

        let name = Session::extract_name(&file_path);
        assert_eq!(name, Some("Hello world".to_string()));
    }

    #[test]
    fn test_session_health() {
        let tmp = tempdir().unwrap();
        let file_path = tmp.path().join("s.json");
        fs::write(&file_path, "{}").unwrap();

        let mut session = Session::from_path(&file_path, "p".into(), None).unwrap();
        assert_eq!(session.check_health(), SessionHealth::Ok);

        // Host not found
        session.host_path = Some(PathBuf::from("/non/existent/path"));
        assert_eq!(session.check_health(), SessionHealth::Warn);

        // Session file missing
        fs::remove_file(&file_path).unwrap();
        assert_eq!(session.check_health(), SessionHealth::Error);
    }
}
