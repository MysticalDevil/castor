use crate::tui::app::{App, InputMode};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

pub fn render(app: &mut App, frame: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(frame.size());

    // Title
    let title = Paragraph::new("Castor: Gemini Session Manager").block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::Cyan)),
    );
    frame.render_widget(title, chunks[0]);

    // Main area (Sessions List)
    let sessions: Vec<ListItem> = app
        .registry
        .list()
        .iter()
        .enumerate()
        .map(|(i, s)| {
            let style = if i == app.selected_index {
                Style::default()
                    .bg(Color::DarkGray)
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            let display_id = s.id.strip_suffix(".json")
                .unwrap_or(&s.id)
                .split('-')
                .last()
                .unwrap_or(&s.id);
            
            let host = if let Some(path) = &s.host_path {
                let home = std::env::var("HOME").ok();
                crate::utils::fs::format_host(path, home.as_deref())
            } else {
                s.project_id.clone()
            };
            
            let head = s.name.as_deref().unwrap_or("---");
            let head_truncated = if head.chars().count() > 15 {
                format!("{}...", head.chars().take(12).collect::<String>())
            } else {
                head.to_string()
            };

            ListItem::new(format!("{:<12} | {:<20} | {:<20}", display_id, host, head_truncated)).style(style)
        })
        .collect();

    let list = List::new(sessions)
        .block(Block::default().title("Sessions").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol(">> ");

    frame.render_widget(list, chunks[1]);

    // Footer / Help
    let help_text = match app.input_mode {
        InputMode::Normal => " [q] Quit | [j/k] Up/Down | [d] Delete | [r] Reload",
        InputMode::ConfirmDelete => " Confirm Delete? [y] Yes | [n] No",
    };

    let footer = Paragraph::new(app.message.as_deref().unwrap_or(help_text))
        .block(Block::default().borders(Borders::ALL));
    frame.render_widget(footer, chunks[2]);
}
