use crate::app::state::*;
use crate::ui::mirc_colors;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap,
};

const LOGO: [&str; 5] = [
    r"  ____           _      ____ _           _   ",
    r" / ___|_ __ __ _| |__  / ___| |__   __ _| |_ ",
    r"| |   | '__/ _` | '_ \| |   | '_ \ / _` | __|",
    r"| |___| | | (_| | |_) | |___| | | | (_| | |_ ",
    r" \____|_|  \__,_|_.__/ \____|_| |_|\__,_|\__|",
];

fn wave_color(col: u16, tick: u64) -> Color {
    let gradient: [(f64, f64, f64); 6] = [
        (80.0, 200.0, 210.0),  // teal
        (100.0, 170.0, 230.0), // blue
        (175.0, 140.0, 220.0), // lavender
        (220.0, 150.0, 180.0), // pink
        (230.0, 180.0, 80.0),  // amber
        (90.0, 210.0, 130.0),  // green
    ];
    let len = gradient.len() as f64;
    let phase = (col as f64 * 0.15 - tick as f64 * 0.12).rem_euclid(len);
    let idx = phase.floor() as usize;
    let frac = phase - phase.floor();
    let (r1, g1, b1) = gradient[idx % gradient.len()];
    let (r2, g2, b2) = gradient[(idx + 1) % gradient.len()];
    Color::Rgb(
        (r1 + (r2 - r1) * frac) as u8,
        (g1 + (g2 - g1) * frac) as u8,
        (b1 + (b2 - b1) * frac) as u8,
    )
}

fn is_welcome_screen(state: &AppState, key: &BufferKey) -> bool {
    if let BufferKey::ServerStatus(id) = key {
        if let Some(srv) = state.get_server(*id) {
            return srv.name == "welcome" && srv.host.is_empty();
        }
    }
    false
}

