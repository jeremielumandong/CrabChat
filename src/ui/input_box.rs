use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::block::Padding;
use ratatui::widgets::{Block, Borders, Clear, List, ListItem, Paragraph};

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

    // Render autocomplete popup above the input box
    if state.autocomplete.visible && !state.autocomplete.suggestions.is_empty() {
        let count = state.autocomplete.suggestions.len();
        let visible_count = count.min(8);
        let popup_height = visible_count as u16 + 2; // +2 for borders
        let popup_width = 25u16.min(area.width);

        // Position directly above the input box, aligned to inner left
        if area.y >= popup_height {
            let popup_area = Rect::new(inner.x, area.y - popup_height, popup_width, popup_height);

            frame.render_widget(Clear, popup_area);

            let popup_block = Block::default()
                .borders(Borders::ALL)
                .border_type(Theme::border_type_focused())
                .border_style(Style::default().fg(Theme::BORDER_FOCUS))
                .style(Style::default().bg(Theme::BG_ELEVATED));

            let items: Vec<ListItem> = state
                .autocomplete
                .suggestions
                .iter()
                .enumerate()
                .map(|(i, cmd)| {
                    let style = if i == state.autocomplete.selected {
                        Style::default().fg(Theme::BG_DARK).bg(Theme::ACCENT_TEAL)
                    } else {
                        Style::default().fg(Theme::TEXT_PRIMARY)
                    };
                    ListItem::new(format!("/{}", cmd)).style(style)
                })
                .collect();

            let list = List::new(items).block(popup_block);
            frame.render_widget(list, popup_area);
        }
    }
}
