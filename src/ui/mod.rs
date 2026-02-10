mod channel_browser;
mod input_box;
mod layout;
mod message_area;
pub mod mirc_colors;
mod server_browser;
mod server_tree;
mod status_bar;
mod theme;
mod topic_bar;
mod user_list;

use crate::app::state::AppState;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Gauge, List, ListItem};

pub fn render(frame: &mut Frame, state: &AppState) {
    let area = frame.area();

    // Fill entire background with dark theme color
    let bg = Block::default().style(Style::default().bg(theme::Theme::BG_DARK));
    frame.render_widget(bg, area);

    let app_layout = layout::compute_layout(area);

    server_tree::render(frame, app_layout.server_tree, state);
    topic_bar::render(frame, app_layout.topic_bar, state);
    message_area::render(frame, app_layout.message_area, state);
    input_box::render(frame, app_layout.input_box, state);
    user_list::render(frame, app_layout.user_list, state);
    render_status_panel(frame, app_layout.status_panel, state);
    status_bar::render(frame, app_layout.status_bar, state);

    // Overlay popups (rendered last, on top)
    server_browser::render(frame, state);
    channel_browser::render(frame, state);
}

fn render_status_panel(frame: &mut Frame, area: Rect, state: &AppState) {
    let block = Block::default()
        .title(" ⇅ Transfers ")
        .title_style(theme::Theme::title())
        .borders(Borders::ALL)
        .border_type(theme::Theme::border_type())
        .border_style(theme::Theme::border())
        .style(theme::Theme::panel_bg());

    let inner = block.inner(area);
    frame.render_widget(block, area);

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
        let items = vec![ListItem::new(Span::styled(
            " No active transfers",
            Style::default().fg(theme::Theme::TEXT_MUTED),
        ))];
        let list = List::new(items);
        frame.render_widget(list, inner);
    } else {
        // Use Layout to split inner area for each transfer
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                pending
                    .iter()
                    .map(|_| Constraint::Length(1))
                    .collect::<Vec<_>>(),
            )
            .split(inner);

        for (i, t) in pending.iter().enumerate() {
            if i >= chunks.len() {
                break;
            }

            match &t.status {
                crate::app::state::DccTransferStatus::Pending => {
                    let line = Line::from(vec![
                        Span::styled("⏳ ", Style::default().fg(theme::Theme::ACCENT_AMBER)),
                        Span::styled(
                            format!("[{}] {} pending", t.id, t.filename),
                            Style::default().fg(theme::Theme::ACCENT_AMBER),
                        ),
                    ]);
                    let p = ratatui::widgets::Paragraph::new(line);
                    frame.render_widget(p, chunks[i]);
                }
                crate::app::state::DccTransferStatus::Active => {
                    let ratio = if t.size > 0 {
                        (t.received as f64 / t.size as f64).min(1.0)
                    } else {
                        0.0
                    };
                    let pct = (ratio * 100.0) as u32;
                    let label = format!("{} {}%", t.filename, pct);

                    let gauge = Gauge::default()
                        .ratio(ratio)
                        .label(Span::styled(label, theme::Theme::gauge_label()))
                        .gauge_style(theme::Theme::gauge_filled())
                        .use_unicode(true);

                    frame.render_widget(gauge, chunks[i]);
                }
                _ => {}
            }
        }
    }
}
