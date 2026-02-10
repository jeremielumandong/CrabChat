use crate::app::state::AppState;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{
    Block, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

/// Network descriptions for known servers (keyed by name).
fn network_description(name: &str) -> &'static str {
    match name {
        "libera" => "FOSS & open-source community hub",
        "oftc" => "Open/free technology community",
        "efnet" => "Original IRC network, est. 1990",
        "undernet" => "One of the largest classic networks",
        "dalnet" => "User-friendly, services-heavy network",
        "rizon" => "Anime, XDCC & file-sharing community",
        "quakenet" => "Gaming-focused European network",
        "ircnet" => "European old-school IRC network",
        "snoonet" => "Reddit community IRC network",
        "gamesurge" => "Multiplayer gaming community",
        "esper" => "Gaming & Minecraft community",
        "irc-hispano" => "Largest Spanish-language network",
        "hackint" => "Hacker & privacy-focused network",
        "twitch" => "Twitch.tv chat via IRC",
        "slashnet" => "Slashdot & tech community",
        "chatspike" => "InspIRCd development network",
        "rezosup" => "French academic network",
        "chathispano" => "Spanish chat community",
        "europnet" => "European multilingual network",
        "interlinked" => "Modern community network",
        _ => "",
    }
}

pub fn render(frame: &mut Frame, state: &AppState) {
    if !state.server_browser.visible {
        return;
    }

    let area = frame.area();

    // Center the popup: 70% width, 80% height, min 60x20
    let popup_w = (area.width * 70 / 100)
        .max(60)
        .min(area.width.saturating_sub(4));
    let popup_h = (area.height * 80 / 100)
        .max(20)
        .min(area.height.saturating_sub(2));
    let popup_x = (area.width.saturating_sub(popup_w)) / 2;
    let popup_y = (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(popup_x, popup_y, popup_w, popup_h);

    // Clear background
    frame.render_widget(Clear, popup_area);

    let block = Block::default()
        .title(" Server Browser — Enter to connect, Esc to close ")
        .title_style(Theme::title())
        .borders(Borders::ALL)
        .border_type(Theme::border_type())
        .border_style(Style::default().fg(Theme::ACCENT_TEAL))
        .style(Style::default().bg(Theme::BG_SURFACE));

    let inner = block.inner(popup_area);
    frame.render_widget(block, popup_area);

    if inner.height < 3 || inner.width < 20 {
        return;
    }

    // Header row
    let header_area = Rect::new(inner.x, inner.y, inner.width, 1);
    let header = Line::from(vec![
        Span::styled("  ", Style::default()),
        Span::styled(
            format!(
                "{:<16} {:<30} {:<6} {}",
                "Network", "Host", "Port", "Description"
            ),
            Style::default()
                .fg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD),
        ),
    ]);
    frame.render_widget(Paragraph::new(header), header_area);

    // Separator
    let sep_area = Rect::new(inner.x, inner.y + 1, inner.width, 1);
    let sep = Paragraph::new(Line::from(Span::styled(
        "─".repeat(inner.width as usize),
        Style::default().fg(Theme::BORDER_DIM),
    )));
    frame.render_widget(sep, sep_area);

    // List area
    let list_h = (inner.height as usize).saturating_sub(4); // header + sep + footer + help
    let list_area = Rect::new(
        inner.x,
        inner.y + 2,
        inner.width.saturating_sub(1),
        list_h as u16,
    );

    let servers = &state.config.servers;
    let browser = &state.server_browser;

    // Build visible rows
    let mut lines: Vec<Line> = Vec::new();
    let start = browser.scroll_offset;
    let end = (start + list_h).min(servers.len());

    for (i, srv) in servers.iter().enumerate().take(end).skip(start) {
        let is_selected = i == browser.selected;
        let is_connected = state.servers.iter().any(|s| {
            s.host == srv.host && s.status == crate::app::state::ConnectionStatus::Connected
        });
        let desc = network_description(&srv.name);

        let tls_icon = if srv.tls { "🔒" } else { "  " };
        let status_icon = if is_connected { "●" } else { " " };

        let line_text = format!(
            "{} {}{:<14} {:<28} {:<6} {}",
            status_icon, tls_icon, srv.name, srv.host, srv.port, desc
        );

        let style = if is_selected {
            Style::default()
                .fg(Theme::BG_DARK)
                .bg(Theme::ACCENT_TEAL)
                .add_modifier(Modifier::BOLD)
        } else if is_connected {
            Style::default().fg(Theme::ACCENT_GREEN)
        } else {
            Style::default().fg(Theme::TEXT_PRIMARY)
        };

        lines.push(Line::from(Span::styled(line_text, style)));
    }

    let list_paragraph = Paragraph::new(lines);
    frame.render_widget(list_paragraph, list_area);

    // Scrollbar
    if servers.len() > list_h {
        let scrollbar_area = Rect::new(
            inner.x + inner.width.saturating_sub(1),
            inner.y + 2,
            1,
            list_h as u16,
        );
        let mut scrollbar_state = ScrollbarState::new(servers.len().saturating_sub(list_h))
            .position(browser.scroll_offset);
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .thumb_style(Theme::scrollbar_thumb())
                .track_style(Theme::scrollbar_track()),
            scrollbar_area,
            &mut scrollbar_state,
        );
    }

    // Footer / help line
    let footer_area = Rect::new(inner.x, inner.y + inner.height - 2, inner.width, 1);
    let count = servers.len();
    let footer = Line::from(vec![
        Span::styled(
            format!(" {} networks available", count),
            Style::default().fg(Theme::TEXT_SECONDARY),
        ),
        Span::styled("  ● = connected", Style::default().fg(Theme::ACCENT_GREEN)),
        Span::styled("  🔒 = TLS", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]);
    frame.render_widget(Paragraph::new(footer), footer_area);

    // Keybinding help
    let help_area = Rect::new(inner.x, inner.y + inner.height - 1, inner.width, 1);
    let help = Line::from(vec![
        Span::styled(
            " ↑↓",
            Style::default()
                .fg(Theme::ACCENT_AMBER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Navigate  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled(
            "Enter",
            Style::default()
                .fg(Theme::ACCENT_AMBER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Connect  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Theme::ACCENT_AMBER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" Close  ", Style::default().fg(Theme::TEXT_SECONDARY)),
        Span::styled(
            "L",
            Style::default()
                .fg(Theme::ACCENT_AMBER)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" List channels", Style::default().fg(Theme::TEXT_SECONDARY)),
    ]);
    frame.render_widget(Paragraph::new(help), help_area);
}
