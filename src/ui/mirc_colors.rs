use ratatui::prelude::*;
use ratatui::style::{Color, Modifier, Style};

/// mIRC 16-color palette
const MIRC_PALETTE: [Color; 16] = [
    Color::Rgb(255, 255, 255), // 0  White
    Color::Rgb(0, 0, 0),       // 1  Black
    Color::Rgb(0, 0, 127),     // 2  Dark Blue
    Color::Rgb(0, 147, 0),     // 3  Dark Green
    Color::Rgb(255, 0, 0),     // 4  Red
    Color::Rgb(127, 0, 0),     // 5  Dark Red
    Color::Rgb(156, 0, 156),   // 6  Purple
    Color::Rgb(252, 127, 0),   // 7  Orange
    Color::Rgb(255, 255, 0),   // 8  Yellow
    Color::Rgb(0, 252, 0),     // 9  Light Green
    Color::Rgb(0, 147, 147),   // 10 Teal
    Color::Rgb(0, 255, 255),   // 11 Light Cyan
    Color::Rgb(0, 0, 252),     // 12 Light Blue
    Color::Rgb(255, 0, 255),   // 13 Pink
    Color::Rgb(127, 127, 127), // 14 Dark Gray
    Color::Rgb(210, 210, 210), // 15 Light Gray
];

/// Parse mIRC-formatted text into styled spans.
pub fn parse_mirc_formatted(text: &str, base_style: Style) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut current_style = base_style;
    let mut current_text = String::new();
    let bytes = text.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        match bytes[i] {
            // Bold toggle
            0x02 => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                if current_style.add_modifier.contains(Modifier::BOLD) {
                    current_style = current_style.remove_modifier(Modifier::BOLD);
                } else {
                    current_style = current_style.add_modifier(Modifier::BOLD);
                }
                i += 1;
            }
            // Italic toggle
            0x1D => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                if current_style.add_modifier.contains(Modifier::ITALIC) {
                    current_style = current_style.remove_modifier(Modifier::ITALIC);
                } else {
                    current_style = current_style.add_modifier(Modifier::ITALIC);
                }
                i += 1;
            }
            // Underline toggle
            0x1F => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                if current_style.add_modifier.contains(Modifier::UNDERLINED) {
                    current_style = current_style.remove_modifier(Modifier::UNDERLINED);
                } else {
                    current_style = current_style.add_modifier(Modifier::UNDERLINED);
                }
                i += 1;
            }
            // Color code
            0x03 => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                i += 1;
                // Parse foreground color (1-2 digits)
                let fg_start = i;
                while i < len && i - fg_start < 2 && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if i > fg_start {
                    let fg_num: usize = std::str::from_utf8(&bytes[fg_start..i])
                        .unwrap_or("0")
                        .parse()
                        .unwrap_or(0);
                    if fg_num < 16 {
                        current_style = current_style.fg(MIRC_PALETTE[fg_num]);
                    }
                    // Parse optional background color
                    if i < len && bytes[i] == b',' {
                        i += 1;
                        let bg_start = i;
                        while i < len && i - bg_start < 2 && bytes[i].is_ascii_digit() {
                            i += 1;
                        }
                        if i > bg_start {
                            let bg_num: usize = std::str::from_utf8(&bytes[bg_start..i])
                                .unwrap_or("0")
                                .parse()
                                .unwrap_or(0);
                            if bg_num < 16 {
                                current_style = current_style.bg(MIRC_PALETTE[bg_num]);
                            }
                        }
                    }
                } else {
                    // Bare \x03 resets colors
                    current_style = Style {
                        fg: base_style.fg,
                        bg: base_style.bg,
                        ..current_style
                    };
                }
            }
            // Reset all formatting
            0x0F => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                current_style = base_style;
                i += 1;
            }
            // Reverse colors
            0x16 => {
                if !current_text.is_empty() {
                    spans.push(Span::styled(std::mem::take(&mut current_text), current_style));
                }
                let old_fg = current_style.fg;
                let old_bg = current_style.bg;
                if let Some(bg) = old_bg {
                    current_style = current_style.fg(bg);
                }
                if let Some(fg) = old_fg {
                    current_style = current_style.bg(fg);
                }
                i += 1;
            }
            _ => {
                current_text.push(bytes[i] as char);
                i += 1;
            }
        }
    }

    if !current_text.is_empty() {
        spans.push(Span::styled(current_text, current_style));
    }

    if spans.is_empty() {
        spans.push(Span::styled(String::new(), base_style));
    }

    spans
}

/// URL style
fn url_style() -> Style {
    Style::default()
        .fg(Color::Rgb(100, 180, 255))
        .add_modifier(Modifier::UNDERLINED)
}

/// Post-process spans to detect and highlight URLs.
pub fn highlight_urls(spans: Vec<Span<'static>>) -> Vec<Span<'static>> {
    let mut result = Vec::new();
    let url_prefixes = ["http://", "https://", "ftp://", "www."];

    for span in spans {
        let text = span.content.to_string();
        let style = span.style;

        let mut remaining = text.as_str();
        let mut has_url = false;

        while !remaining.is_empty() {
            // Find earliest URL
            let mut earliest_pos = None;
            for prefix in &url_prefixes {
                if let Some(pos) = remaining.find(prefix) {
                    if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                        earliest_pos = Some(pos);
                    }
                }
            }

            if let Some(pos) = earliest_pos {
                has_url = true;
                // Text before URL
                if pos > 0 {
                    result.push(Span::styled(remaining[..pos].to_string(), style));
                }

                // Find end of URL (space, >, ", or end of string)
                let url_start = pos;
                let url_remaining = &remaining[url_start..];
                let url_end = url_remaining
                    .find(|c: char| c.is_whitespace() || c == '>' || c == '"' || c == ')')
                    .unwrap_or(url_remaining.len());

                let url = &remaining[url_start..url_start + url_end];
                result.push(Span::styled(url.to_string(), url_style()));

                remaining = &remaining[url_start + url_end..];
            } else {
                // No more URLs in this span
                if has_url {
                    result.push(Span::styled(remaining.to_string(), style));
                } else {
                    result.push(Span::styled(remaining.to_string(), style));
                }
                break;
            }
        }
    }

    result
}
