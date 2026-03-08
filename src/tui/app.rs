use crate::core::{Registry, Session};
use crate::error::Result;
use crate::ops::Executor;
use ratatui::widgets::ListState;
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
    pub list_state: ListState,
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
            list_state: ListState::default(),
            input_mode: InputMode::Normal,
            grouping_mode: GroupingMode::Host,
            should_quit: false,
            message: None,
            current_preview: None,
            last_selected_id: None,
        }
    }

    /// Incremental loading: add a batch of sessions and rebuild the tree view
    pub fn add_sessions(&mut self, sessions: Vec<Session>) -> Result<()> {
        let home = std::env::var("HOME").ok();

        for s in sessions {
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

            // Add to registry and local grouping
            self.registry.sessions.push(s.clone());
            self.sessions_by_group.entry(group_key).or_default().push(s);
        }

        self.rebuild_tree();
        Ok(())
    }

    fn rebuild_tree(&mut self) {
        self.flat_items.clear();
        self.groups = self.sessions_by_group.keys().cloned().collect();

        // Sort groups: Month desc, Host asc
        self.groups.sort_by(|a, b| match self.grouping_mode {
            GroupingMode::Month => b.cmp(a),
            GroupingMode::Host => a.cmp(b),
        });

        for group in &self.groups {
            self.flat_items.push(Selection::Group(group.clone()));
            if let Some(sessions) = self.sessions_by_group.get(group) {
                // Sort sessions within group by updated_at desc
                let mut sorted_sessions = sessions.clone();
                sorted_sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
                for s in sorted_sessions {
                    self.flat_items.push(Selection::Session(s.id.clone()));
                }
            }
        }

        // Maintain selection if possible
        if self.list_state.selected().is_none() && !self.flat_items.is_empty() {
            self.list_state.select(Some(0));
            if matches!(self.flat_items[0], Selection::Group(_)) {
                self.next();
            }
        }
    }

    pub fn toggle_grouping(&mut self) -> Result<()> {
        self.grouping_mode = match self.grouping_mode {
            GroupingMode::Host => GroupingMode::Month,
            GroupingMode::Month => GroupingMode::Host,
        };
        // When toggling, we need a full regroup of all already loaded sessions
        let sessions = self.registry.list().to_vec();
        self.sessions_by_group.clear();
        self.registry.sessions.clear();
        self.add_sessions(sessions)
    }

    pub fn reload(&mut self) -> Result<()> {
        // Full reset for a clean reload
        self.registry.sessions.clear();
        self.sessions_by_group.clear();
        self.rebuild_tree();
        Ok(())
    }

    pub fn next(&mut self) {
        if self.flat_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let mut i = current;
        loop {
            i = (i + 1) % self.flat_items.len();
            if matches!(self.flat_items[i], Selection::Session(_)) {
                self.list_state.select(Some(i));
                break;
            }
            if i == current {
                break;
            }
        }
        self.update_selection_id();
    }

    pub fn previous(&mut self) {
        if self.flat_items.is_empty() {
            return;
        }
        let current = self.list_state.selected().unwrap_or(0);
        let mut i = current;
        loop {
            if i == 0 {
                i = self.flat_items.len() - 1;
            } else {
                i -= 1;
            }
            if matches!(self.flat_items[i], Selection::Session(_)) {
                self.list_state.select(Some(i));
                break;
            }
            if i == current {
                break;
            }
        }
        self.update_selection_id();
    }

    fn update_selection_id(&mut self) {
        let current_id = if let Some(idx) = self.list_state.selected() {
            if let Some(Selection::Session(id)) = self.flat_items.get(idx) {
                Some(id.clone())
            } else {
                None
            }
        } else {
            None
        };

        if current_id != self.last_selected_id {
            self.current_preview = None; // Show "Loading..."
            self.last_selected_id = current_id;
        }
    }

    pub fn get_selected_session(&self) -> Option<&Session> {
        if let Some(idx) = self.list_state.selected()
            && let Some(Selection::Session(id)) = self.flat_items.get(idx)
        {
            return self.registry.find_by_id(id);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_app_incremental_loading() {
        let tmp = tempdir().unwrap();
        let project_path = tmp.path().join("proj1/chats");
        fs::create_dir_all(&project_path).unwrap();
        let s_path = project_path.join("session-2026-03-08T12-00-aaaa1111.json");
        fs::write(&s_path, "{}").unwrap();

        let registry = Registry::new(tmp.path(), &tmp.path().join("cache.json"));
        let executor = Executor::new(Config {
            gemini_sessions_path: tmp.path().to_path_buf(),
            trash_path: tmp.path().join("trash"),
            audit_path: tmp.path().join("audit"),
            cache_path: tmp.path().join("cache"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
            theme: crate::tui::theme::ThemeConfig::default(),
        });
        let mut app = App::new(registry, executor);

        let session = Session::from_path(&s_path, "proj1".into(), None).unwrap();
        app.add_sessions(vec![session]).unwrap();

        assert_eq!(app.flat_items.len(), 2); // 1 Group + 1 Session
        assert!(matches!(app.flat_items[1], Selection::Session(_)));
    }
}
