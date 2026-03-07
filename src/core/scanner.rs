use crate::core::session::Session;
use crate::error::Result;
use std::path::{Path, PathBuf};

pub struct Scanner {
    base_path: PathBuf,
}

impl Scanner {
    pub fn new(base_path: &Path) -> Self {
        Self {
            base_path: base_path.to_path_buf(),
        }
    }

    pub fn scan(&self) -> Result<Vec<Session>> {
        let mut sessions = Vec::new();

        if !self.base_path.exists() {
            return Ok(sessions);
        }

        for entry in std::fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let project_path = entry.path();
            if project_path.is_dir() {
                let project_id = project_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("unknown")
                    .to_string();

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

        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    fn scan_chats_dir(
        &self,
        chats_path: &Path,
        project_id: String,
        host_path: Option<PathBuf>,
        sessions: &mut Vec<Session>,
    ) -> Result<()> {
        for entry in std::fs::read_dir(chats_path)? {
            let entry = entry?;
            let path = entry.path();

            if path
                .file_name()
                .and_then(|n| n.to_str())
                .is_none_or(|n| n.starts_with('.'))
            {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_scanner_empty() {
        let tmp = tempdir().unwrap();
        let scanner = Scanner::new(tmp.path());
        let results = scanner.scan().unwrap();
        assert!(results.is_empty());
    }

    #[test]
    fn test_scanner_with_data() {
        let tmp = tempdir().unwrap();
        let project_id = "test-proj";
        let project_path = tmp.path().join(project_id);
        let chats_path = project_path.join("chats");
        fs::create_dir_all(&chats_path).unwrap();

        fs::write(project_path.join(".project_root"), "/home/user/code").unwrap();
        fs::write(chats_path.join("s1.json"), r#"{"messages": []}"#).unwrap();

        let scanner = Scanner::new(tmp.path());
        let results = scanner.scan().unwrap();

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].project_id, project_id);
        assert_eq!(
            results[0].host_path.as_ref().unwrap().to_str().unwrap(),
            "/home/user/code"
        );
    }
}
