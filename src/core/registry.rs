use crate::core::scanner::Scanner;
use crate::core::session::Session;
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

    /// Reload sessions by scanning the directory.
    pub fn reload(&mut self) -> Result<()> {
        self.sessions = self.scanner.scan()?;
        Ok(())
    }

    /// Finds a session by its unique ID.
    pub fn find_by_id(&self, id: &str) -> Option<&Session> {
        self.sessions.iter().find(|s| s.id == id)
    }

    /// Finds a session by its name or ID.
    pub fn find(&self, query: &str) -> Option<&Session> {
        self.sessions
            .iter()
            .find(|s| s.id == query || s.name.as_ref().map_or(false, |n| n == query))
    }

    /// Lists all sessions.
    pub fn list(&self) -> &[Session] {
        &self.sessions
    }
}
