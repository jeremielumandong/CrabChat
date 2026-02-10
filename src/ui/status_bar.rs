use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let mut parts: Vec<Span> = Vec::new();

    // Active server nick
    if let Some(server_id) = state.active_server_id() {
        if let Some(srv) = state.get_server(server_id) {
            parts.push(Span::styled(
                format!(" [{}] ", srv.nickname),
                Style::default().fg(Color::Green).bg(Color::DarkGray),
            ));
        }
    }

    // Status text
    parts.push(Span::styled(
        format!(" {} ", state.status_line()),
        Theme::status_bar(),
    ));

    // DCC transfer info
    let active_transfers: Vec<_> = state
        .transfers
        .iter()
        .filter(|t| t.status == DccTransferStatus::Active)
        .collect();

    if !active_transfers.is_empty() {
        for t in &active_transfers {
            let pct = if t.size > 0 {
                (t.received as f64 / t.size as f64 * 100.0) as u32
            } else {
                0
            };
            parts.push(Span::styled(
                format!(" | DCC:{} {}% ", t.filename, pct),
                Style::default().fg(Color::Yellow).bg(Color::DarkGray),
            ));
        }
    }

    // Focus indicator
    let focus_name = match state.focus {
        FocusPanel::Input => "INPUT",
        FocusPanel::ServerTree => "SERVERS",
        FocusPanel::MessageArea => "MESSAGES",
        FocusPanel::UserList => "USERS",
    };
    // Pad to fill remaining space
    let used: usize = parts.iter().map(|s| s.content.len()).sum();
    let remaining = (area.width as usize).saturating_sub(used + focus_name.len() + 3);
    parts.push(Span::styled(
        " ".repeat(remaining),
        Theme::status_bar(),
    ));
    parts.push(Span::styled(
        format!(" [{}] ", focus_name),
        Style::default().fg(Color::Cyan).bg(Color::DarkGray),
    ));

    let line = Line::from(parts);
    let paragraph = Paragraph::new(line);
    frame.render_widget(paragraph, area);
}
