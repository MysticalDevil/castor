use crate::core::session::SessionHealth;
use crate::ops::export;
use crate::tui::app::{App, InputMode, Selection};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let root_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(1), // Keys bar
        ])
        .split(frame.size());

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(30), // Left tree
            Constraint::Percentage(70), // Right panels
        ])
        .split(root_layout[0]);

    render_tree(app, frame, main_layout[0]);
    render_details(app, frame, main_layout[1]);
    render_keys_bar(app, frame, root_layout[1]);
}

fn render_tree(app: &App, frame: &mut Frame, area: Rect) {
    let items: Vec<ListItem> = app
        .flat_items
        .iter()
        .enumerate()
        .map(|(i, sel)| {
            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };

            match sel {
                Selection::Project(id) => {
                    ListItem::new(format!("📁 {}", id)).style(style.fg(Color::Cyan))
                }
                Selection::Session(id) => {
                    let session = app.registry.find_by_id(id).unwrap();
                    let health_symbol = match session.check_health() {
                        SessionHealth::Ok => "●".green(),
                        SessionHealth::Warn => "▲".yellow(),
                        SessionHealth::Error => "✖".red(),
                        SessionHealth::Risk => "⚠".magenta(),
                    };
                    let display_id = id
                        .strip_suffix(".json")
                        .unwrap_or(id)
                        .split('-')
                        .next_back()
                        .unwrap_or(id);
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        health_symbol,
                        Span::raw(format!(" {}", display_id)),
                    ]))
                    .style(style)
                }
            }
        })
        .collect();

    let list = List::new(items)
        .block(
            Block::default()
                .title(" Projects / Sessions ")
                .borders(Borders::ALL),
        )
        .highlight_symbol("> ");
    frame.render_widget(list, area);
}

fn render_details(app: &App, frame: &mut Frame, area: Rect) {
    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // File Status
            Constraint::Min(0),    // Conversation Preview
        ])
        .split(area);

    if let Some(session) = app.get_selected_session() {
        // 1. File Status Panel
        let home = std::env::var("HOME").ok();
        let host_display = session
            .host_path
            .as_ref()
            .map(|p| crate::utils::fs::format_host(p, home.as_deref()))
            .unwrap_or_else(|| "Unknown".to_string());

        let status_text = vec![
            Line::from(vec![Span::raw("ID:       ").bold(), Span::raw(&session.id)]),
            Line::from(vec![
                Span::raw("Project:  ").bold(),
                Span::raw(&session.project_id),
            ]),
            Line::from(vec![
                Span::raw("Host:     ").bold(),
                Span::raw(host_display),
            ]),
            Line::from(vec![
                Span::raw("Updated:  ").bold(),
                Span::raw(session.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Line::from(vec![
                Span::raw("Size:     ").bold(),
                Span::raw(format!("{:.2} KB", session.size as f64 / 1024.0)),
            ]),
            Line::from(vec![
                Span::raw("Health:   ").bold(),
                Span::raw(format!("{}", session.check_health())),
            ]),
        ];

        let status_block = Paragraph::new(status_text).block(
            Block::default()
                .title(" File Status ")
                .borders(Borders::ALL),
        );
        frame.render_widget(status_block, details_layout[0]);

        // 2. Conversation Preview Panel
        let preview_content = match export::session_to_markdown(session) {
            Ok(md) => md,
            Err(_) => "Error reading session content.".to_string(),
        };

        let preview_block = Paragraph::new(preview_content)
            .block(
                Block::default()
                    .title(" Conversation Preview ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: true });
        frame.render_widget(preview_block, details_layout[1]);
    } else {
        let placeholder = Paragraph::new("Select a session to view details")
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(placeholder, area);
    }
}

fn render_keys_bar(app: &App, frame: &mut Frame, area: Rect) {
    let keys = match app.input_mode {
        InputMode::Normal => {
            " [q] Quit | [j/k] Navigate | [d] Delete | [r] Reload | [Enter] Select "
        }
        InputMode::ConfirmDelete => " Confirm Delete? [y] Yes | [n] No ",
    };

    let style = Style::default().bg(Color::Cyan).fg(Color::Black);
    let bar = Paragraph::new(keys).style(style);
    frame.render_widget(bar, area);
}
