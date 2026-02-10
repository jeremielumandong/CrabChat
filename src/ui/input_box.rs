use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::{Block, Borders, Paragraph};

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let focused = state.focus == FocusPanel::Input;
    let border_style = if focused {
        Theme::border_focused()
    } else {
        Theme::border()
    };

    let block = Block::default()
        .title(" Input ")
        .title_style(if focused {
            Theme::title()
        } else {
            Theme::border()
        })
        .borders(Borders::ALL)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let input_text = &state.input.text;
    let paragraph = Paragraph::new(input_text.as_str()).style(Theme::input_text());
    frame.render_widget(paragraph, inner);

    // Set cursor position
    if focused {
        // Calculate cursor position in display coordinates
        let cursor_x = inner.x + state.input.cursor as u16;
        let cursor_y = inner.y;
        frame.set_cursor_position((cursor_x.min(inner.right() - 1), cursor_y));
    }
}
