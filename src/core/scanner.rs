use std::path::{Path, PathBuf};
use crate::core::session::Session;
use crate::error::Result;

pub struct Scanner {
    base_path: PathBuf,
}

impl Scanner {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    /// Scans the base directory recursively for sessions.
    pub fn scan(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();

        if !self.base_path.exists() {
            return Ok(sessions);
        }

        // Iterate through project directories in ~/.gemini/tmp/
        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let project_path = entry.path();
            if project_path.is_dir() {
                let project_id = project_path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                // Check for .project_root file
                let project_root_file = project_path.join(".project_root");
                let host_path = if project_root_file.exists() {
                    std::fs::read_to_string(project_root_file)
                        .ok()
                        .map(|s| PathBuf::from(s.trim()))
                } else {
                    None
                };

                let chats_path = project_path.join("chats");
                if chats_path.exists() && chats_path.is_dir() {
                    self.scan_chats_dir(&chats_path, project_id, host_path, &mut sessions)?;
                }
            }
        }

        // Sort by update time descending
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    fn scan_chats_dir(&self, chats_path: &Path, project_id: String, host_path: Option<PathBuf>, sessions: &mut Vec<Session>) -> Result<()> {
        for entry in std::fs::read_dir(chats_path)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.file_name().and_then(|n| n.to_str()).map_or(true, |n| n.starts_with('.')) {
                continue;
            }

            match Session::from_path(&path, project_id.clone(), host_path.clone()) {
                Ok(session) => sessions.push(session),
                Err(_) => continue,
            }
        }
        Ok(())
    }
}
