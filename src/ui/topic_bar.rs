use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let bg_style = Style::default().bg(Theme::BG_ELEVATED);

    let line = match &state.active_buffer {
        Some(BufferKey::Channel(server_id, channel)) => {
            let topic = state
                .get_server(*server_id)
                .and_then(|srv| srv.topics.get(channel))
                .map(|t| t.as_str())
                .unwrap_or("");

            if topic.is_empty() {
                Line::from(vec![
                    Span::styled(
                        " # ",
                        Style::default()
                            .fg(Theme::ACCENT_TEAL)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        channel,
                        Style::default()
                            .fg(Theme::ACCENT_TEAL)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " │ ",
                        Style::default()
                            .fg(Theme::BORDER_DIM)
                            .bg(Theme::BG_ELEVATED),
                    ),
                    Span::styled(
                        "No topic set",
                        Style::default()
                            .fg(Theme::TEXT_MUTED)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ])
            } else {
                Line::from(vec![
                    Span::styled(
                        " # ",
                        Style::default()
                            .fg(Theme::ACCENT_TEAL)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        channel,
                        Style::default()
                            .fg(Theme::ACCENT_TEAL)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        " │ ",
                        Style::default()
                            .fg(Theme::BORDER_DIM)
                            .bg(Theme::BG_ELEVATED),
                    ),
                    Span::styled(
                        topic,
                        Style::default()
                            .fg(Theme::TEXT_PRIMARY)
                            .bg(Theme::BG_ELEVATED)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ])
            }
        }
        Some(BufferKey::Query(_, target)) => Line::from(vec![
            Span::styled(
                " → ",
                Style::default()
                    .fg(Theme::ACCENT_LAVENDER)
                    .bg(Theme::BG_ELEVATED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                target,
                Style::default()
                    .fg(Theme::ACCENT_LAVENDER)
                    .bg(Theme::BG_ELEVATED)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Some(BufferKey::ServerStatus(server_id)) => {
            let name = state
                .get_server(*server_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            Line::from(vec![
                Span::styled(
                    " ◆ ",
                    Style::default()
                        .fg(Theme::ACCENT_GREEN)
                        .bg(Theme::BG_ELEVATED)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    name,
                    Style::default()
                        .fg(Theme::TEXT_PRIMARY)
                        .bg(Theme::BG_ELEVATED)
                        .add_modifier(Modifier::BOLD),
                ),
            ])
        }
        None => Line::from(vec![
            Span::styled(
                " 🦀 ",
                Style::default()
                    .fg(Theme::ACCENT_TEAL)
                    .bg(Theme::BG_ELEVATED),
            ),
            Span::styled(
                "CrabChat",
                Style::default()
                    .fg(Theme::TEXT_PRIMARY)
                    .bg(Theme::BG_ELEVATED)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(
                " │ ",
                Style::default()
                    .fg(Theme::BORDER_DIM)
                    .bg(Theme::BG_ELEVATED),
            ),
            Span::styled(
                "/help for commands",
                Style::default()
                    .fg(Theme::TEXT_SECONDARY)
                    .bg(Theme::BG_ELEVATED)
                    .add_modifier(Modifier::ITALIC),
            ),
        ]),
    };

    let paragraph = Paragraph::new(line).style(bg_style);
    frame.render_widget(paragraph, area);
}
