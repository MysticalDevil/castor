use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use chrono::{DateTime, Utc};
use crate::error::{Result, CastorError};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Session {
    /// Unique identifier for the session (filename with extension).
    pub id: String,

    /// The project hash/ID this session belongs to (directory name in tmp).
    pub project_id: String,

    /// The actual host project path (from .project_root if available).
    pub host_path: Option<PathBuf>,

    /// Human-readable name (extracted from the first user message).
    pub name: Option<String>,

    /// Full path to the session file or directory.
    pub path: PathBuf,

    /// Creation time.
    pub created_at: DateTime<Utc>,

    /// Last update time.
    pub updated_at: DateTime<Utc>,

    /// Size in bytes.
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
    /// Parses a session from its path, project ID, and host path.
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
