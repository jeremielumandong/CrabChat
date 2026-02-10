use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::MessageArea;
    let border_style = if focused {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let block = Block::default()
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let Some(ref buf_key) = state.active_buffer else {
        let empty = Paragraph::new("No active buffer. Use /server add <name> <host:port> to connect.")
            .style(Style::default().fg(Color::DarkGray));
        frame.render_widget(empty, inner);
        return;
    };

    let Some(buf) = state.buffers.get(buf_key) else {
        return;
    };

    let available_height = inner.height as usize;
    let total = buf.messages.len();

    // Compute visible range with scroll offset
    let end = total.saturating_sub(buf.scroll_offset);
    let start = end.saturating_sub(available_height);

    let visible = &buf.messages[start..end];

    let our_nick = state.active_server_id().and_then(|id| {
        state.get_server(id).map(|s| s.nickname.clone())
    });

    let lines: Vec<Line> = visible
        .iter()
        .map(|msg| format_message(msg, our_nick.as_deref()))
        .collect();

    let paragraph = Paragraph::new(lines).wrap(Wrap { trim: false });
    frame.render_widget(paragraph, inner);
}

fn format_message<'a>(msg: &Message, our_nick: Option<&str>) -> Line<'a> {
    let ts = Span::styled(
        format!("[{}] ", msg.timestamp),
        Theme::timestamp(),
    );

    match msg.kind {
        MessageKind::Normal => {
            let is_self = our_nick
                .map(|n| n.eq_ignore_ascii_case(&msg.sender))
                .unwrap_or(false);
            let nick_style = if is_self {
                Theme::nick_self()
            } else {
                Theme::nick_other()
            };
            Line::from(vec![
                ts,
                Span::styled(format!("<{}> ", msg.sender), nick_style),
                Span::styled(msg.text.clone(), Theme::message_text()),
            ])
        }
        MessageKind::Action => {
            Line::from(vec![
                ts,
                Span::styled(format!("* {} {}", msg.sender, msg.text), Theme::action_message()),
            ])
        }
        MessageKind::System => {
            Line::from(vec![
                ts,
                Span::styled(msg.text.clone(), Theme::system_message()),
            ])
        }
        MessageKind::Error => {
            Line::from(vec![
                ts,
                Span::styled(msg.text.clone(), Theme::error_message()),
            ])
        }
        MessageKind::Join => {
            Line::from(vec![
                ts,
                Span::styled(
                    format!("→ {} {}", msg.sender, msg.text),
                    Theme::join_message(),
                ),
            ])
        }
        MessageKind::Part | MessageKind::Quit => {
            Line::from(vec![
                ts,
                Span::styled(
                    format!("← {} {}", msg.sender, msg.text),
                    Theme::part_message(),
                ),
            ])
        }
    }
}
