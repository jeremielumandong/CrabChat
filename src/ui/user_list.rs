use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::UserList;
    let (border_style, border_type, bg) = if focused {
        (
            Theme::border_focused(),
            Theme::border_type_focused(),
            Theme::panel_bg_focused(),
        )
    } else {
        (Theme::border(), Theme::border_type(), Theme::panel_bg())
    };

    let mut items: Vec<ListItem> = Vec::new();
    let mut user_count = 0usize;

    if let Some(BufferKey::Channel(server_id, ref channel)) = state.active_buffer {
        if let Some(srv) = state.get_server(server_id) {
            if let Some(users) = srv.users.get(channel) {
                user_count = users.len();

                // Sort users: ops first, then voiced, then normal
                let mut sorted: Vec<_> = users.iter().collect();
                sorted.sort_by(|a, b| {
                    let rank = |u: &ChannelUser| match u.prefix.as_str() {
                        "~" => 0,
                        "&" => 1,
                        "@" => 2,
                        "%" => 3,
                        "+" => 4,
                        _ => 5,
                    };
                    rank(a)
                        .cmp(&rank(b))
                        .then_with(|| a.nick.to_lowercase().cmp(&b.nick.to_lowercase()))
                });

                let mut last_group: Option<u8> = None;

                for user in sorted {
                    let group = match user.prefix.as_str() {
                        "~" | "&" | "@" => 0u8,
                        "+" | "%" => 1,
                        _ => 2,
                    };

                    // Dashed separator between mode groups
                    if let Some(prev) = last_group {
                        if prev != group {
                            items.push(ListItem::new(Span::styled(
                                " ╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌╌",
                                Style::default().fg(Theme::BORDER_DIM),
                            )));
                        }
                    }
                    last_group = Some(group);

                    let (icon, style) = match user.prefix.as_str() {
                        "@" | "~" | "&" => (" ★ ", Theme::user_op()),
                        "+" | "%" => (" ○ ", Theme::user_voice()),
                        _ => ("   ", Theme::user_normal()),
                    };

                    items.push(ListItem::new(Line::from(vec![
                        Span::styled(icon, style),
                        Span::styled(user.nick.clone(), style),
                    ])));
                }
            }
        }
    }

    let title = if user_count > 0 {
        format!(" Users ({}) ", user_count)
    } else {
        " Users ".to_string()
    };

    let block = Block::default()
        .title(title)
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .style(bg);

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            " —",
            Style::default().fg(Theme::TEXT_MUTED),
        )));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
