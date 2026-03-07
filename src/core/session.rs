use crate::error::{CastorError, Result};
use chrono::{DateTime, NaiveDateTime, Utc};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

static SESSION_ID_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    // Standard Gemini pattern: session-YYYY-MM-DDTHH-MM-hash.json
    Regex::new(r"^session-(\d{4}-\d{2}-\d{2})T(\d{2})-(\d{2})-([a-zA-Z0-9]{8})\.json$").unwrap()
});

const MAX_SESSION_SIZE_BYTES: u64 = 50 * 1024 * 1024; // 50MB anomaly threshold

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum SessionHealth {
    Unknown, // Not yet validated
    Ok,
    Warn,  // Orphaned (Host missing)
    Error, // Corrupted (Structural/IO)
    Risk,  // Security/Temporal Anomaly
}

impl std::fmt::Display for SessionHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionHealth::Unknown => write!(f, "UNKNOWN"),
            SessionHealth::Ok => write!(f, "OK"),
            SessionHealth::Warn => write!(f, "WARN"),
            SessionHealth::Error => write!(f, "ERROR"),
            SessionHealth::Risk => write!(f, "RISK"),
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
    pub health: SessionHealth,
    pub validation_notes: Vec<String>,
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
    /// Shallow load of a session (metadata only)
    pub fn from_path(path: &Path, project_id: String, host_path: Option<PathBuf>) -> Result<Self> {
        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| CastorError::InvalidSession("Invalid session filename".to_string()))?
            .to_string();

        let metadata = std::fs::metadata(path)?;
        let created_at: DateTime<Utc> = metadata
            .created()
            .unwrap_or_else(|_| metadata.modified().unwrap())
            .into();
        let updated_at: DateTime<Utc> = metadata.modified().unwrap().into();
        let size = if path.is_dir() {
            Self::calculate_dir_size(path)?
        } else {
            metadata.len()
        };

        let mut s = Self {
            id,
            project_id,
            host_path,
            name: None,
            path: path.to_path_buf(),
            created_at,
            updated_at,
            size,
            health: SessionHealth::Unknown,
            validation_notes: Vec::new(),
        };
        s.health = s.calculate_health();
        Ok(s)
    }

    /// Performs deep validation if not already done.
    pub fn deep_validate(&mut self) {
        if self.health == SessionHealth::Error || self.health == SessionHealth::Risk {
            // Already failed
            return;
        }

        let mut notes = Vec::new();
        self.name = Self::extract_and_validate(&self.path, &mut notes);
        self.validation_notes = notes;
        self.health = self.calculate_health();
    }

    pub fn calculate_health(&self) -> SessionHealth {
        // 1. Critical Errors
        if !self.path.exists() || self.size == 0 {
            return SessionHealth::Error;
        }
        if self
            .validation_notes
            .iter()
            .any(|n| n.contains("Structural"))
        {
            return SessionHealth::Error;
        }

        // 2. Risks
        if !SESSION_ID_REGEX.is_match(&self.id) {
            return SessionHealth::Risk;
        }

        if let Some(caps) = SESSION_ID_REGEX.captures(&self.id) {
            let date_str = format!("{} {}:{}", &caps[1], &caps[2], &caps[3]);
            if let Ok(id_date) = NaiveDateTime::parse_from_str(&date_str, "%Y-%m-%d %H:%M")
                && id_date.and_utc() > self.updated_at + chrono::Duration::hours(25)
            {
                return SessionHealth::Risk;
            }
        }

        if self.size > MAX_SESSION_SIZE_BYTES {
            return SessionHealth::Risk;
        }

        // 3. Warnings (Host check is shallow)
        if self.host_path.as_ref().is_some_and(|h| !h.exists()) {
            return SessionHealth::Warn;
        }

        // If we have any structural or content notes but no name, it might be an error or warn
        if !self.validation_notes.is_empty() && self.name.is_none() {
            if self
                .validation_notes
                .iter()
                .any(|n| n.contains("Structural"))
            {
                return SessionHealth::Error;
            }
            return SessionHealth::Warn;
        }

        // If validation hasn't run yet, return Unknown instead of Ok
        if self.name.is_none() {
            return SessionHealth::Unknown;
        }

        SessionHealth::Ok
    }

    fn extract_and_validate(path: &Path, notes: &mut Vec<String>) -> Option<String> {
        if path.is_dir() {
            return None;
        }

        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => {
                notes.push("IO: Could not open file".into());
                return None;
            }
        };

        let reader = std::io::BufReader::new(file);
        let session_data: GeminiSessionFile = match serde_json::from_reader(reader) {
            Ok(d) => d,
            Err(e) => {
                notes.push(format!("Structural: Invalid JSON ({})", e));
                return None;
            }
        };

        let messages = match session_data.messages {
            Some(m) => m,
            None => {
                notes.push("Structural: Missing 'messages' field".into());
                return None;
            }
        };

        for msg in messages {
            if msg.msg_type == "user" {
                let raw_text = if let Some(text) = msg.content.as_str() {
                    text
                } else if let Some(arr) = msg.content.as_array() {
                    arr.first()
                        .and_then(|f| f.get("text"))
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                } else {
                    ""
                };

                if !raw_text.is_empty() {
                    let single_line = raw_text.replace(['\n', '\r'], " ");
                    return Some(single_line.trim().chars().take(100).collect::<String>());
                }
            }
        }

        notes.push("Content: No user messages found".into());
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
    fn test_lazy_loading() {
        let tmp = tempdir().unwrap();
        let path = tmp.path().join("session-2026-03-08T12-00-abcdef12.json");
        fs::write(&path, r#"{"messages": [{"type":"user","content":"hi"}]}"#).unwrap();

        let mut s = Session::from_path(&path, "p".into(), None).unwrap();
        assert_eq!(s.health, SessionHealth::Unknown);
        assert!(s.name.is_none());

        s.deep_validate();
        assert_eq!(s.health, SessionHealth::Ok);
        assert_eq!(s.name, Some("hi".into()));
    }

    #[test]
    fn test_deep_validation() {
        let tmp = tempdir().unwrap();

        // 1. Structural Error
        let path_no_msg = tmp.path().join("session-2026-03-08T12-00-abcdef12.json");
        fs::write(&path_no_msg, r#"{"other": []}"#).unwrap();
        let mut s_no_msg = Session::from_path(&path_no_msg, "p".into(), None).unwrap();
        s_no_msg.deep_validate();
        assert_eq!(s_no_msg.health, SessionHealth::Error);

        // 2. Temporal Risk: ID is from far future
        let path_future = tmp.path().join("session-2099-01-01T12-00-abcdef12.json");
        fs::write(
            &path_future,
            r#"{"messages": [{"type":"user","content":"hi"}]}"#,
        )
        .unwrap();
        let s_future = Session::from_path(&path_future, "p".into(), None).unwrap();
        assert_eq!(s_future.calculate_health(), SessionHealth::Risk);
    }
}
