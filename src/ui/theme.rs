use ratatui::style::{Color, Modifier, Style};

pub struct Theme;

impl Theme {
    pub fn border() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn title() -> Style {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    }

    pub fn timestamp() -> Style {
        Style::default().fg(Color::DarkGray)
    }

    pub fn nick_self() -> Style {
        Style::default().fg(Color::Green).add_modifier(Modifier::BOLD)
    }

    pub fn nick_other() -> Style {
        Style::default().fg(Color::Cyan)
    }

    pub fn message_text() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn system_message() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn error_message() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn action_message() -> Style {
        Style::default().fg(Color::Magenta)
    }

    pub fn join_message() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn part_message() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn server_connected() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn server_disconnected() -> Style {
        Style::default().fg(Color::Red)
    }

    pub fn server_connecting() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn channel_normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn channel_active() -> Style {
        Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
    }

    pub fn channel_unread() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn channel_mention() -> Style {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    }

    pub fn user_op() -> Style {
        Style::default().fg(Color::Green)
    }

    pub fn user_voice() -> Style {
        Style::default().fg(Color::Yellow)
    }

    pub fn user_normal() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn input_text() -> Style {
        Style::default().fg(Color::White)
    }

    pub fn status_bar() -> Style {
        Style::default().fg(Color::White).bg(Color::DarkGray)
    }

    pub fn topic_bar() -> Style {
        Style::default().fg(Color::Cyan)
    }
}
