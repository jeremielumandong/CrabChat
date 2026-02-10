use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let bar_bg = Style::default().bg(Theme::STATUSBAR_BG);
    let seg_style = |fg: Color| Style::default().fg(fg).bg(Theme::STATUS_SEG_BG);
    let sep = Span::styled(
        " │ ",
        Style::default()
            .fg(Theme::BORDER_DIM)
            .bg(Theme::STATUSBAR_BG),
    );

    let mut parts: Vec<Span> = Vec::new();

    // Nick segment pill
    if let Some(server_id) = state.active_server_id() {
        if let Some(srv) = state.get_server(server_id) {
            parts.push(Span::styled(
                format!(" {} ", srv.nickname),
                seg_style(Theme::ACCENT_GREEN).add_modifier(Modifier::BOLD),
            ));
            if srv.is_away {
                parts.push(Span::styled(" [AWAY] ", seg_style(Theme::ACCENT_AMBER)));
            }
            parts.push(sep.clone());
        }
    }

    // Status text
    parts.push(Span::styled(
        format!(" {} ", state.status_line()),
        Style::default()
            .fg(Theme::TEXT_PRIMARY)
            .bg(Theme::STATUSBAR_BG),
    ));

    // Paused indicator
    if let Some(ref buf_key) = state.active_buffer {
        if let Some(buf) = state.buffers.get(buf_key) {
            if buf.paused {
                parts.push(sep.clone());
                parts.push(Span::styled(
                    " PAUSED ",
                    Style::default()
                        .fg(Theme::BG_DARK)
                        .bg(Theme::ACCENT_AMBER)
                        .add_modifier(Modifier::BOLD),
                ));
            }
        }
    }

    // DCC transfer info with inline progress bar
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

            parts.push(sep.clone());

            // Inline progress bar: 10 chars wide
            let bar_width = 10u32;
            let filled = (pct * bar_width / 100).min(bar_width);
            let empty = bar_width - filled;
            let bar: String = "█".repeat(filled as usize) + &"░".repeat(empty as usize);

            parts.push(Span::styled(
                format!(" {} ", t.filename),
                Style::default()
                    .fg(Theme::ACCENT_AMBER)
                    .bg(Theme::STATUSBAR_BG),
            ));
            parts.push(Span::styled(
                bar,
                Style::default()
                    .fg(Theme::ACCENT_TEAL)
                    .bg(Theme::STATUSBAR_BG),
            ));
            parts.push(Span::styled(
                format!(" {}% ", pct),
                Style::default()
                    .fg(Theme::TEXT_SECONDARY)
                    .bg(Theme::STATUSBAR_BG),
            ));
        }
    }

    // Focus indicator pill with per-panel icon and color
    let (focus_icon, focus_label, focus_color) = match state.focus {
        FocusPanel::Input => ("✎", "INPUT", Theme::ACCENT_TEAL),
        FocusPanel::ServerTree => ("◆", "SERVERS", Theme::ACCENT_GREEN),
        FocusPanel::MessageArea => ("☰", "MESSAGES", Theme::ACCENT_BLUE),
        FocusPanel::UserList => ("👤", "USERS", Theme::ACCENT_LAVENDER),
    };

    // Pad to fill remaining space
    let used: usize = parts.iter().map(|s| s.content.len()).sum();
    let focus_len = focus_icon.len() + focus_label.len() + 3; // " icon LABEL "
    let remaining = (area.width as usize).saturating_sub(used + focus_len);
    parts.push(Span::styled(" ".repeat(remaining), bar_bg));
    parts.push(Span::styled(
        format!(" {} {} ", focus_icon, focus_label),
        Style::default()
            .fg(focus_color)
            .bg(Theme::STATUS_SEG_BG)
            .add_modifier(Modifier::BOLD),
    ));

    let line = Line::from(parts);
    let paragraph = Paragraph::new(line).style(bar_bg);
    frame.render_widget(paragraph, area);
}