fn render_welcome(frame: &mut Frame, area: Rect, state: &AppState) {
    let tick = state.tick_count;
    let logo_h = LOGO.len() as u16;
    let logo_w = LOGO.iter().map(|l| l.len()).max().unwrap_or(0) as u16;

    // Build help lines
    let mut help_lines: Vec<Line> = Vec::new();

    if !state.config.servers.is_empty() {
        help_lines.push(Line::from(Span::styled(
            "Built-in servers:",
            Style::default()
                .fg(Theme::TEXT_PRIMARY)
                .add_modifier(Modifier::BOLD),
        )));
        for srv in &state.config.servers {
            help_lines.push(Line::from(Span::styled(
                format!("  {}  ({}:{})", srv.name, srv.host, srv.port),
                Style::default().fg(Theme::TEXT_SECONDARY),
            )));
        }
        help_lines.push(Line::from(""));
    }

    help_lines.push(Line::from(vec![
        Span::styled(
            "  F2 ",
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Server browser", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]));
    help_lines.push(Line::from(vec![
        Span::styled(
            "  F3 ",
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Channel list (when connected)",
            Style::default().fg(Theme::TEXT_SECONDARY),
        ),
    ]));
    help_lines.push(Line::from(vec![
        Span::styled(
            "  /server connect <name> ",
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Quick connect", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]));
    help_lines.push(Line::from(vec![
        Span::styled(
            "  /server add <name> <host:port> ",
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled("Add server", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]));
    help_lines.push(Line::from(vec![
        Span::styled(
            "  /help ",
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(
            "Show all commands",
            Style::default().fg(Theme::TEXT_SECONDARY),
        ),
    ]));

    // subtitle + gap + help
    let total_h = logo_h + 2 + 1 + help_lines.len() as u16;
    let start_y = area.y + area.height.saturating_sub(total_h) / 3;

    // Render animated logo
    for (i, line) in LOGO.iter().enumerate() {
        let y = start_y + i as u16;
        if y >= area.y + area.height {
            return;
        }

        let line_w = line.len() as u16;
        let x = area.x + area.width.saturating_sub(line_w) / 2;

        let spans: Vec<Span> = line
            .chars()
            .enumerate()
            .map(|(c, ch)| {
                if ch == ' ' {
                    Span::raw(" ")
                } else {
                    Span::styled(
                        ch.to_string(),
                        Style::default()
                            .fg(wave_color(c as u16, tick))
                            .add_modifier(Modifier::BOLD),
                    )
                }
            })
            .collect();

        frame.render_widget(
            Paragraph::new(Line::from(spans)),
            Rect::new(x, y, line_w.min(area.width), 1),
        );
    }

    // Subtitle
    let sub_y = start_y + logo_h + 1;
    if sub_y < area.y + area.height {
        let sub_text = "Secure IRC \u{2022} Terminal Client";
        let sub_w = sub_text.len() as u16;
        let sub_x = area.x + area.width.saturating_sub(sub_w) / 2;
        frame.render_widget(
            Paragraph::new(Span::styled(
                sub_text,
                Style::default().fg(Theme::TEXT_SECONDARY),
            )),
            Rect::new(sub_x, sub_y, sub_w.min(area.width), 1),
        );
    }

    // Help block (centered as a unit, left-aligned within)
    let help_y = sub_y + 2;
    if help_y < area.y + area.height {
        let max_w = help_lines.iter().map(|l| l.width()).max().unwrap_or(0) as u16;
        let max_w = max_w.max(logo_w); // at least as wide as the logo
        let help_x = area.x + area.width.saturating_sub(max_w) / 2;
        let remaining_h = (area.y + area.height).saturating_sub(help_y);
        let help_area = Rect::new(help_x, help_y, max_w.min(area.width), remaining_h);
        frame.render_widget(Paragraph::new(help_lines), help_area);
    }
}

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::MessageArea;
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
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .style(bg);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref buf_key) = state.active_buffer else {
        let empty =
            Paragraph::new("No active buffer. Use /server add <name> <host:port> to connect.")
                .style(Style::default().fg(Theme::TEXT_MUTED));
        frame.render_widget(empty, inner);
        return;
    };

    // Animated welcome screen
    if is_welcome_screen(state, buf_key) {
        render_welcome(frame, inner, state);
        return;
    }

    let Some(buf) = state.buffers.get(buf_key) else {
        return;
    };

    let available_height = inner.height as usize;
    let total = buf.messages.len();

    // Compute visible range with scroll offset
    let end = total.saturating_sub(buf.scroll_offset);
    let start = end.saturating_sub(available_height);

    let our_nick = state
        .active_server_id()
        .and_then(|id| state.get_server(id).map(|s| s.nickname.clone()));

    let parse_colors = state.config.ui.parse_mirc_colors;
    let do_urls = state.config.ui.highlight_urls;

    let lines: Vec<Line> = buf
        .messages
        .iter()
        .skip(start)
        .take(end - start)
        .map(|msg| format_message(msg, our_nick.as_deref(), parse_colors, do_urls))
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);

    // Scrollbar
    if total > available_height {
        let scroll_pos = start;
        let mut scrollbar_state =
            ScrollbarState::new(total.saturating_sub(available_height)).position(scroll_pos);

        let scrollbar = Scrollbar::new(ScrollbarOrientation::VerticalRight)
            .thumb_symbol("┃")
            .track_symbol(Some("│"))
            .thumb_style(Theme::scrollbar_thumb())
            .track_style(Theme::scrollbar_track());

        frame.render_stateful_widget(scrollbar, area, &mut scrollbar_state);
    }
}

fn format_message<'a>(
    msg: &Message,
    our_nick: Option<&str>,
    parse_colors: bool,
    highlight_urls: bool,
) -> Line<'a> {
    let ts = Span::styled(format!("[{}] ", msg.timestamp), Theme::timestamp());

    match msg.kind {
        MessageKind::Normal => {
            let is_self = our_nick
                .map(|n| n.eq_ignore_ascii_case(&msg.sender))
                .unwrap_or(false);
            let nick_style = if is_self {
                Theme::nick_self()
            } else {
                Theme::nick_color(&msg.sender)
            };

            let mut spans = vec![ts, Span::styled(format!("<{}> ", msg.sender), nick_style)];

            let text_spans = if parse_colors {
                mirc_colors::parse_mirc_formatted(&msg.text, Theme::message_text())
            } else {
                vec![Span::styled(msg.text.clone(), Theme::message_text())]
            };

            let text_spans = if highlight_urls {
                mirc_colors::highlight_urls(text_spans)
            } else {
                text_spans
            };

            spans.extend(text_spans);
            Line::from(spans)
        }
        MessageKind::Action => Line::from(vec![
            ts,
            Span::styled(
                format!("* {} {}", msg.sender, msg.text),
                Theme::action_message(),
            ),
        ]),
        MessageKind::System => Line::from(vec![
            ts,
            Span::styled("• ", Style::default().fg(Theme::ACCENT_AMBER)),
            Span::styled(msg.text.clone(), Theme::system_message()),
        ]),
        MessageKind::Error => Line::from(vec![
            ts,
            Span::styled("✘ ", Style::default().fg(Theme::ACCENT_ROSE)),
            Span::styled(msg.text.clone(), Theme::error_message()),
        ]),
        MessageKind::Join => Line::from(vec![
            ts,
            Span::styled(
                format!("→ {} {}", msg.sender, msg.text),
                Theme::join_message(),
            ),
        ]),
        MessageKind::Part | MessageKind::Quit => Line::from(vec![
            ts,
            Span::styled(
                format!("← {} {}", msg.sender, msg.text),
                Theme::part_message(),
            ),
        ]),
        MessageKind::Notice => Line::from(vec![
            ts,
            Span::styled(format!("-{}- ", msg.sender), Theme::notice_message()),
            Span::styled(msg.text.clone(), Theme::notice_message()),
        ]),
    }
}
