use crate::core::{Registry, Session};
use crate::error::Result;

pub enum InputMode {
    Normal,
    ConfirmDelete,
}

pub struct App {
    pub registry: Registry,
    pub selected_index: usize,
    pub input_mode: InputMode,
    pub should_quit: bool,
    pub message: Option<String>,
}

impl App {
    pub fn new(registry: Registry) -> Self {
        Self {
            registry,
            selected_index: 0,
            input_mode: InputMode::Normal,
            should_quit: false,
            message: None,
        }
    }

    pub fn next(&mut self) {
        let count = self.registry.list().len();
        if count > 0 {
            self.selected_index = (self.selected_index + 1) % count;
        }
    }

    pub fn previous(&mut self) {
        let count = self.registry.list().len();
        if count > 0 {
            if self.selected_index == 0 {
                self.selected_index = count - 1;
            } else {
                self.selected_index -= 1;
            }
        }
    }

    pub fn selected_session(&self) -> Option<&Session> {
        self.registry.list().get(self.selected_index)
    }

    pub fn reload(&mut self) -> Result<()> {
        self.registry.reload()?;
        if self.selected_index >= self.registry.list().len() && !self.registry.list().is_empty() {
            self.selected_index = self.registry.list().len() - 1;
        }
        Ok(())
    }
}
