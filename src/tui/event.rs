use crate::tui::app::{App, InputMode};
use crossterm::event::KeyCode;

pub fn handle_key_event(
    app: &mut App,
    key: crossterm::event::KeyEvent,
) -> crate::error::Result<()> {
    match app.input_mode {
        InputMode::Normal => match key.code {
            KeyCode::Char('q') => app.should_quit = true,
            KeyCode::Char('j') | KeyCode::Down => app.next(),
            KeyCode::Char('k') | KeyCode::Up => app.previous(),
            KeyCode::Char('g') => app.toggle_grouping()?,
            KeyCode::Char('r') => {
                // Background reload logic would go here or just triggered via msg
                app.message = Some("Reloading...".to_string());
                app.reload()?;
            }
            KeyCode::Char('d') => {
                if app.get_selected_session().is_some() {
                    app.input_mode = InputMode::ConfirmDelete;
                }
            }
            _ => {}
        },
        InputMode::ConfirmDelete => match key.code {
            KeyCode::Char('y') => {
                if let Some(session) = app.get_selected_session() {
                    app.executor.delete_soft(&session, false)?;
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
    Ok(())
}
