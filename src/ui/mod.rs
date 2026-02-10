mod input_box;
mod layout;
mod message_area;
mod server_tree;
mod status_bar;
mod theme;
mod topic_bar;
mod user_list;

use crate::app::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, List, ListItem};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();
    let app_layout = layout::compute_layout(area);

    server_tree::render(frame, app_layout.server_tree, state);
    topic_bar::render(frame, app_layout.topic_bar, state);
    message_area::render(frame, app_layout.message_area, state);
    input_box::render(frame, app_layout.input_box, state);
    user_list::render(frame, app_layout.user_list, state);
    render_status_panel(frame, app_layout.status_panel, state);
    status_bar::render(frame, app_layout.status_bar, state);
}

fn render_status_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" Status ")
        .title_style(theme::Theme::title())
        .borders(Borders::ALL)
        .border_style(theme::Theme::border());

    let mut items: Vec<ListItem> = Vec::new();

    // Show pending DCC transfers
    let pending: Vec<_> = state
        .transfers
        .iter()
        .filter(|t| {
            matches!(
                t.status,
                crate::app::state::DccTransferStatus::Pending
                    | crate::app::state::DccTransferStatus::Active
            )
        })
        .collect();

    if pending.is_empty() {
        items.push(ListItem::new(Span::styled(
            " No active transfers",
            Style::default().fg(Color::DarkGray),
        )));
    } else {
        for t in pending {
            let pct = if t.size > 0 {
                (t.received as f64 / t.size as f64 * 100.0) as u32
            } else {
                0
            };
            let status_str = match &t.status {
                crate::app::state::DccTransferStatus::Pending => "pending".to_string(),
                crate::app::state::DccTransferStatus::Active => format!("{}%", pct),
                _ => "?".to_string(),
            };
            items.push(ListItem::new(Span::styled(
                format!(" [{}] {} {}", t.id, t.filename, status_str),
                Style::default().fg(Color::Yellow),
            )));
        }
    }

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}
