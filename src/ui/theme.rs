//! Visual theme — the "Midnight Ocean" color palette and style helpers.
//!
//! All colors are defined as `const` RGB values on the [`Theme`] struct. Style
//! helper methods return ready-to-use `ratatui::Style` values so that UI
//! components never hard-code colors directly.

use ratatui::style::{Color, Modifier, Style};
use ratatui::widgets::BorderType;

/// The Midnight Ocean visual theme.
///
/// Provides a dark color palette with teal/green/amber/rose accents, a
/// 12-color pastel nick palette, and style helpers for every UI element.
pub struct Theme;

impl Theme {
    // ── Midnight Ocean palette ──────────────────────────────────────

    pub const BG_DARK: Color = Color::Rgb(22, 22, 30);
    pub const BG_SURFACE: Color = Color::Rgb(30, 32, 44);
    pub const BG_ELEVATED: Color = Color::Rgb(40, 42, 54);
    pub const BORDER_DIM: Color = Color::Rgb(58, 60, 78);
    pub const BORDER_FOCUS: Color = Color::Rgb(100, 200, 220);
    pub const TEXT_PRIMARY: Color = Color::Rgb(205, 210, 225);
    pub const TEXT_SECONDARY: Color = Color::Rgb(120, 125, 145);
    pub const TEXT_MUTED: Color = Color::Rgb(75, 78, 95);
    pub const ACCENT_TEAL: Color = Color::Rgb(80, 200, 210);
    pub const ACCENT_GREEN: Color = Color::Rgb(90, 210, 130);
    pub const ACCENT_AMBER: Color = Color::Rgb(230, 180, 80);
    pub const ACCENT_ROSE: Color = Color::Rgb(220, 95, 110);
    pub const ACCENT_LAVENDER: Color = Color::Rgb(175, 140, 220);
    pub const ACCENT_BLUE: Color = Color::Rgb(100, 140, 230);
    pub const NICK_SELF_COLOR: Color = Color::Rgb(110, 230, 160);
    pub const STATUSBAR_BG: Color = Color::Rgb(25, 27, 38);
    pub const STATUS_SEG_BG: Color = Color::Rgb(45, 48, 65);

    // ── Nick hash colors (12 pastel tones) ──────────────────────────

    const NICK_PALETTE: [Color; 12] = [
        Color::Rgb(220, 150, 150), // soft red
        Color::Rgb(220, 180, 150), // soft orange
        Color::Rgb(220, 210, 150), // soft yellow
        Color::Rgb(170, 220, 150), // soft green
        Color::Rgb(150, 220, 190), // soft teal
        Color::Rgb(150, 210, 220), // soft cyan
        Color::Rgb(150, 170, 220), // soft blue
        Color::Rgb(180, 150, 220), // soft indigo
        Color::Rgb(210, 150, 220), // soft purple
        Color::Rgb(220, 150, 200), // soft pink
        Color::Rgb(200, 190, 220), // soft lavender
        Color::Rgb(150, 220, 160), // soft mint
    ];

    /// Return a deterministic color for a nickname, based on a hash of the
    /// nick string. The same nick always gets the same color.
    pub fn nick_color(nick: &str) -> Style {
        let hash = nick
            .bytes()
            .fold(0u32, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u32));
        let idx = (hash as usize) % Self::NICK_PALETTE.len();
        Style::default().fg(Self::NICK_PALETTE[idx])
    }

    // ── Border helpers ──────────────────────────────────────────────

    pub fn border_type() -> BorderType {
        BorderType::Rounded
    }

    pub fn border_type_focused() -> BorderType {
        BorderType::Double
    }

    // ── Panel backgrounds ───────────────────────────────────────────

    pub fn panel_bg() -> Style {
        Style::default().bg(Self::BG_DARK)
    }

    pub fn panel_bg_focused() -> Style {
        Style::default().bg(Self::BG_SURFACE)
    }

    // ── Scrollbar styles ────────────────────────────────────────────

    pub fn scrollbar_thumb() -> Style {
        Style::default().fg(Self::ACCENT_BLUE)
    }

    pub fn scrollbar_track() -> Style {
        Style::default().fg(Self::BORDER_DIM)
    }

    // ── Gauge styles ────────────────────────────────────────────────

    pub fn gauge_filled() -> Style {
        Style::default().fg(Self::ACCENT_TEAL).bg(Self::BG_ELEVATED)
    }

    pub fn gauge_label() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    // ── Borders ─────────────────────────────────────────────────────

    pub fn border() -> Style {
        Style::default().fg(Self::BORDER_DIM)
    }

    pub fn border_focused() -> Style {
        Style::default().fg(Self::BORDER_FOCUS)
    }

    // ── Titles ──────────────────────────────────────────────────────

    pub fn title() -> Style {
        Style::default()
            .fg(Self::TEXT_PRIMARY)
            .add_modifier(Modifier::BOLD)
    }

    // ── Message styles ──────────────────────────────────────────────

    pub fn timestamp() -> Style {
        Style::default().fg(Self::TEXT_SECONDARY)
    }

    pub fn nick_self() -> Style {
        Style::default()
            .fg(Self::NICK_SELF_COLOR)
            .add_modifier(Modifier::BOLD)
    }

    pub fn message_text() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    pub fn system_message() -> Style {
        Style::default().fg(Self::ACCENT_AMBER)
    }

    pub fn error_message() -> Style {
        Style::default().fg(Self::ACCENT_ROSE)
    }

    pub fn action_message() -> Style {
        Style::default()
            .fg(Self::ACCENT_LAVENDER)
            .add_modifier(Modifier::ITALIC)
    }

    pub fn join_message() -> Style {
        Style::default()
            .fg(Self::ACCENT_GREEN)
            .add_modifier(Modifier::DIM)
    }

    pub fn part_message() -> Style {
        Style::default()
            .fg(Self::ACCENT_ROSE)
            .add_modifier(Modifier::DIM)
    }

    pub fn notice_message() -> Style {
        Style::default().fg(Self::ACCENT_TEAL)
    }

    #[allow(dead_code)]
    pub fn url() -> Style {
        Style::default()
            .fg(Color::Rgb(100, 180, 255))
            .add_modifier(Modifier::UNDERLINED)
    }

    // ── Server status ───────────────────────────────────────────────

    pub fn server_connected() -> Style {
        Style::default().fg(Self::ACCENT_GREEN)
    }

    pub fn server_disconnected() -> Style {
        Style::default()
            .fg(Self::ACCENT_ROSE)
            .add_modifier(Modifier::DIM)
    }

    pub fn server_connecting() -> Style {
        Style::default().fg(Self::ACCENT_AMBER)
    }

    // ── Channel tree ────────────────────────────────────────────────

    pub fn channel_normal() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    pub fn channel_active() -> Style {
        Style::default()
            .fg(Self::ACCENT_TEAL)
            .add_modifier(Modifier::BOLD)
    }

    pub fn channel_unread() -> Style {
        Style::default().fg(Self::ACCENT_AMBER)
    }

    pub fn channel_mention() -> Style {
        Style::default()
            .fg(Self::ACCENT_ROSE)
            .add_modifier(Modifier::BOLD)
    }

    // ── User list ───────────────────────────────────────────────────

    pub fn user_op() -> Style {
        Style::default().fg(Self::ACCENT_GREEN)
    }

    pub fn user_voice() -> Style {
        Style::default().fg(Self::ACCENT_AMBER)
    }

    pub fn user_normal() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }

    // ── Input ───────────────────────────────────────────────────────

    pub fn input_text() -> Style {
        Style::default().fg(Self::TEXT_PRIMARY)
    }
}
