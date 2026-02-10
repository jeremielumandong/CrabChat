use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::ServerTree;
    let (border_style, border_type, bg) = if focused {
        (
            Theme::border_focused(),
            Theme::border_type_focused(),
            Theme::panel_bg_focused(),
        )
    } else {
        (Theme::border(), Theme::border_type(), Theme::panel_bg())
    };

    let block = Block::default()
        .title(" Servers ")
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .style(bg);

    let mut items: Vec<ListItem> = Vec::new();

    // Highlights entry at the top
    {
        let hl_key = BufferKey::Highlights;
        let buf = state.buffers.get(&hl_key);
        let is_active = state.active_buffer.as_ref() == Some(&hl_key);
        let has_mention = buf.map(|b| b.has_mention).unwrap_or(false);
        let unread = buf.map(|b| b.unread_count).unwrap_or(0);

        let style = if is_active {
            Style::default()
                .fg(Theme::ACCENT_AMBER)
                .add_modifier(Modifier::BOLD)
                .bg(Theme::BG_ELEVATED)
        } else if has_mention {
            Theme::channel_mention()
        } else if unread > 0 {
            Theme::channel_unread()
        } else {
            Style::default().fg(Theme::TEXT_SECONDARY)
        };

        let mut spans = vec![
            Span::styled(
                " ★ ",
                Style::default()
                    .fg(Theme::ACCENT_AMBER)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled("Highlights", style),
        ];

        if unread > 0 && !is_active {
            spans.push(Span::styled(
                format!(" {}", unread),
                Style::default()
                    .fg(Theme::BG_DARK)
                    .bg(Theme::ACCENT_AMBER)
                    .add_modifier(Modifier::BOLD),
            ));
        }

        items.push(ListItem::new(Line::from(spans)));
    }

    for srv in &state.servers {
        // Server line with diamond status icons
        let (indicator, style) = match srv.status {
            ConnectionStatus::Connected => ("◆", Theme::server_connected()),
            ConnectionStatus::Connecting => ("◇", Theme::server_connecting()),
            ConnectionStatus::Disconnected => ("◈", Theme::server_disconnected()),
        };

        let is_active_server_status = state.active_buffer == Some(BufferKey::ServerStatus(srv.id));

        let server_style = if is_active_server_status {
            style.add_modifier(Modifier::BOLD).bg(Theme::BG_ELEVATED)
        } else {
            style
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!(" {} ", indicator), style),
            Span::styled(&srv.name, server_style),
        ])));

        let total_entries = srv.channels.len()
            + state
                .buffers
                .keys()
                .filter(|k| matches!(k, BufferKey::Query(sid, _) if *sid == srv.id))
                .count();

        // Channel lines with tree indentation
        for (i, ch) in srv.channels.iter().enumerate() {
            let key = BufferKey::Channel(srv.id, ch.clone());
            let buf = state.buffers.get(&key);

            let is_active = state.active_buffer.as_ref() == Some(&key);
            let has_mention = buf.map(|b| b.has_mention).unwrap_or(false);
            let unread = buf.map(|b| b.unread_count).unwrap_or(0);

            let is_last_in_tree = i == srv.channels.len() - 1
                && !state
                    .buffers
                    .keys()
                    .any(|k| matches!(k, BufferKey::Query(sid, _) if *sid == srv.id));

            let tree_prefix = if is_last_in_tree && i + 1 == total_entries {
                " └─"
            } else {
                " ├─"
            };

            let ch_style = if is_active {
                Theme::channel_active().bg(Theme::BG_ELEVATED)
            } else if has_mention {
                Theme::channel_mention()
            } else if unread > 0 {
                Theme::channel_unread()
            } else {
                Theme::channel_normal()
            };

            let mut spans = vec![
                Span::styled(tree_prefix, Style::default().fg(Theme::BORDER_DIM)),
                Span::styled(ch, ch_style),
            ];

            // Unread badge
            if unread > 0 && !is_active {
                spans.push(Span::styled(
                    format!(" {}", unread),
                    Style::default()
                        .fg(Theme::BG_DARK)
                        .bg(Theme::ACCENT_AMBER)
                        .add_modifier(Modifier::BOLD),
                ));
            }

            items.push(ListItem::new(Line::from(spans)));
        }

        // Query lines with tree indentation
        let query_keys: Vec<_> = state
            .buffers
            .keys()
            .filter(|k| matches!(k, BufferKey::Query(sid, _) if *sid == srv.id))
            .cloned()
            .collect();

        for (qi, key) in query_keys.iter().enumerate() {
            if let BufferKey::Query(_, ref target) = key {
                let buf = state.buffers.get(key);
                let is_active = state.active_buffer.as_ref() == Some(key);
                let unread = buf.map(|b| b.unread_count).unwrap_or(0);

                let is_last = qi == query_keys.len() - 1;
                let tree_prefix = if is_last { " └─" } else { " ├─" };

                let style = if is_active {
                    Theme::channel_active().bg(Theme::BG_ELEVATED)
                } else if unread > 0 {
                    Theme::channel_unread()
                } else {
                    Theme::channel_normal()
                };

                let mut spans = vec![
                    Span::styled(tree_prefix, Style::default().fg(Theme::BORDER_DIM)),
                    Span::styled(target.clone(), style),
                ];

                if unread > 0 && !is_active {
                    spans.push(Span::styled(
                        format!(" {}", unread),
                        Style::default()
                            .fg(Theme::BG_DARK)
                            .bg(Theme::ACCENT_AMBER)
                            .add_modifier(Modifier::BOLD),
                    ));
                }

                items.push(ListItem::new(Line::from(spans)));
            }
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            " No servers",
            Style::default().fg(Theme::TEXT_MUTED),
        )));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
