use ratatui::layout::{Constraint, Direction, Layout, Rect};

pub struct AppLayout {
    pub server_tree: Rect,
    pub user_list: Rect,
    pub status_panel: Rect,
    pub topic_bar: Rect,
    pub message_area: Rect,
    pub input_box: Rect,
    pub status_bar: Rect,
}

pub fn compute_layout(area: Rect) -> AppLayout {
    // Main vertical split: content | status bar
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Min(5),    // Main content
            Constraint::Length(1), // Status bar
        ])
        .split(area);

    let content = main_chunks[0];
    let status_bar = main_chunks[1];

    // Horizontal: left panel | gap | right content
    let h_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .spacing(1)
        .constraints([
            Constraint::Length(22), // Left panel (wider)
            Constraint::Min(30),    // Right content
        ])
        .split(content);

    let left_panel = h_chunks[0];
    let right_panel = h_chunks[1];

    // Left panel: server tree | user list | status panel
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage(50), // Server tree
            Constraint::Percentage(40), // User list
            Constraint::Min(3),         // Status info
        ])
        .split(left_panel);

    let server_tree = left_chunks[0];
    let user_list = left_chunks[1];
    let status_panel = left_chunks[2];

    // Right panel: topic bar | messages | input
    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(1), // Topic bar
            Constraint::Min(5),    // Messages
            Constraint::Length(3), // Input box
        ])
        .split(right_panel);

    let topic_bar = right_chunks[0];
    let message_area = right_chunks[1];
    let input_box = right_chunks[2];

    AppLayout {
        server_tree,
        user_list,
        status_panel,
        topic_bar,
        message_area,
        input_box,
        status_bar,
    }
}
