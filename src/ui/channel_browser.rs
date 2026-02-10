use crate::app::state::AppState;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};

pub fn render(frame: &mut Frame, state: &AppState) {
    if !state.channel_browser.visible {
        return;
    }

    let area = frame.area();
    let popup_w = (area.width * 75 / 100).max(60).min(area.width.saturating_sub(4));
    let popup_h = (area.height * 85 / 100).max(20).min(area.height.saturating_sub(2));
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    frame.render_widget(Clear, popup_area);

    let srv_name = state.channel_browser.server_id
        .and_then(|id| state.get_server(id))
        .map(|s| s.name.clone())
        .unwrap_or_default();

    let title = if state.channel_browser.loading {
        format!(" Channel List — {} (loading...) ", srv_name)
    } else {
        format!(" Channel List — {} ({} channels) ", srv_name, state.channel_browser.filtered.len())
    };

    let block = Block::default()
        .title(title)
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_type(Theme::border_type())
        .border_style(Style::default().fg(Theme::ACCENT_LAVENDER))
        .style(Style::default().bg(Theme::BG_SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if inner.height < 4 || inner.width < 20 {
        return;
    }

    let browser = &state.channel_browser;

    // Filter bar
    let filter_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let filter_line = Line::from(vec![
        Span::styled(" Filter: ", Style::default().fg(Theme::ACCENT_AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(
            if browser.filter.is_empty() { "(type to filter channels)" } else { &browser.filter },
            if browser.filter.is_empty() {
                Style::default().fg(Theme::TEXT_MUTED)
            } else {
                Style::default().fg(Theme::TEXT_PRIMARY)
            },
        ),
        Span::styled("█", Style::default().fg(Theme::ACCENT_TEAL)),
    ]);
    frame.render_widget(Paragraph::new(filter_line), filter_area);

    // Header
    let header_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
    let header = Line::from(Span::styled(
        format!("  {:<30} {:>6}  {}", "Channel", "Users", "Topic"),
        Style::default().fg(Theme::ACCENT_LAVENDER).add_modifier(Modifier::BOLD),
    ));
    frame.render_widget(Paragraph::new(header), header_area);

    // Separator
    let sep_area = Rect::new(inner.x, inner.y + 2, inner.width, 1);
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(Theme::BORDER_DIM),
    )));
    frame.render_widget(sep, sep_area);

    // List area
    let list_h = (inner.height as usize).saturating_sub(5); // filter + header + sep + footer + help
    let list_area = Rect::new(inner.x, inner.y + 3, inner.width.saturating_sub(1), list_h as u16);

    if browser.loading && browser.channels.is_empty() {
        let loading = Paragraph::new(Line::from(Span::styled(
            "  Requesting channel list from server...",
            Style::default().fg(Theme::ACCENT_AMBER),
        )));
        frame.render_widget(loading, list_area);
    } else if browser.filtered.is_empty() {
        let empty_msg = if browser.filter.is_empty() {
            "  No channels found."
        } else {
            "  No channels match filter."
        };
        let empty = Paragraph::new(Line::from(Span::styled(
            empty_msg,
            Style::default().fg(Theme::TEXT_MUTED),
        )));
        frame.render_widget(empty, list_area);
    } else {
        let mut lines: Vec<Line> = Vec::new();
        let start = browser.scroll_offset;
        let end = (start + list_h).min(browser.filtered.len());
        let max_topic_w = (inner.width as usize).saturating_sub(42);

        for vis_i in start..end {
            let idx = browser.filtered[vis_i];
            let ch = &browser.channels[idx];
            let is_selected = vis_i == browser.selected;

            // Truncate topic safely at char boundaries
            let topic_display: String = if max_topic_w < 4 {
                String::new()
            } else {
                let chars: Vec<char> = ch.topic.chars().collect();
                if chars.len() > max_topic_w {
                    let truncated: String = chars[..max_topic_w.saturating_sub(3)].iter().collect();
                    format!("{}...", truncated)
                } else {
                    ch.topic.clone()
                }
            };

            // Truncate channel name safely too
            let name_display: String = {
                let chars: Vec<char> = ch.name.chars().collect();
                if chars.len() > 30 {
                    chars[..27].iter().collect::<String>() + "..."
                } else {
                    ch.name.clone()
                }
            };

            let line_text = format!("  {:<30} {:>6}  {}", name_display, ch.users, topic_display);

            let style = if is_selected {
                Style::default().fg(Theme::BG_DARK).bg(Theme::ACCENT_LAVENDER).add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Theme::TEXT_PRIMARY)
            };

            lines.push(Line::from(Span::styled(line_text, style)));
        }

        let list_paragraph = Paragraph::new(lines);
        frame.render_widget(list_paragraph, list_area);

        // Scrollbar
        if browser.filtered.len() > list_h {
            let scrollbar_area = Rect::new(
                inner.x + inner.width.saturating_sub(1),
                inner.y + 3,
                1,
                list_h as u16,
            );
            let mut scrollbar_state = ScrollbarState::new(browser.filtered.len().saturating_sub(list_h))
                .position(browser.scroll_offset);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight)
                    .thumb_style(Theme::scrollbar_thumb())
                    .track_style(Theme::scrollbar_track()),
                scrollbar_area,
                &mut scrollbar_state,
            );
        }
    }

    // Footer
    let footer_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
    let count_text = if browser.loading {
        format!(" {} channels so far...", browser.channels.len())
    } else {
        format!(" {} / {} channels shown", browser.filtered.len(), browser.channels.len())
    };
    let footer = Line::from(Span::styled(count_text, Style::default().fg(Theme::TEXT_SECONDARY)));
    frame.render_widget(Paragraph::new(footer), footer_area);

    // Help
    let help_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    let help = Line::from(vec![
        Span::styled(" ↑↓/PgUp/Dn", Style::default().fg(Theme::ACCENT_AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(" Navigate  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled("Enter", Style::default().fg(Theme::ACCENT_AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(" Join  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled("Ctrl+R", Style::default().fg(Theme::ACCENT_AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(" Refresh  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled("Esc", Style::default().fg(Theme::ACCENT_AMBER).add_modifier(Modifier::BOLD)),
        Span::styled(" Close", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]);
    frame.render_widget(Paragraph::new(help), help_area);
}
