use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Default)]
pub enum SessionHealth {
    #[default]
    Unknown,
    Ok,
    Warn,  // Orphaned (no host)
    Error, // Corrupted/Missing
    Risk,  // Anomaly
}

impl fmt::Display for SessionHealth {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unknown => write!(f, "UNKNOWN"),
            Self::Ok => write!(f, "OK"),
            Self::Warn => write!(f, "WARN"),
            Self::Error => write!(f, "ERROR"),
            Self::Risk => write!(f, "RISK"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    pub id: String,
    pub display_id: String, // Pre-calculated for rendering performance
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

impl Session {
    pub fn from_path(
        path: &std::path::Path,
        project_id: String,
        host_path: Option<PathBuf>,
    ) -> Result<Self> {
        let metadata = std::fs::metadata(path)?;
        let updated_at: DateTime<Utc> = metadata.modified()?.into();
        let id = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string();

        // Pre-calculate short ID: session-2026-03-08T12-00-abcdef01.json -> abcdef01
        let display_id = id
            .strip_suffix(".json")
            .unwrap_or(&id)
            .split('-')
            .next_back()
            .unwrap_or(&id)
            .to_string();

        Ok(Self {
            id,
            display_id,
            project_id,
            host_path,
            name: None,
            path: path.to_path_buf(),
            created_at: updated_at,
            updated_at,
            size: metadata.len(),
            health: SessionHealth::Unknown,
            validation_notes: Vec::new(),
        })
    }

    pub fn deep_validate(&mut self) {
        self.validation_notes.clear();

        if self.size == 0 {
            self.health = SessionHealth::Error;
            self.validation_notes.push("File is empty".into());
            return;
        }

        let content = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => {
                self.health = SessionHealth::Error;
                self.validation_notes.push("Unreadable file".into());
                return;
            }
        };

        let json: serde_json::Value = match serde_json::from_str(&content) {
            Ok(j) => j,
            Err(_) => {
                self.health = SessionHealth::Error;
                self.validation_notes.push("Invalid JSON structure".into());
                return;
            }
        };

        if self.name.is_none()
            && let Some(messages) = json.get("messages").and_then(|m| m.as_array())
        {
            for msg in messages {
                if msg.get("type").and_then(|t| t.as_str()) == Some("user")
                    && let Some(c) = msg.get("content").and_then(|c| c.as_str())
                {
                    let first_line = c.lines().next().unwrap_or("");
                    let truncated: String = first_line.chars().take(50).collect();
                    self.name = Some(truncated);
                    break;
                }
            }
        }

        if let Some(host) = &self.host_path {
            if !host.exists() {
                self.health = SessionHealth::Warn;
                self.validation_notes
                    .push("Host project path missing".into());
            } else {
                self.health = SessionHealth::Ok;
            }
        }

        if self.size > 50 * 1024 * 1024 {
            self.health = SessionHealth::Risk;
            self.validation_notes
                .push("Extremely large session (>50MB)".into());
        }

        if self.updated_at > Utc::now() + chrono::Duration::hours(1) {
            self.health = SessionHealth::Risk;
            self.validation_notes
                .push("Temporal anomaly: future update time".into());
        }
    }

    pub fn get_content(&self) -> Result<String> {
        Ok(std::fs::read_to_string(&self.path)?)
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
        let path = tmp.path().join("session-2026-03-08T12-00-abcdef01.json");
        fs::write(&path, "{}").unwrap();

        let s = Session::from_path(&path, "p".into(), None).unwrap();
        assert_eq!(s.display_id, "abcdef01");
        assert_eq!(s.health, SessionHealth::Unknown);
    }
}
