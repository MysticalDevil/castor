use crate::core::session::Session;
use crate::core::scanner::Scanner;
use crate::error::Result;
use std::path::Path;

pub struct Registry {
    sessions: Vec<Session>,
    scanner: Scanner,
}

impl Registry {
    pub fn new(base_path: &Path) -> Self {
        Self {
            sessions: Vec::new(),
            scanner: Scanner::new(base_path),
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        self.sessions = self.scanner.scan()?;
        Ok(())
    }

    pub fn find_by_id(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    pub fn find(&self, query: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| {
            s.id == query || s.name.as_ref().map_or(false, |n| n == query)
        })
    }

    pub fn list(&self) -> &[Session] {
        &self.sessions
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;
    use std::fs;

    #[test]
    fn test_registry_find() {
        let tmp = tempdir().unwrap();
        let project_id = "test-proj";
        let project_path = tmp.path().join(project_id);
        let chats_path = project_path.join("chats");
        fs::create_dir_all(&chats_path).unwrap();

        fs::write(chats_path.join("s1.json"), r#"{"messages": [{"type": "user", "content": "Query1"}]}"#).unwrap();

        let mut registry = Registry::new(tmp.path());
        registry.reload().unwrap();

        assert!(registry.find("s1.json").is_some());
        assert!(registry.find("Query1").is_some());
        assert!(registry.find("NonExistent").is_none());
    }
}
