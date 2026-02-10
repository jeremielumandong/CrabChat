use crate::app::state::*;
use crate::ui::theme::Theme;
use ratatui::prelude::*;
use ratatui::widgets::Paragraph;

pub fn render(frame: &mut Frame, area: Rect, state: &AppState) {
    let text = match &state.active_buffer {
        Some(BufferKey::Channel(server_id, channel)) => {
            let topic = state
                .get_server(*server_id)
                .and_then(|srv| srv.topics.get(channel))
                .map(|t| t.as_str())
                .unwrap_or("");
            if topic.is_empty() {
                format!(" {} — No topic set", channel)
            } else {
                format!(" {} — {}", channel, topic)
            }
        }
        Some(BufferKey::Query(_, target)) => {
            format!(" Query: {}", target)
        }
        Some(BufferKey::ServerStatus(server_id)) => {
            let name = state
                .get_server(*server_id)
                .map(|s| s.name.as_str())
                .unwrap_or("Unknown");
            format!(" Server: {}", name)
        }
        None => " CrabChat — /help for commands".to_string(),
    };

    let paragraph = Paragraph::new(text).style(Theme::topic_bar());
    frame.render_widget(paragraph, area);
}
