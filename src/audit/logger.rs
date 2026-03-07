use crate::error::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum OperationType {
    SoftDelete,
    HardDelete,
    Restore,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuditEntry {
    pub batch_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub op_type: OperationType,
    pub session_id: String,
    pub original_path: PathBuf,
    pub target_path: Option<PathBuf>,
}

pub struct AuditLogger {
    pub log_path: PathBuf,
}

impl AuditLogger {
    pub fn new(audit_path: &Path) -> Self {
        Self {
            log_path: audit_path.join("audit.jsonl"),
        }
    }

    /// Appends a new audit entry to the log.
    pub fn log(&self, entry: &AuditEntry) -> Result<()> {
        let json = serde_json::to_string(entry)?;
        use std::fs::OpenOptions;
        use std::io::Write;

        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)?;

        writeln!(file, "{}", json)?;
        Ok(())
    }

    /// Loads all audit entries.
    pub fn load_history(&self) -> Result<Vec<AuditEntry>> {
        if !self.log_path.exists() {
            return Ok(Vec::new());
        }

        let content = std::fs::read_to_string(&self.log_path)?;
        let entries: std::result::Result<Vec<AuditEntry>, serde_json::Error> = content
            .lines()
            .map(|line| serde_json::from_str(line))
            .collect();

        Ok(entries?)
    }
}
