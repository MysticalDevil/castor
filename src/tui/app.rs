use crate::core::{Registry, Session};
use crate::error::Result;
use crate::ops::Executor;
use std::collections::HashMap;

pub enum InputMode {
    Normal,
    ConfirmDelete,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Selection {
    Project(String),
    Session(String),
}

pub struct App {
    pub registry: Registry,
    pub executor: Executor,
    pub projects: Vec<String>,
    pub sessions_by_project: HashMap<String, Vec<Session>>,
    pub flat_items: Vec<Selection>, // Flattened tree for easy indexing
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(registry: Registry, executor: Executor) -> Self {
        Self {
            registry,
            executor,
            projects: Vec::new(),
            sessions_by_project: HashMap::new(),
            flat_items: Vec::new(),
            selected_index: 0,
            input_mode: InputMode::Normal,
            should_quit: false,
            message: None,
        }
    }

    pub fn reload(&mut self) -> Result<()> {
        self.registry.reload()?;
        self.sessions_by_project.clear();
        self.projects.clear();
        self.flat_items.clear();

        for s in self.registry.list() {
            let proj_id = s.project_id.clone();
            self.sessions_by_project
                .entry(proj_id.clone())
                .or_default()
                .push(s.clone());
        }

        self.projects = self.sessions_by_project.keys().cloned().collect();
        self.projects.sort();

        // Build flat tree view: Project -> [Session, Session...]
        for proj in &self.projects {
            self.flat_items.push(Selection::Project(proj.clone()));
            if let Some(sessions) = self.sessions_by_project.get(proj) {
                for s in sessions {
                    self.flat_items.push(Selection::Session(s.id.clone()));
                }
            }
        }

        if self.selected_index >= self.flat_items.len() && !self.flat_items.is_empty() {
            self.selected_index = self.flat_items.len() - 1;
        }
        Ok(())
    }

    pub fn next(&mut self) {
        if !self.flat_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.flat_items.len();
        }
    }

    pub fn previous(&mut self) {
        if !self.flat_items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.flat_items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn get_selected_session(&self) -> Option<&Session> {
        if let Some(Selection::Session(id)) = self.flat_items.get(self.selected_index) {
            self.registry.find_by_id(id)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_app_tree_navigation() {
        let tmp = tempdir().unwrap();
        let project_path = tmp.path().join("proj1/chats");
        fs::create_dir_all(&project_path).unwrap();
        fs::write(
            project_path.join("session-2026-03-08T12-00-aaaa1111.json"),
            "{}",
        )
        .unwrap();

        let mut registry = Registry::new(tmp.path(), &tmp.path().join("cache.json"));
        registry.reload().unwrap();

        let executor = Executor::new(Config {
            gemini_sessions_path: tmp.path().to_path_buf(),
            trash_path: tmp.path().join("trash"),
            audit_path: tmp.path().join("audit"),
            cache_path: tmp.path().join("cache"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
        });
        let mut app = App::new(registry, executor);
        app.reload().unwrap();

        // Items should be: [Project("proj1"), Session("session-...-aaaa1111.json")]
        assert_eq!(app.flat_items.len(), 2);
        assert!(matches!(app.flat_items[0], Selection::Project(_)));
        assert!(matches!(app.flat_items[1], Selection::Session(_)));

        assert_eq!(app.selected_index, 0);
        app.next();
        assert_eq!(app.selected_index, 1);

        let sel = app.get_selected_session().unwrap();
        assert!(sel.id.contains("aaaa1111"));
    }
}
