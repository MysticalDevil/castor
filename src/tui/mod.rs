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
use std::thread;

pub enum TuiEvent {
    Input(crossterm::event::KeyEvent),
    PartialScan(Vec<Arc<crate::core::Session>>),
    ScanComplete,
    PreviewLoaded { id: String, content: String },
    Tick,
}

pub fn run(registry: Registry, executor: Executor) -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let (tx, rx) = mpsc::channel();

    // 1. IMPROVED: Streaming background scan
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
                                && let Ok(mut s) = crate::core::Session::from_path(
                                    &path,
                                    project_id.clone(),
                                    host_path.clone(),
                                )
                            {
                                if let Some(entry) = inner_registry.cache.get(&s.path, s.updated_at)
                                {
                                    s.health = entry.health;
                                    s.name = entry.name;
                                    s.validation_notes = entry.notes;
                                }
                                batch.push(Arc::new(s));
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

    // 2. Input polling thread
    let tx_input = tx.clone();
    thread::spawn(move || {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(50)).unwrap_or(false)
                && let Ok(crossterm::event::Event::Key(key)) = crossterm::event::read()
            {
                let _ = tx_input.send(TuiEvent::Input(key));
            }
            let _ = tx_input.send(TuiEvent::Tick);
        }
    });

    // create app
    let mut app = App::new(registry, executor);
    app.message = Some("Streaming sessions...".to_string());

    let mut last_input_time = std::time::Instant::now();
    let mut preview_triggered = true;
    let mut should_render = true;

    loop {
        if should_render {
            terminal.draw(|f| ui::render(&mut app, f))?;
            should_render = false;
        }

        match rx.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(TuiEvent::PartialScan(new_batch)) => {
                app.add_sessions(new_batch)?;
                should_render = true;
            }
            Ok(TuiEvent::ScanComplete) => {
                app.message = None;
                should_render = true;
            }
            Ok(TuiEvent::PreviewLoaded { id, content }) => {
                if app.last_selected_id.as_ref() == Some(&id) {
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
            Ok(TuiEvent::Tick) => {
                if !preview_triggered
                    && last_input_time.elapsed() > std::time::Duration::from_millis(100)
                    && let Some(s) = app.get_selected_session()
                {
                    trigger_async_preview(s, tx.clone());
                    preview_triggered = true;
                }
            }
            Err(_) => {}
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

fn trigger_async_preview(session: Arc<crate::core::Session>, tx: mpsc::Sender<TuiEvent>) {
    thread::spawn(move || {
        let markdown = crate::ops::export::session_to_markdown_limited(&session, 20)
            .unwrap_or_else(|_| "Error loading preview".to_string());

        let _ = tx.send(TuiEvent::PreviewLoaded {
            id: session.id.clone(),
            content: markdown,
        });
    });
}
