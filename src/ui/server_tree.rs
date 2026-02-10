use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::ServerTree;
    let border_style = if focused {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let block = Block::default()
        .title(" Servers ")
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_style(border_style);

    let mut items: Vec<ListItem> = Vec::new();

    for srv in &state.servers {
        // Server line
        let indicator = match srv.status {
            ConnectionStatus::Connected => "●",
            ConnectionStatus::Connecting => "◌",
            ConnectionStatus::Disconnected => "○",
        };
        let style = match srv.status {
            ConnectionStatus::Connected => Theme::server_connected(),
            ConnectionStatus::Connecting => Theme::server_connecting(),
            ConnectionStatus::Disconnected => Theme::server_disconnected(),
        };
        let is_active_server = state.active_buffer.as_ref().map(|k| match k {
            BufferKey::ServerStatus(id) => *id == srv.id,
            BufferKey::Channel(id, _) => *id == srv.id,
            BufferKey::Query(id, _) => *id == srv.id,
        }).unwrap_or(false);

        let server_style = if is_active_server && state.active_buffer == Some(BufferKey::ServerStatus(srv.id)) {
            style.add_modifier(Modifier::BOLD | Modifier::REVERSED)
        } else {
            style
        };

        items.push(ListItem::new(Line::from(vec![
            Span::styled(format!("{} ", indicator), style),
            Span::styled(&srv.name, server_style),
        ])));

        // Channel lines
        for ch in &srv.channels {
            let key = BufferKey::Channel(srv.id, ch.clone());
            let buf = state.buffers.get(&key);

            let is_active = state.active_buffer.as_ref() == Some(&key);
            let has_mention = buf.map(|b| b.has_mention).unwrap_or(false);
            let unread = buf.map(|b| b.unread_count).unwrap_or(0);

            let ch_style = if is_active {
                Theme::channel_active().add_modifier(Modifier::REVERSED)
            } else if has_mention {
                Theme::channel_mention()
            } else if unread > 0 {
                Theme::channel_unread()
            } else {
                Theme::channel_normal()
            };

            let label = if unread > 0 && !is_active {
                format!("  {} ({})", ch, unread)
            } else {
                format!("  {}", ch)
            };

            items.push(ListItem::new(Span::styled(label, ch_style)));
        }

        // Query lines
        for key in state.buffers.keys() {
            if let BufferKey::Query(sid, target) = key {
                if *sid == srv.id {
                    let buf = state.buffers.get(key);
                    let is_active = state.active_buffer.as_ref() == Some(key);
                    let unread = buf.map(|b| b.unread_count).unwrap_or(0);

                    let style = if is_active {
                        Theme::channel_active().add_modifier(Modifier::REVERSED)
                    } else if unread > 0 {
                        Theme::channel_unread()
                    } else {
                        Theme::channel_normal()
                    };

                    let label = if unread > 0 && !is_active {
                        format!("  {} ({})", target, unread)
                    } else {
                        format!("  {}", target)
                    };

                    items.push(ListItem::new(Span::styled(label, style)));
                }
            }
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            " No servers",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
