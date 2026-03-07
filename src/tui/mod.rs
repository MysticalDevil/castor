pub mod app;
pub mod event;
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

pub enum TuiEvent {
    Input(crossterm::event::KeyEvent),
    ScanComplete(Registry),
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

    // Start background scanning thread
    let tx_scan = tx.clone();
    let mut scan_registry = Registry::new(&registry.scanner.base_path, &registry.cache_path);
    thread::spawn(move || {
        if scan_registry.reload().is_ok() {
            let _ = tx_scan.send(TuiEvent::ScanComplete(scan_registry));
        }
    });

    // Start input polling thread
    let tx_input = tx.clone();
    thread::spawn(move || {
        loop {
            if crossterm::event::poll(std::time::Duration::from_millis(100)).unwrap_or(false)
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

    loop {
        terminal.draw(|f| ui::render(&mut app, f))?;

        match rx.recv_timeout(std::time::Duration::from_millis(50)) {
            Ok(TuiEvent::ScanComplete(new_registry)) => {
                app.registry = new_registry;
                app.reload()?; // Recalculate tree
                app.message = None;
            }
            Ok(TuiEvent::Input(key)) => {
                event::handle_key_event(&mut app, key)?;
            }
            Ok(TuiEvent::Tick) => {}
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
