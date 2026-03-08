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
use std::sync::mpsc;
use std::thread;
pub use theme::{Theme, ThemeConfig};

pub enum TuiEvent {
    Input(crossterm::event::KeyEvent),
    ScanComplete(Registry),
    PreviewLoaded { id: String, content: String },
    Tick,
}

pub fn run(registry: Registry, executor: Executor) -> Result<()> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup communication channels
    let (tx, rx) = mpsc::channel();

    // 1. Initial background scan
    let tx_scan = tx.clone();
    let mut scan_registry = Registry::new(&registry.scanner.base_path, &registry.cache_path);
    thread::spawn(move || {
        if scan_registry.reload().is_ok() {
            let _ = tx_scan.send(TuiEvent::ScanComplete(scan_registry));
        }
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
    app.message = Some("Scanning sessions...".to_string());

    let mut last_input_time = std::time::Instant::now();
    let mut preview_triggered = true;

    loop {
        terminal.draw(|f| ui::render(&mut app, f))?;

        match rx.recv_timeout(std::time::Duration::from_millis(20)) {
            Ok(TuiEvent::ScanComplete(new_registry)) => {
                app.registry = new_registry;
                app.reload()?;
                app.message = None;
                preview_triggered = false;
            }
            Ok(TuiEvent::PreviewLoaded { id, content }) => {
                if app.last_selected_id.as_ref() == Some(&id) {
                    app.current_preview = Some(content);
                }
            }
            Ok(TuiEvent::Input(key)) => {
                let old_id = app.last_selected_id.clone();
                event::handle_key_event(&mut app, key)?;

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
                    trigger_async_preview(s.clone(), tx.clone());
                    preview_triggered = true;
                }
            }
            Err(_) => {}
        }

        if app.should_quit {
            break;
        }
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen,)?;
    terminal.show_cursor()?;

    Ok(())
}

fn trigger_async_preview(session: crate::core::Session, tx: mpsc::Sender<TuiEvent>) {
    thread::spawn(move || {
        let s_clone = session.clone();
        // Use a limited markdown conversion for performance
        let content = crate::ops::export::session_to_markdown_limited(&s_clone, 20)
            .unwrap_or_else(|_| "Error loading preview".to_string());

        let _ = tx.send(TuiEvent::PreviewLoaded {
            id: session.id,
            content,
        });
    });
}
