use crate::core::{Registry, Session};
use crate::error::Result;
use crate::ops::Executor;
use std::collections::HashMap;

pub enum InputMode {
    Normal,
    ConfirmDelete,
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub enum GroupingMode {
    #[default]
    Host,
    Month,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Selection {
    Group(String),
    Session(String),
}

pub struct App {
    pub registry: Registry,
    pub executor: Executor,
    pub groups: Vec<String>,
    pub sessions_by_group: HashMap<String, Vec<Session>>,
    pub flat_items: Vec<Selection>,
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub grouping_mode: GroupingMode,
    pub should_quit: bool,
    pub message: Option<String>,

    // Performance: Cache for the currently selected session's preview
    pub current_preview: Option<String>,
    pub last_selected_id: Option<String>,
}

impl App {
    pub fn new(registry: Registry, executor: Executor) -> Self {
        Self {
            registry,
            executor,
            groups: Vec::new(),
            sessions_by_group: HashMap::new(),
            flat_items: Vec::new(),
            selected_index: 0,
            input_mode: InputMode::Normal,
            grouping_mode: GroupingMode::Host,
            should_quit: false,
            message: None,
            current_preview: None,
            last_selected_id: None,
        }
    }

    pub fn toggle_grouping(&mut self) -> Result<()> {
        self.grouping_mode = match self.grouping_mode {
            GroupingMode::Host => GroupingMode::Month,
            GroupingMode::Month => GroupingMode::Host,
        };
        self.reload()
    }

    pub fn reload(&mut self) -> Result<()> {
        self.registry.reload()?;
        self.sessions_by_group.clear();
        self.groups.clear();
        self.flat_items.clear();

        let home = std::env::var("HOME").ok();

        for s in self.registry.list() {
            let group_key = match self.grouping_mode {
                GroupingMode::Host => {
                    if let Some(path) = &s.host_path {
                        crate::utils::fs::format_host(path, home.as_deref())
                    } else {
                        s.project_id.clone()
                    }
                }
                GroupingMode::Month => s.updated_at.format("%Y-%m").to_string(),
            };

            self.sessions_by_group
                .entry(group_key)
                .or_default()
                .push(s.clone());
        }

        self.groups = self.sessions_by_group.keys().cloned().collect();
        self.groups.sort_by(|a, b| match self.grouping_mode {
            GroupingMode::Month => b.cmp(a),
            GroupingMode::Host => a.cmp(b),
        });

        for group in &self.groups {
            self.flat_items.push(Selection::Group(group.clone()));
            if let Some(sessions) = self.sessions_by_group.get(group) {
                for s in sessions {
                    self.flat_items.push(Selection::Session(s.id.clone()));
                }
            }
        }

        if self.selected_index >= self.flat_items.len() && !self.flat_items.is_empty() {
            self.selected_index = self.flat_items.len() - 1;
        }

        self.update_preview();
        Ok(())
    }

    pub fn next(&mut self) {
        if !self.flat_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.flat_items.len();
            self.update_preview();
        }
    }

    pub fn previous(&mut self) {
        if !self.flat_items.is_empty() {
            if self.selected_index == 0 {
                self.selected_index = self.flat_items.len() - 1;
            } else {
                self.selected_index -= 1;
            }
            self.update_preview();
        }
    }

    /// Updates the preview cache if the selection has changed
    fn update_preview(&mut self) {
        let current_id =
            if let Some(Selection::Session(id)) = self.flat_items.get(self.selected_index) {
                Some(id.clone())
            } else {
                None
            };

        if current_id != self.last_selected_id {
            if let Some(id) = &current_id {
                if let Some(session) = self.registry.find_by_id(id) {
                    // Perform on-demand deep validation and markdown generation
                    let mut s_clone = session.clone();
                    s_clone.deep_validate();
                    self.current_preview = crate::ops::export::session_to_markdown(&s_clone).ok();
                } else {
                    self.current_preview = None;
                }
            } else {
                self.current_preview = None;
            }
            self.last_selected_id = current_id;
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
    fn test_app_grouping_toggle() {
        let tmp = tempdir().unwrap();
        let project_path = tmp.path().join("proj1/chats");
        fs::create_dir_all(&project_path).unwrap();
        fs::write(
            project_path.join("session-2026-03-08T12-00-aaaa1111.json"),
            "{}",
        )
        .unwrap();

        let registry = Registry::new(tmp.path(), &tmp.path().join("cache.json"));
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

        assert_eq!(app.grouping_mode, GroupingMode::Host);
        app.toggle_grouping().unwrap();
        assert_eq!(app.grouping_mode, GroupingMode::Month);
        assert!(app.groups.contains(&"2026-03".to_string()));
    }
}
