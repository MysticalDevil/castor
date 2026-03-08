use crate::core::session::SessionHealth;
use crate::tui::app::{App, GroupingMode, InputMode, Selection};
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
            Constraint::Length(3), // Keys bar with borders
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

fn render_tree(app: &mut App, frame: &mut Frame, area: Rect) {
    let icons = Icons::get(app.executor.config.icon_set);
    let items: Vec<ListItem> = app
        .flat_items
        .iter()
        .map(|sel| match sel {
            Selection::Group(id) => {
                let prefix = match app.grouping_mode {
                    GroupingMode::Host => format!("{} ", icons.folder),
                    GroupingMode::Month => "🗓 ".to_string(),
                };
                ListItem::new(format!("{}{}", prefix, id)).style(
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                )
            }
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
            }
        })
        .collect();

    let title = match app.grouping_mode {
        GroupingMode::Host => " Projects / Sessions ",
        GroupingMode::Month => " Months / Sessions ",
    };

    let list = List::new(items)
        .block(Block::default().title(title).borders(Borders::ALL))
        .highlight_style(
            Style::default()
                .bg(Color::DarkGray)
                .fg(Color::Yellow)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("> ");

    // Stateful rendering for scrolling
    frame.render_stateful_widget(list, area, &mut app.list_state);
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

        // 2. Conversation Preview Panel (Using Cache)
        let preview_content = app
            .current_preview
            .as_deref()
            .unwrap_or("No content or error reading session.");

        // Use tui-markdown for rich rendering
        let mut text = tui_markdown::from_str(preview_content);

        // Post-process to colorize USER and GEMINI headers
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
            .wrap(Wrap { trim: false });
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
    let mut spans = vec![Span::styled(
        "[KEYS] ",
        Style::default()
            .fg(Color::Magenta)
            .add_modifier(Modifier::BOLD),
    )];

    match app.input_mode {
        InputMode::Normal => {
            spans.extend(vec![
                Span::styled("q ", Style::default().fg(Color::Cyan)),
                Span::styled("quit ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("j/k ", Style::default().fg(Color::Cyan)),
                Span::styled("navigate ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("g ", Style::default().fg(Color::Cyan)),
                Span::styled("group ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("d ", Style::default().fg(Color::Cyan)),
                Span::styled("delete ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("r ", Style::default().fg(Color::Cyan)),
                Span::styled("reload ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("enter ", Style::default().fg(Color::Cyan)),
                Span::styled("select ", Style::default().fg(Color::DarkGray)),
            ]);
        }
        InputMode::ConfirmDelete => {
            spans.extend(vec![
                Span::styled(
                    "CONFIRM DELETE? ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("y ", Style::default().fg(Color::Green)),
                Span::styled("yes ", Style::default().fg(Color::DarkGray)),
                Span::raw("| "),
                Span::styled("n ", Style::default().fg(Color::Yellow)),
                Span::styled("no ", Style::default().fg(Color::DarkGray)),
            ]);
        }
    }

    let bar = Paragraph::new(Line::from(spans)).block(Block::default().borders(Borders::ALL));
    frame.render_widget(bar, area);
}
