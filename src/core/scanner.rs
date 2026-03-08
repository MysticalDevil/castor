use crate::core::session::Session;
use crate::error::Result;
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

pub struct Scanner {
    pub base_path: PathBuf,
}

impl Scanner {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    /// Optimized scan that discovers sessions using parallel I/O.
    pub fn scan(&self) -> Result<Vec<Session>> {
        if !self.base_path.exists() {
            return Ok(Vec::new());
        }

        let project_dirs: Vec<PathBuf> = fs::read_dir(&self.base_path)?
            .flatten()
            .filter(|e| e.path().is_dir())
            .map(|e| e.path())
            .collect();

        let results: Vec<Vec<Session>> = project_dirs
            .into_par_iter()
            .map(|project_path| {
                let mut sessions = Vec::new();
                let project_id = project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

                let project_root_file = project_path.join(".project_root");
                let host_path = fs::read_to_string(project_root_file)
                    .ok()
                    .map(|s| PathBuf::from(s.trim()));

                let chats_path = project_path.join("chats");
                if chats_path.exists()
                    && chats_path.is_dir()
                    && let Ok(entries) = fs::read_dir(chats_path)
                {
                    for chat_entry in entries.flatten() {
                        let path = chat_entry.path();
                        if path.extension().is_some_and(|ext| ext == "json")
                            && let Ok(metadata) = chat_entry.metadata()
                        {
                            let updated_at = metadata
                                .modified()
                                .unwrap_or_else(|_| std::time::SystemTime::now())
                                .into();

                            let id = path
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown")
                                .to_string();

                            let display_id = id
                                .strip_suffix(".json")
                                .unwrap_or(&id)
                                .split('-')
                                .next_back()
                                .unwrap_or(&id)
                                .to_string();

                            sessions.push(Session {
                                id,
                                display_id,
                                project_id: project_id.clone(),
                                host_path: host_path.clone(),
                                name: None,
                                path,
                                created_at: updated_at,
                                updated_at,
                                size: metadata.len(),
                                health: crate::core::session::SessionHealth::Unknown,
                                validation_notes: Vec::new(),
                            });
                        }
                    }
                }
                sessions
            })
            .collect();

        Ok(results.into_iter().flatten().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scanner_empty() {
        let tmp = tempdir().expect("create tempdir");
        let scanner = Scanner::new(tmp.path());
        let results = scanner.scan().expect("scan empty tree");
        assert!(results.is_empty());
    }

    #[test]
    fn test_scanner_with_data() {
        let tmp = tempdir().expect("create tempdir");
        let project_dir = tmp.path().join("proj1");
        let chats_dir = project_dir.join("chats");
        fs::create_dir_all(&chats_dir).expect("create chats dir");
        fs::write(chats_dir.join("session-1.json"), "{}").expect("write session fixture");
        fs::write(project_dir.join(".project_root"), "/home/user/proj1")
            .expect("write project root marker");

        let scanner = Scanner::new(tmp.path());
        let results = scanner.scan().expect("scan tree with one session");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project_id, "proj1");
        assert_eq!(
            results[0].host_path,
            Some(PathBuf::from("/home/user/proj1"))
        );
    }
}
