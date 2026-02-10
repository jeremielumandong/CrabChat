use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::UserList;
    let border_style = if focused {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let block = Block::default()
        .title(" Users ")
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_style(border_style);

    let mut items: Vec<ListItem> = Vec::new();

    if let Some(BufferKey::Channel(server_id, ref channel)) = state.active_buffer {
        if let Some(srv) = state.get_server(server_id) {
            if let Some(users) = srv.users.get(channel) {
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
                    rank(a).cmp(&rank(b)).then_with(|| a.nick.to_lowercase().cmp(&b.nick.to_lowercase()))
                });

                for user in sorted {
                    let style = match user.prefix.as_str() {
                        "@" | "~" | "&" => Theme::user_op(),
                        "+" | "%" => Theme::user_voice(),
                        _ => Theme::user_normal(),
                    };
                    items.push(ListItem::new(Span::styled(user.display_name(), style)));
                }
            }
        }
    }

    if items.is_empty() {
        items.push(ListItem::new(Span::styled(
            " —",
            Style::default().fg(Color::DarkGray),
        )));
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
