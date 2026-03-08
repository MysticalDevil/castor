use crate::core::session::SessionHealth;
use crate::tui::app::{App, GroupingMode, InputMode, Selection};
use crate::utils::icons::Icons;
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span, Text},
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
    let theme = app.executor.config.theme.get_theme();
    let icons = Icons::get(app.executor.config.icon_set);
    let line_budget = area.width.saturating_sub(6) as usize;

    let items = if let Some(cached) = &app.items_cache {
        cached.clone()
    } else {
        let new_items: Vec<ListItem> = app
            .flat_items
            .iter()
            .map(|sel| match sel {
                Selection::Group(id) => {
                    let session_count = app
                        .sessions_by_group
                        .get(id)
                        .map(|v| v.len())
                        .unwrap_or(0usize);
                    let marker = if app.is_group_collapsed(id) {
                        "▸"
                    } else {
                        "▾"
                    };
                    let group_label = format!("{} {} ({})", marker, id, session_count);
                    let group_label = truncate_with_ellipsis(&group_label, line_budget);
                    ListItem::new(group_label).style(
                        Style::default()
                            .fg(theme.folder)
                            .add_modifier(Modifier::BOLD),
                    )
                }
                Selection::SessionIndex(idx) => {
                    let session = &app.registry.sessions[*idx];
                    let health_symbol = match session.health {
                        SessionHealth::Unknown => Span::raw("?").fg(theme.key_desc),
                        SessionHealth::Ok => Span::raw("•").green(),
                        SessionHealth::Warn => Span::raw("!").yellow(),
                        SessionHealth::Error => Span::raw("×").red(),
                        SessionHealth::Risk => Span::raw("▲").magenta(),
                    };

                    let id_text =
                        truncate_with_ellipsis(&session.display_id, line_budget.saturating_sub(6));
                    ListItem::new(Line::from(vec![
                        Span::raw("  "),
                        Span::raw(format!("{} ", icons.chat)).fg(theme.key_desc),
                        health_symbol,
                        Span::raw(format!(" {}", id_text)),
                    ]))
                }
            })
            .collect();

        app.items_cache = Some(new_items.clone());
        new_items
    };

    let title = match app.grouping_mode {
        GroupingMode::Host => " Projects / Sessions ",
        GroupingMode::Month => " Months / Sessions ",
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title_style(Style::default().fg(theme.title)),
        )
        .highlight_style(
            Style::default()
                .bg(theme.selection_bg)
                .fg(theme.selection_fg)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol("› ");

    frame.render_stateful_widget(list, area, &mut app.list_state);
}

fn truncate_with_ellipsis(input: &str, max_chars: usize) -> String {
    if max_chars == 0 {
        return String::new();
    }
    let count = input.chars().count();
    if count <= max_chars {
        return input.to_string();
    }
    if max_chars <= 1 {
        return "…".to_string();
    }
    let kept = max_chars - 1;
    let mut out = input.chars().take(kept).collect::<String>();
    out.push('…');
    out
}

fn render_details(app: &mut App, frame: &mut Frame, area: Rect) {
    let theme = app.executor.config.theme.get_theme();
    let details_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // File Status
            Constraint::Min(0),    // Conversation Preview
        ])
        .split(area);

    if let Some(session) = app.get_selected_session() {
        let home = std::env::var("HOME").ok();
        let host_display = session
            .host_path
            .as_ref()
            .map(|p| crate::utils::fs::format_host(p, home.as_deref()))
            .unwrap_or_else(|| "Unknown".to_string());

        let health = &session.health;
        let health_color = match health {
            SessionHealth::Unknown => theme.key_desc,
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
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&session.id),
            ]),
            Line::from(vec![
                Span::styled(
                    "Project:  ",
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(&session.project_id),
            ]),
            Line::from(vec![
                Span::styled(
                    "Host:     ",
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(host_display),
            ]),
            Line::from(vec![
                Span::styled(
                    "Updated:  ",
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(session.updated_at.format("%Y-%m-%d %H:%M:%S").to_string()),
            ]),
            Line::from(vec![
                Span::styled(
                    "Size:     ",
                    Style::default()
                        .fg(theme.title)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::raw(format!("{:.2} KB", session.size as f64 / 1024.0)),
            ]),
            Line::from(vec![
                Span::styled(
                    "Health:   ",
                    Style::default()
                        .fg(theme.title)
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
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.border))
                .title_style(Style::default().fg(theme.title)),
        );
        frame.render_widget(status_block, details_layout[0]);

        let preview_content = app
            .current_preview
            .as_deref()
            .unwrap_or("Loading preview...");

        // CACHE HIT: Only parse if not in cache or if it's the "Loading preview..." message
        let mut text: Text = if let Some(session_id) = &app.last_selected_id
            && preview_content != "Loading preview..."
        {
            if let Some(cached) = app.markdown_cache.get(session_id) {
                cached.clone()
            } else {
                let parsed = tui_markdown::from_str(preview_content);
                let owned = crate::tui::app::to_owned_text(parsed);
                app.markdown_cache.insert(session_id.clone(), owned.clone());
                owned
            }
        } else {
            tui_markdown::from_str(preview_content)
        };

        for line in &mut text.lines {
            let is_user = line.spans.iter().any(|s| s.content.contains("USER"));
            let is_gemini = line.spans.iter().any(|s| s.content.contains("GEMINI"));

            if is_user {
                for span in &mut line.spans {
                    span.style = span.style.fg(theme.user_msg).add_modifier(Modifier::BOLD);
                }
            } else if is_gemini {
                for span in &mut line.spans {
                    span.style = span.style.fg(theme.gemini_msg).add_modifier(Modifier::BOLD);
                }
            }
        }

        let preview_block = Paragraph::new(text)
            .block(
                Block::default()
                    .title(" Conversation Preview ")
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border))
                    .title_style(Style::default().fg(theme.title)),
            )
            .wrap(Wrap { trim: false });
        frame.render_widget(preview_block, details_layout[1]);
    } else {
        let msg = app
            .message
            .as_deref()
            .unwrap_or("Select a session to view details");
        let placeholder = Paragraph::new(msg)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(Style::default().fg(theme.border)),
            )
            .alignment(ratatui::layout::Alignment::Center);
        frame.render_widget(placeholder, area);
    }
}

