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
            KeyCode::Char('h') | KeyCode::Left => app.collapse_selected_group(),
            KeyCode::Char('l') | KeyCode::Right => app.expand_selected_group(),
            KeyCode::Char(' ') => app.toggle_selected_group(),
            KeyCode::Char('r') => {
                // Background reload logic would go here or just triggered via msg
                app.message = Some("Reloading...".to_string());
                app.reload()?;
            }
            KeyCode::Char('p') => {
                app.request_deep_preview();
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, PreviewConfig};
    use crate::core::Registry;
    use crate::ops::Executor;
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use std::fs;
    use tempfile::tempdir;

    fn key(c: KeyCode) -> KeyEvent {
        KeyEvent::new(c, KeyModifiers::NONE)
    }

    fn make_app() -> App {
        let tmp = tempdir().expect("create tempdir");
        let project_path = tmp.path().join("proj1/chats");
        fs::create_dir_all(&project_path).expect("create project chats dir");
        let s_path = project_path.join("session-2026-03-08T12-00-aaaa1111.json");
        fs::write(
            &s_path,
            r#"{"messages":[{"type":"user","content":"hello"},{"type":"assistant","content":"world"}]}"#,
        )
        .expect("write session fixture");

        let mut registry = Registry::new(tmp.path(), &tmp.path().join("cache.json"));
        registry.reload().expect("reload registry");
        let sessions = registry.sessions.clone();
        registry.sessions.clear();
        registry.session_indices.clear();

        let executor = Executor::new(Config {
            gemini_sessions_path: tmp.path().to_path_buf(),
            trash_path: tmp.path().join("trash"),
            audit_path: tmp.path().join("audit"),
            cache_path: tmp.path().join("cache").join("metadata.json"),
            dry_run_by_default: true,
            icon_set: crate::utils::icons::IconSet::Ascii,
            theme: crate::tui::theme::ThemeConfig::default(),
            preview: PreviewConfig::default(),
        });
        let mut app = App::new(registry, executor);
        app.add_sessions(sessions, true).expect("add sessions");
        app
    }

    #[test]
    fn test_handle_key_navigation_and_quit() {
        let mut app = make_app();
        app.list_state.select(Some(0));
        handle_key_event(&mut app, key(KeyCode::Char('j'))).expect("handle j");
        handle_key_event(&mut app, key(KeyCode::Char('k'))).expect("handle k");
        handle_key_event(&mut app, key(KeyCode::Char('h'))).expect("handle h");
        handle_key_event(&mut app, key(KeyCode::Char('l'))).expect("handle l");
        handle_key_event(&mut app, key(KeyCode::Char(' '))).expect("handle space");
        handle_key_event(&mut app, key(KeyCode::Char('p'))).expect("handle p");
        handle_key_event(&mut app, key(KeyCode::Char('q'))).expect("handle q");
        assert!(app.should_quit);
    }

    #[test]
    fn test_handle_key_delete_mode_transitions() {
        let mut app = make_app();
        handle_key_event(&mut app, key(KeyCode::Char('d'))).expect("enter delete mode");
        assert!(matches!(app.input_mode, InputMode::ConfirmDelete));
        handle_key_event(&mut app, key(KeyCode::Char('n'))).expect("cancel delete mode");
        assert!(matches!(app.input_mode, InputMode::Normal));
    }
}
