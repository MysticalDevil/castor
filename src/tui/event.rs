use crate::ops::Executor;
use crate::tui::app::{App, InputMode};
use crossterm::event::{self, Event, KeyCode};
use std::time::Duration;

pub fn handle_events(app: &mut App, executor: &Executor) -> crate::error::Result<()> {
    if event::poll(Duration::from_millis(100))? {
        if let Event::Key(key) = event::read()? {
            match app.input_mode {
                InputMode::Normal => match key.code {
                    KeyCode::Char('q') => app.should_quit = true,
                    KeyCode::Char('j') | KeyCode::Down => app.next(),
                    KeyCode::Char('k') | KeyCode::Up => app.previous(),
                    KeyCode::Char('r') => app.reload()?,
                    KeyCode::Char('d') => {
                        if app.selected_session().is_some() {
                            app.input_mode = InputMode::ConfirmDelete;
                        }
                    }
                    _ => {}
                },
                InputMode::ConfirmDelete => match key.code {
                    KeyCode::Char('y') => {
                        if let Some(session) = app.selected_session().cloned() {
                            executor.delete_soft(&session, false)?;
                            app.message = Some(format!("Deleted session {}", session.id));
                            app.reload()?;
                        }
                        app.input_mode = InputMode::Normal;
                    }
                    KeyCode::Char('n') | KeyCode::Esc => {
                        app.input_mode = InputMode::Normal;
                    }
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