fn render_keys_bar(app: &App, frame: &mut Frame, area: Rect) {
    let theme = app.executor.config.theme.get_theme();
    let mut spans = vec![Span::styled(
        "[KEYS] ",
        Style::default()
            .fg(theme.key_hint)
            .add_modifier(Modifier::BOLD),
    )];

    match app.input_mode {
        InputMode::Normal => {
            spans.extend(vec![
                Span::styled("q ", Style::default().fg(theme.title)),
                Span::styled("quit ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("j/k ", Style::default().fg(theme.title)),
                Span::styled("navigate ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("h/l ", Style::default().fg(theme.title)),
                Span::styled("fold/unfold ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("space ", Style::default().fg(theme.title)),
                Span::styled("toggle group ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("g ", Style::default().fg(theme.title)),
                Span::styled("group ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("d ", Style::default().fg(theme.title)),
                Span::styled("delete ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("r ", Style::default().fg(theme.title)),
                Span::styled("reload ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("p ", Style::default().fg(theme.title)),
                Span::styled("deep preview ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("enter ", Style::default().fg(theme.title)),
                Span::styled("select ", Style::default().fg(theme.key_desc)),
            ]);
        }
        InputMode::ConfirmDelete => {
            spans.extend(vec![
                Span::styled(
                    "CONFIRM DELETE? ",
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
                Span::styled("y ", Style::default().fg(Color::Green)),
                Span::styled("yes ", Style::default().fg(theme.key_desc)),
                Span::raw("| "),
                Span::styled("n ", Style::default().fg(Color::Yellow)),
                Span::styled("no ", Style::default().fg(theme.key_desc)),
            ]);
        }
    }

    let bar = Paragraph::new(Line::from(spans)).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border)),
    );
    frame.render_widget(bar, area);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, PreviewConfig};
    use crate::core::Registry;
    use crate::ops::Executor;
    use ratatui::{Terminal, backend::TestBackend};
    use std::fs;
    use tempfile::tempdir;

    fn make_app_with_one_session() -> App {
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
    fn test_render_with_placeholder() {
        let mut app = make_app_with_one_session();
        app.list_state.select(None);
        app.message = Some("hello".to_string());

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        terminal
            .draw(|f| render(&mut app, f))
            .expect("render placeholder");
    }

    #[test]
    fn test_render_with_selected_session_and_preview() {
        let mut app = make_app_with_one_session();
        app.list_state.select(Some(1));
        app.current_preview = Some("## USER\nhi\n\n## GEMINI\nhello".to_string());
        app.last_selected_id = app
            .get_selected_session()
            .map(|s| s.id.clone())
            .or_else(|| Some("fallback".to_string()));

        let backend = TestBackend::new(120, 40);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        terminal
            .draw(|f| render(&mut app, f))
            .expect("render with selected session");
    }

    #[test]
    fn test_render_tree_with_collapsed_group() {
        let mut app = make_app_with_one_session();
        app.list_state.select(Some(0));
        app.toggle_selected_group();

        let backend = TestBackend::new(80, 24);
        let mut terminal = Terminal::new(backend).expect("create terminal");
        terminal
            .draw(|f| render(&mut app, f))
            .expect("render collapsed group");
    }

    #[test]
    fn test_truncate_with_ellipsis() {
        assert_eq!(truncate_with_ellipsis("abc", 5), "abc");
        assert_eq!(truncate_with_ellipsis("abcdef", 4), "abc…");
        assert_eq!(truncate_with_ellipsis("abcdef", 1), "…");
        assert_eq!(truncate_with_ellipsis("abcdef", 0), "");
    }
}
