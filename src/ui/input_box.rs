use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::Input;
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
        .title(" Input ")
        .title_style(if focused {
            Theme::title()
        } else {
            Theme::border()
        })
        .borders(Borders::ALL)
        .border_type(border_type)
        .border_style(border_style)
        .padding(Padding::horizontal(1))
        .style(bg);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = &state.input.text;

    if focused {
        // Prompt chevron + input text
        let line = Line::from(vec![
            Span::styled("❯ ", Style::default().fg(Theme::ACCENT_TEAL)),
            Span::styled(input_text.as_str(), Theme::input_text()),
        ]);
        let paragraph = Paragraph::new(line);
        frame.render_widget(paragraph, inner);

        // Cursor offset: padding(1) + chevron "❯ " (2 chars)
        let prompt_offset = 2u16;
        let cursor_x = inner.x + prompt_offset + state.input.cursor as u16;
        let cursor_y = inner.y;
        frame.set_cursor_position((cursor_x.min(inner.right() - 1), cursor_y));
    } else {
        let paragraph = Paragraph::new(input_text.as_str()).style(Theme::input_text());
        frame.render_widget(paragraph, inner);
    }
}
