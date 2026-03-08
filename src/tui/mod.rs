pub mod app;
pub mod event;
pub mod theme;
pub mod ui;
pub mod widgets;

use crate::core::Registry;
use crate::error::Result;
use crate::ops::Executor;
use app::App;
use crossterm::{
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{Terminal, backend::CrosstermBackend};
use std::io;
use std::sync::Arc;
use std::sync::mpsc;
use std::sync::mpsc::RecvTimeoutError;
use std::thread;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PreviewMode {
    Quick,
    Deep,
}

pub enum TuiEvent {
    Input(crossterm::event::KeyEvent),
    PartialScan(Vec<Arc<crate::core::Session>>),
    ScanComplete,
    PreviewLoaded {
        id: String,
        content: String,
        mode: PreviewMode,
    },
    SessionUpdated(Arc<crate::core::Session>),
}

fn has_deep_preview_marker(current_preview: Option<&str>) -> bool {
    current_preview.is_some_and(|p| p.contains("-- [ Deep preview ] --"))
}

fn should_apply_preview_update(
    selected_id: Option<&str>,
    event_id: &str,
    mode: PreviewMode,
    current_preview: Option<&str>,
) -> bool {
    if selected_id != Some(event_id) {
        return false;
    }
    if mode == PreviewMode::Quick && has_deep_preview_marker(current_preview) {
        return false;
    }
    true
}

pub fn run(registry: Registry, executor: Executor) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();

    // 1. Streaming background scan
    let tx_scan = tx.clone();
    let base_path = registry.scanner.base_path.clone();
    let cache_path = registry.cache_path.clone();

    thread::spawn(move || {
        let inner_registry = Registry::new(&base_path, &cache_path);
        if let Ok(all_dirs) = std::fs::read_dir(&base_path) {
            for entry in all_dirs.flatten() {
                let project_path = entry.path();
                if project_path.is_dir() {
                    let project_id = project_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or("unknown")
                        .to_string();

                    let project_root_file = project_path.join(".project_root");
                    let host_path = std::fs::read_to_string(project_root_file)
                        .ok()
                        .map(|s| std::path::PathBuf::from(s.trim()));

                    let chats_path = project_path.join("chats");
                    if chats_path.exists()
                        && chats_path.is_dir()
                        && let Ok(chats) = std::fs::read_dir(chats_path)
                    {
                        let mut batch = Vec::new();
                        for chat_entry in chats.flatten() {
                            let path = chat_entry.path();
                            if path.extension().is_some_and(|ext| ext == "json")
                                && let Ok(s) = crate::core::Session::from_path(
                                    &path,
                                    project_id.clone(),
                                    host_path.clone(),
                                )
                            {
                                let mut s_arc = Arc::new(s);
                                // Hit cache if possible
                                if let Some(entry) =
                                    inner_registry.cache.get(&s_arc.path, s_arc.updated_at)
                                    && let Some(s_mut) = Arc::get_mut(&mut s_arc)
                                {
                                    s_mut.health = entry.health;
                                    s_mut.name = entry.name;
                                    s_mut.validation_notes = entry.notes;
                                }
                                batch.push(s_arc);
                                if batch.len() >= 20 {
                                    let _ = tx_scan
                                        .send(TuiEvent::PartialScan(std::mem::take(&mut batch)));
                                }
                            }
                        }
                        if !batch.is_empty() {
                            let _ = tx_scan.send(TuiEvent::PartialScan(batch));
                        }
                    }
                }
            }
        }
        let _ = tx_scan.send(TuiEvent::ScanComplete);
    });

    // 2. CONSTANT WORKER: Dedicated Preview Thread
    let (tx_preview_worker, rx_preview_worker) = mpsc::channel::<(
        Arc<crate::core::Session>,
        PreviewMode,
        crate::config::PreviewConfig,
        mpsc::Sender<TuiEvent>,
    )>();
    thread::spawn(move || {
        while let Ok((session, mode, preview_cfg, tx_out)) = rx_preview_worker.recv() {
            let markdown = match mode {
                PreviewMode::Quick => {
                    crate::ops::export::session_to_markdown_limited(&session, 20, &preview_cfg)
                }
                PreviewMode::Deep => crate::ops::export::session_to_markdown_deep_limited(
                    &session,
                    200,
                    &preview_cfg,
                ),
            }
            .unwrap_or_else(|_| "Error loading preview".to_string());

            let _ = tx_out.send(TuiEvent::PreviewLoaded {
                id: session.id.clone(),
                content: markdown,
                mode,
            });
        }
    });

    // 3. VALIDATION WORKER: Performs deep_validate for sessions with Unknown health
    let (tx_validation, rx_validation) =
        mpsc::channel::<(Arc<crate::core::Session>, mpsc::Sender<TuiEvent>)>();
    thread::spawn(move || {
        while let Ok((session, tx_out)) = rx_validation.recv() {
            let mut s = (*session).clone();
            s.deep_validate();
            let _ = tx_out.send(TuiEvent::SessionUpdated(Arc::new(s)));
        }
    });

    // 4. Input polling thread
    let tx_input = tx.clone();
    thread::spawn(move || {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(20)).unwrap_or(false)
                && let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
            {
                let _ = tx_input.send(TuiEvent::Input(key));
            }
        }
    });

    let mut app = App::new(registry, executor);
    app.message = Some("Streaming sessions...".to_string());

    let mut last_input_time = std::time::Instant::now();
    let mut last_rebuild_time = std::time::Instant::now();
    let mut last_session_update_render = std::time::Instant::now();
    let mut preview_triggered = true;
    let mut should_render = true;
    let mut needs_rebuild = false;
    let mut has_pending_session_visual_update = false;

    loop {
        if should_render {
            terminal.draw(|f| ui::render(&mut app, f))?;
            should_render = false;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(TuiEvent::PartialScan(new_batch)) => {
                // Background validation for Unknown ones
                for s in &new_batch {
                    if s.health == crate::core::session::SessionHealth::Unknown {
                        let _ = tx_validation.send((s.clone(), tx.clone()));
                    }
                }
                app.add_sessions(new_batch, false)?; // DEFER SORT
                needs_rebuild = true;
            }
            Ok(TuiEvent::ScanComplete) => {
                app.message = None;
                app.rebuild_tree();
                needs_rebuild = false;
                should_render = true;
            }
            Ok(TuiEvent::SessionUpdated(updated_session)) => {
                if let Some(&idx) = app.registry.session_indices.get(&updated_session.id) {
                    app.registry.sessions[idx] = updated_session;
                    // Batch visual refresh to avoid redrawing on every single worker update.
                    has_pending_session_visual_update = true;
                }
            }
            Ok(TuiEvent::PreviewLoaded { id, content, mode }) => {
                if should_apply_preview_update(
                    app.last_selected_id.as_deref(),
                    &id,
                    mode,
                    app.current_preview.as_deref(),
                ) {
                    app.current_preview = Some(content);
                    should_render = true;
                }
            }
            Ok(TuiEvent::Input(key)) => {
                let old_id = app.last_selected_id.clone();
                event::handle_key_event(&mut app, key)?;
                should_render = true;

                if app.last_selected_id != old_id {
                    last_input_time = std::time::Instant::now();
                    preview_triggered = false;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }

        // Periodic tasks run off timeout-driven loop instead of queued Tick events.
        if needs_rebuild && last_rebuild_time.elapsed() > std::time::Duration::from_millis(250) {
            app.rebuild_tree();
            last_rebuild_time = std::time::Instant::now();
            needs_rebuild = false;
            should_render = true;
        }

        if has_pending_session_visual_update
            && last_session_update_render.elapsed() > std::time::Duration::from_millis(200)
        {
            app.items_cache = None;
            should_render = true;
            has_pending_session_visual_update = false;
            last_session_update_render = std::time::Instant::now();
        }

        if !preview_triggered
            && last_input_time.elapsed() > std::time::Duration::from_millis(150)
            && let Some(s) = app.get_selected_session()
        {
            let _ = tx_preview_worker.send((
                s,
                PreviewMode::Quick,
                app.executor.config.preview.clone(),
                tx.clone(),
            ));
            preview_triggered = true;
        }

        if app.force_deep_preview
            && let Some(s) = app.get_selected_session()
        {
            let _ = tx_preview_worker.send((
                s,
                PreviewMode::Deep,
                app.executor.config.preview.clone(),
                tx.clone(),
            ));
            app.force_deep_preview = false;
            preview_triggered = true;
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_should_apply_preview_update_for_matching_id() {
        let ok = should_apply_preview_update(
            Some("session-a"),
            "session-a",
            PreviewMode::Quick,
            Some("Loading preview..."),
        );
        assert!(ok);
    }

    #[test]
    fn test_should_reject_preview_update_for_non_selected_id() {
        let ok = should_apply_preview_update(
            Some("session-a"),
            "session-b",
            PreviewMode::Quick,
            Some("Loading preview..."),
        );
        assert!(!ok);
    }

    #[test]
    fn test_should_reject_quick_preview_when_deep_preview_active() {
        let ok = should_apply_preview_update(
            Some("session-a"),
            "session-a",
            PreviewMode::Quick,
            Some("-- [ Deep preview ] --\n\ncontent"),
        );
        assert!(!ok);
    }

    #[test]
    fn test_should_accept_deep_preview_when_deep_preview_active() {
        let ok = should_apply_preview_update(
            Some("session-a"),
            "session-a",
            PreviewMode::Deep,
            Some("-- [ Deep preview ] --\n\ncontent"),
        );
        assert!(ok);
    }
}
