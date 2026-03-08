use crate::core::session::SessionHealth;
use crate::ops::export;
use crate::tui::app::{App, InputMode, Selection};
use crate::utils::icons::Icons;
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
        .split(frame.area());

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
    let icons = Icons::get(app.executor.config.icon_set);
    let items: Vec<ListItem> =
        app.flat_items
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
                    Selection::Project(id) => ListItem::new(format!("{} {}", icons.folder, id))
                        .style(style.fg(Color::Cyan)),
                    Selection::Session(id) => {
                        let session = app.registry.find_by_id(id).unwrap();
                        let health_symbol = match session.health {
                            SessionHealth::Unknown => Span::raw(icons.unknown).fg(Color::DarkGray),
                            SessionHealth::Ok => Span::raw(icons.ok).green(),
                            SessionHealth::Warn => Span::raw(icons.warn).yellow(),
                            SessionHealth::Error => Span::raw(icons.error).red(),
                            SessionHealth::Risk => Span::raw(icons.risk).magenta(),
                        };
                        let display_id = id
                            .strip_suffix(".json")
                            .unwrap_or(id)
                            .split('-')
                            .next_back()
                            .unwrap_or(id);
                        ListItem::new(Line::from(vec![
                            Span::raw(format!("  {} ", icons.chat)),
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

        let health = &session.health;
        let health_color = match health {
            SessionHealth::Unknown => Color::DarkGray,
            SessionHealth::Ok => Color::Green,
            SessionHealth::Warn => Color::Yellow,
            SessionHealth::Error => Color::Red,
            SessionHealth::Risk => Color::Magenta,
        };

        let status_text = vec![
            Line::from(vec![
                Span::styled(
                    "ID:       ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&session.id),
            ]),
            Line::from(vec![
                Span::styled(
                    "Project:  ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&session.project_id),
            ]),
            Line::from(vec![
                Span::styled(
                    "Host:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(host_display),
            ]),
            Line::from(vec![
                Span::styled(
                    "Updated:  ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(session.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Line::from(vec![
                Span::styled(
                    "Size:     ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.2} KB", session.size as f64 / 1024.0)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Health:   ",
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    format!("{}", health),
                    Style::default()
                        .fg(health_color)
                        .add_modifier(Modifier::BOLD),
                ),
            ]),
        ];

        let status_block = Paragraph::new(status_text).block(
            Block::default()
                .title(" File Status ")
                .borders(Borders::ALL),
        );
        frame.render_widget(status_block, details_layout[0]);

        // 2. Conversation Preview Panel (Using tui-markdown)
        let markdown_content = match export::session_to_markdown(session) {
            Ok(md) => md,
            Err(_) => "Error reading session content.".to_string(),
        };

        // Use tui-markdown for rich rendering
        let mut text = tui_markdown::from_str(&markdown_content);

        // Post-process to colorize USER and GEMINI headers specifically
        for line in &mut text.lines {
            let is_user = line.spans.iter().any(|s| s.content.contains("USER"));
            let is_gemini = line.spans.iter().any(|s| s.content.contains("GEMINI"));

            if is_user {
                for span in &mut line.spans {
                    span.style = span.style.fg(Color::Blue).add_modifier(Modifier::BOLD);
                }
            } else if is_gemini {
                for span in &mut line.spans {
                    span.style = span.style.fg(Color::Green).add_modifier(Modifier::BOLD);
                }
            }
        }

        let preview_block = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Conversation Preview ")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false }); // keep markdown spacing
        frame.render_widget(preview_block, details_layout[1]);
    } else {
        let msg = app
            .message
            .as_deref()
            .unwrap_or("Select a session to view details");
        let placeholder = Paragraph::new(msg)
            .block(Block::default().borders(Borders::ALL))
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(placeholder, area);
    }
}

fn render_keys_bar(app: &App, frame: &mut Frame, area: Rect) {
    let keys = match app.input_mode {
        InputMode::Normal => vec![
            " Q ".black().on_cyan(),
            " Quit ".white().on_dark_gray(),
            " J/K ".black().on_cyan(),
            " Navigate ".white().on_dark_gray(),
            " D ".black().on_cyan(),
            " Delete ".white().on_dark_gray(),
            " R ".black().on_cyan(),
            " Reload ".white().on_dark_gray(),
            " Enter ".black().on_cyan(),
            " Select ".white().on_dark_gray(),
        ],
        InputMode::ConfirmDelete => vec![
            " CONFIRM DELETE? ".black().on_red().bold(),
            " Y ".black().on_green(),
            " Yes ".white().on_dark_gray(),
            " N ".black().on_yellow(),
            " No ".white().on_dark_gray(),
        ],
    };

    let bar = Paragraph::new(Line::from(keys));
    frame.render_widget(bar, area);
}
