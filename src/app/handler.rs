use crate::app::action::Action;
use crate::app::event::{AppEvent, ServerId};
use crate::app::state::*;
use crate::irc::commands;
use chrono::Local;
use crossterm::event::{Event as CEvent, KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

pub fn handle_event(state: &mut AppState, event: AppEvent) -> Vec<Action> {
    let mut actions = match event {
        AppEvent::Terminal(cevent) => {
            state.dirty = true;
            handle_terminal(state, cevent)
        }
        AppEvent::IrcMessage { server_id, message } => {
            handle_irc_message(state, server_id, message);
            vec![]
        }
        AppEvent::IrcConnected { server_id } => {
            if let Some(srv) = state.get_server_mut(server_id) {
                srv.status = ConnectionStatus::Connected;
            }
            let key = BufferKey::ServerStatus(server_id);
            state.system_message(&key, "Connected to server.".to_string());
            vec![]
        }
        AppEvent::IrcDisconnected { server_id, reason } => {
            if let Some(srv) = state.get_server_mut(server_id) {
                srv.status = ConnectionStatus::Disconnected;
            }
            let key = BufferKey::ServerStatus(server_id);
            state.system_message(&key, format!("Disconnected: {}", reason));
            vec![]
        }
        AppEvent::IrcError { server_id, error } => {
            let key = BufferKey::ServerStatus(server_id);
            state.error_message(&key, error);
            vec![]
        }
        AppEvent::DccOfferReceived {
            server_id,
            from,
            filename,
            size,
            ip,
            port,
            transfer_id,
        } => {
            let transfer = DccTransfer {
                id: transfer_id,
                server_id,
                from: from.clone(),
                filename: filename.clone(),
                size,
                received: 0,
                ip,
                port,
                status: DccTransferStatus::Pending,
            };
            state.transfers.push(transfer);
            let key = state
                .active_buffer
                .clone()
                .unwrap_or(BufferKey::ServerStatus(server_id));
            state.system_message(
                &key,
                format!(
                    "DCC SEND offer from {}: \"{}\" ({} bytes) [id: {}] — /dcc accept {} to download",
                    from, filename, size, transfer_id, transfer_id
                ),
            );
            vec![]
        }
        AppEvent::DccProgress {
            transfer_id,
            bytes_received,
            total,
        } => {
            if let Some(t) = state.transfers.iter_mut().find(|t| t.id == transfer_id) {
                t.received = bytes_received;
                t.status = DccTransferStatus::Active;
                state.dirty = true;
            }
            let _ = total;
            vec![]
        }
        AppEvent::DccComplete { transfer_id } => {
            if let Some(t) = state.transfers.iter_mut().find(|t| t.id == transfer_id) {
                t.status = DccTransferStatus::Completed;
                let filename = t.filename.clone();
                let download_dir = state.config.dcc.download_dir.display().to_string();
                let key = state
                    .active_buffer
                    .clone()
                    .unwrap_or(BufferKey::ServerStatus(t.server_id));
                state.system_message(&key, format!("DCC transfer complete: {} (saved to {})", filename, download_dir));
            }
            vec![]
        }
        AppEvent::DccFailed { transfer_id, error } => {
            if let Some(t) = state.transfers.iter_mut().find(|t| t.id == transfer_id) {
                t.status = DccTransferStatus::Failed(error.clone());
                let filename = t.filename.clone();
                let key = state
                    .active_buffer
                    .clone()
                    .unwrap_or(BufferKey::ServerStatus(t.server_id));
                state.error_message(&key, format!("DCC transfer failed ({}): {}", filename, error));
            }
            vec![]
        }
        AppEvent::Tick => {
            handle_tick(state)
        }
    };

    // Drain pending_actions from IRC message handlers
    actions.append(&mut state.pending_actions);
    actions
}

fn handle_tick(state: &mut AppState) -> Vec<Action> {
    let mut actions = Vec::new();
    state.tick_count = state.tick_count.wrapping_add(1);

    // Check pending rejoins
    let now = Instant::now();
    let mut ready_rejoins = Vec::new();
    state.pending_rejoins.retain(|r| {
        if now >= r.rejoin_at {
            ready_rejoins.push((r.server_id, r.channel.clone()));
            false
        } else {
            true
        }
    });
    for (server_id, channel) in ready_rejoins {
        actions.push(Action::JoinChannel { server_id, channel });
    }

    // Notify list: send ISON every 60s
    if !state.notify_list.is_empty() && now.duration_since(state.last_ison_check) > Duration::from_secs(60) {
        state.last_ison_check = now;
        // Find a connected server to send ISON
        if let Some(srv) = state.servers.iter().find(|s| s.status == ConnectionStatus::Connected) {
            let nicks: String = state.notify_list.iter().cloned().collect::<Vec<_>>().join(" ");
            actions.push(Action::SendIson { server_id: srv.id, nicks });
        }
    }

    actions
}

fn handle_terminal(state: &mut AppState, event: CEvent) -> Vec<Action> {
    match event {
        CEvent::Key(key) => handle_key(state, key),
        CEvent::Resize(_, _) => {
            state.dirty = true;
            vec![]
        }
        _ => vec![],
    }
}

fn handle_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    // Global keybindings
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('c') {
        return vec![Action::Quit { message: None }];
    }

    // Channel browser captures all input when visible
    if state.channel_browser.visible {
        return handle_channel_browser_key(state, key);
    }

    // Server browser captures all input when visible
    if state.server_browser.visible {
        return handle_server_browser_key(state, key);
    }

    // F2 to toggle server browser
    if key.code == KeyCode::F(2) {
        state.server_browser.toggle();
        return vec![];
    }

    // F3 to open channel browser on active server
    if key.code == KeyCode::F(3) {
        if let Some(server_id) = state.active_server_id() {
            if state.get_server(server_id).map(|s| s.status == ConnectionStatus::Connected).unwrap_or(false) {
                let needs_fetch = state.channel_browser.open(server_id);
                if needs_fetch {
                    return vec![Action::SendList { server_id }];
                }
            }
        }
        return vec![];
    }

    // Tab to cycle focus (when not in input or input is empty)
    if key.code == KeyCode::Tab && state.focus != FocusPanel::Input {
        state.cycle_focus();
        return vec![];
    }

    match state.focus {
        FocusPanel::Input => handle_input_key(state, key),
        FocusPanel::MessageArea => handle_message_key(state, key),
        FocusPanel::ServerTree => handle_tree_key(state, key),
        FocusPanel::UserList => handle_user_list_key(state, key),
    }
}

fn handle_server_browser_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    let total = state.config.servers.len();
    match key.code {
        KeyCode::Esc => {
            state.server_browser.visible = false;
            vec![]
        }
        KeyCode::Up => {
            state.server_browser.move_up();
            state.server_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::Down => {
            state.server_browser.move_down(total);
            state.server_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::PageUp => {
            for _ in 0..10 {
                state.server_browser.move_up();
            }
            state.server_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::PageDown => {
            for _ in 0..10 {
                state.server_browser.move_down(total);
            }
            state.server_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::Char('l') | KeyCode::Char('L') => {
            // List channels on the selected server (if connected)
            let selected = state.server_browser.selected;
            if let Some(srv_cfg) = state.config.servers.get(selected) {
                let host = srv_cfg.host.clone();
                // Find connected server matching this host
                if let Some(srv) = state.servers.iter().find(|s| s.host == host && s.status == ConnectionStatus::Connected) {
                    let server_id = srv.id;
                    state.server_browser.visible = false;
                    let needs_fetch = state.channel_browser.open(server_id);
                    if needs_fetch {
                        return vec![Action::SendList { server_id }];
                    }
                    return vec![];
                } else {
                    // Not connected — show system message
                    if let Some(ref key) = state.active_buffer.clone() {
                        state.system_message(key, "Connect to this server first to list channels.".to_string());
                    }
                }
            }
            vec![]
        }
        KeyCode::Enter => {
            let selected = state.server_browser.selected;
            if let Some(srv_cfg) = state.config.servers.get(selected).cloned() {
                // Check if already connected
                let already = state.servers.iter().any(|s| s.host == srv_cfg.host && s.status != ConnectionStatus::Disconnected);
                if already {
                    // Switch to that server's buffer instead
                    if let Some(srv) = state.servers.iter().find(|s| s.host == srv_cfg.host) {
                        let key = BufferKey::ServerStatus(srv.id);
                        state.set_active_buffer(key);
                    }
                    state.server_browser.visible = false;
                    return vec![];
                }
                state.server_browser.visible = false;
                return vec![Action::ConnectServer {
                    name: srv_cfg.name,
                    host: srv_cfg.host,
                    port: srv_cfg.port,
                    tls: srv_cfg.tls,
                    nick: srv_cfg.nickname,
                    accept_invalid_certs: srv_cfg.accept_invalid_certs,
                }];
            }
            vec![]
        }
        KeyCode::Home => {
            state.server_browser.selected = 0;
            state.server_browser.scroll_offset = 0;
            vec![]
        }
        KeyCode::End => {
            if total > 0 {
                state.server_browser.selected = total - 1;
                state.server_browser.ensure_visible(20);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn handle_channel_browser_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    // Ctrl+R to force refresh
    if key.modifiers.contains(KeyModifiers::CONTROL) && key.code == KeyCode::Char('r') {
        if let Some(server_id) = state.channel_browser.server_id {
            state.channel_browser.refresh();
            return vec![Action::SendList { server_id }];
        }
        return vec![];
    }

    match key.code {
        KeyCode::Esc => {
            state.channel_browser.close();
            vec![]
        }
        KeyCode::Up => {
            state.channel_browser.move_up();
            state.channel_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::Down => {
            state.channel_browser.move_down();
            state.channel_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::PageUp => {
            for _ in 0..20 {
                state.channel_browser.move_up();
            }
            state.channel_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::PageDown => {
            for _ in 0..20 {
                state.channel_browser.move_down();
            }
            state.channel_browser.ensure_visible(20);
            vec![]
        }
        KeyCode::Enter => {
            if let Some(ch) = state.channel_browser.selected_channel() {
                let channel = ch.name.clone();
                let server_id = state.channel_browser.server_id;
                state.channel_browser.close();
                if let Some(server_id) = server_id {
                    return vec![Action::JoinChannel { server_id, channel }];
                }
            }
            vec![]
        }
        KeyCode::Backspace => {
            state.channel_browser.filter.pop();
            state.channel_browser.apply_filter();
            vec![]
        }
        KeyCode::Char(c) => {
            state.channel_browser.filter.push(c);
            state.channel_browser.apply_filter();
            vec![]
        }
        KeyCode::Home => {
            state.channel_browser.selected = 0;
            state.channel_browser.scroll_offset = 0;
            vec![]
        }
        KeyCode::End => {
            let len = state.channel_browser.filtered.len();
            if len > 0 {
                state.channel_browser.selected = len - 1;
                state.channel_browser.ensure_visible(20);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn handle_input_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Enter => {
            let text = state.input.take_text();
            if text.is_empty() {
                return vec![];
            }
            if text.starts_with('/') {
                return handle_command(state, &text);
            }
            // Send message to active buffer target
            if let Some(ref buf_key) = state.active_buffer.clone() {
                match buf_key {
                    BufferKey::Channel(server_id, channel) => {
                        let server_id = *server_id;
                        let channel = channel.clone();
                        let nick = state
                            .get_server(server_id)
                            .map(|s| s.nickname.clone())
                            .unwrap_or_else(|| "me".to_string());
                        let msg = Message {
                            timestamp: Local::now().format(&state.timestamp_format).to_string(),
                            sender: nick,
                            text: text.clone(),
                            kind: MessageKind::Normal,
                        };
                        state.add_message_to_buffer(buf_key, msg);
                        return vec![Action::SendMessage {
                            server_id,
                            target: channel,
                            text,
                        }];
                    }
                    BufferKey::Query(server_id, target) => {
                        let server_id = *server_id;
                        let target = target.clone();
                        let nick = state
                            .get_server(server_id)
                            .map(|s| s.nickname.clone())
                            .unwrap_or_else(|| "me".to_string());
                        let msg = Message {
                            timestamp: Local::now().format(&state.timestamp_format).to_string(),
                            sender: nick,
                            text: text.clone(),
                            kind: MessageKind::Normal,
                        };
                        state.add_message_to_buffer(buf_key, msg);
                        return vec![Action::SendPrivmsg {
                            server_id,
                            target,
                            text,
                        }];
                    }
                    BufferKey::ServerStatus(_) => {
                        state.system_message(buf_key, "Cannot send messages to server status buffer. Use /msg or join a channel.".to_string());
                    }
                }
            }
            vec![]
        }
        KeyCode::Backspace => {
            if key.modifiers.contains(KeyModifiers::ALT) {
                state.input.delete_word_back();
            } else {
                state.input.delete_back();
            }
            vec![]
        }
        KeyCode::Delete => {
            state.input.delete_forward();
            vec![]
        }
        KeyCode::Left => {
            state.input.move_left();
            vec![]
        }
        KeyCode::Right => {
            state.input.move_right();
            vec![]
        }
        KeyCode::Home => {
            state.input.move_home();
            vec![]
        }
        KeyCode::End => {
            state.input.move_end();
            vec![]
        }
        KeyCode::Up => {
            state.input.history_up();
            vec![]
        }
        KeyCode::Down => {
            state.input.history_down();
            vec![]
        }
        KeyCode::Tab => {
            if state.input.text.is_empty() {
                state.cycle_focus();
            } else if state.input.text.starts_with('/') {
                try_command_completion(state);
            } else {
                try_nick_completion(state);
            }
            vec![]
        }
        KeyCode::PageUp => {
            scroll_up(state);
            vec![]
        }
        KeyCode::PageDown => {
            scroll_down(state);
            vec![]
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) {
                match c {
                    'a' => state.input.move_home(),
                    'e' => state.input.move_end(),
                    'w' => state.input.delete_word_back(),
                    'u' => {
                        state.input.text.clear();
                        state.input.cursor = 0;
                    }
                    _ => {}
                }
            } else {
                state.input.insert_char(c);
            }
            vec![]
        }
        _ => vec![],
    }
}

fn handle_message_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::PageUp | KeyCode::Up => {
            scroll_up(state);
            vec![]
        }
        KeyCode::PageDown | KeyCode::Down => {
            scroll_down(state);
            vec![]
        }
        KeyCode::Tab => {
            state.cycle_focus();
            vec![]
        }
        KeyCode::Char(c) => {
            // Start typing: switch to input
            state.focus = FocusPanel::Input;
            state.input.insert_char(c);
            vec![]
        }
        _ => vec![],
    }
}

fn handle_tree_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Up => {
            state.select_prev_buffer();
            vec![]
        }
        KeyCode::Down => {
            state.select_next_buffer();
            vec![]
        }
        KeyCode::Enter => vec![],
        KeyCode::Tab => {
            state.cycle_focus();
            vec![]
        }
        _ => vec![],
    }
}

fn handle_user_list_key(state: &mut AppState, key: KeyEvent) -> Vec<Action> {
    match key.code {
        KeyCode::Tab => {
            state.cycle_focus();
            vec![]
        }
        _ => vec![],
    }
}

fn scroll_up(state: &mut AppState) {
    if let Some(ref key) = state.active_buffer {
        if let Some(buf) = state.buffers.get_mut(key) {
            let max_scroll = buf.messages.len().saturating_sub(1);
            buf.scroll_offset = (buf.scroll_offset + 5).min(max_scroll);
            state.dirty = true;
        }
    }
}

fn scroll_down(state: &mut AppState) {
    if let Some(ref key) = state.active_buffer {
        if let Some(buf) = state.buffers.get_mut(key) {
            buf.scroll_offset = buf.scroll_offset.saturating_sub(5);
            state.dirty = true;
        }
    }
}

fn try_nick_completion(state: &mut AppState) {
    let word_start = state.input.text[..state.input.cursor]
        .rfind(' ')
        .map(|i| i + 1)
        .unwrap_or(0);
    let partial = &state.input.text[word_start..state.input.cursor];
    if partial.is_empty() {
        return;
    }

    // Get nick list for current channel
    if let Some(BufferKey::Channel(server_id, ref channel)) = state.active_buffer {
        let server_id = server_id;
        let channel = channel.clone();
        if let Some(srv) = state.get_server(server_id) {
            if let Some(users) = srv.users.get(&channel) {
                let partial_lower = partial.to_lowercase();
                let matches: Vec<_> = users
                    .iter()
                    .filter(|u| u.nick.to_lowercase().starts_with(&partial_lower))
                    .collect();
                if let Some(first) = matches.first() {
                    let completion = if word_start == 0 {
                        format!("{}: ", first.nick)
                    } else {
                        format!("{} ", first.nick)
                    };
                    let new_text = format!(
                        "{}{}{}",
                        &state.input.text[..word_start],
                        completion,
                        &state.input.text[state.input.cursor..]
                    );
                    let new_cursor = word_start + completion.len();
                    state.input.text = new_text;
                    state.input.cursor = new_cursor;
                }
            }
        }
    }
}

fn try_command_completion(state: &mut AppState) {
    // Clone text to avoid borrow conflicts
    let text = state.input.text.clone();

    const COMMANDS: &[&str] = &[
        "server", "servers", "join", "part", "nick", "msg", "query", "me", "quit", "exit", "help",
        "dcc", "kick", "ban", "mode", "op", "deop", "voice", "devoice", "topic",
        "notice", "whois", "who", "away", "raw", "quote", "list", "channels", "slap", "ignore",
        "unignore", "ignorelist", "notify", "ctcp", "leave", "browse",
    ];
    const SERVER_SUBCMDS: &[&str] = &["add", "connect", "list", "ls", "disconnect", "dc"];
    const DCC_SUBCMDS: &[&str] = &["list", "ls", "accept", "get", "cancel", "close"];

    let parts: Vec<&str> = text[1..].splitn(3, ' ').collect();
    let cmd = parts.first().unwrap_or(&"").to_lowercase();

    // Case 1: completing the command name itself
    if parts.len() == 1 && !text.ends_with(' ') {
        let partial = &cmd;
        let matches: Vec<&&str> = COMMANDS.iter().filter(|c| c.starts_with(partial.as_str())).collect();
        if let Some(first) = matches.first() {
            state.input.text = format!("/{} ", first);
            state.input.cursor = state.input.text.len();
        }
        return;
    }

    // Case 2: completing arguments based on command
    match cmd.as_str() {
        "server" => {
            let sub = parts.get(1).unwrap_or(&"").to_string();
            if parts.len() == 2 && !text.ends_with(' ') {
                let matches: Vec<&&str> = SERVER_SUBCMDS.iter().filter(|s| s.starts_with(sub.as_str())).collect();
                if let Some(first) = matches.first() {
                    state.input.text = format!("/server {} ", first);
                    state.input.cursor = state.input.text.len();
                }
            } else if sub == "connect" || sub == "disconnect" || sub == "dc" {
                let partial = parts.get(2).unwrap_or(&"").to_lowercase();
                let names: Vec<String> = state.config.servers.iter()
                    .map(|s| s.name.clone())
                    .filter(|n| n.to_lowercase().starts_with(&partial))
                    .collect();
                if let Some(first) = names.first() {
                    state.input.text = format!("/server {} {}", sub, first);
                    state.input.cursor = state.input.text.len();
                }
            }
        }
        "dcc" => {
            let sub = parts.get(1).unwrap_or(&"").to_string();
            if parts.len() == 2 && !text.ends_with(' ') {
                let matches: Vec<&&str> = DCC_SUBCMDS.iter().filter(|s| s.starts_with(sub.as_str())).collect();
                if let Some(first) = matches.first() {
                    state.input.text = format!("/dcc {} ", first);
                    state.input.cursor = state.input.text.len();
                }
            }
        }
        "msg" | "query" | "whois" | "slap" | "ignore" | "unignore" | "notice" | "ctcp"
        | "kick" | "op" | "deop" | "voice" | "devoice" | "ban" => {
            let partial = parts.get(1).unwrap_or(&"").to_string();
            if parts.len() == 2 && !text.ends_with(' ') {
                let partial_lower = partial.to_lowercase();
                let nick_match = if let Some(BufferKey::Channel(server_id, ref channel)) = state.active_buffer {
                    let server_id = server_id;
                    let channel = channel.clone();
                    state.get_server(server_id)
                        .and_then(|srv| srv.users.get(&channel))
                        .and_then(|users| {
                            users.iter()
                                .find(|u| u.nick.to_lowercase().starts_with(&partial_lower))
                                .map(|u| u.nick.clone())
                        })
                } else {
                    None
                };
                if let Some(nick) = nick_match {
                    state.input.text = format!("/{} {} ", cmd, nick);
                    state.input.cursor = state.input.text.len();
                }
            }
        }
        "join" | "j" => {
            let partial = parts.get(1).unwrap_or(&"").to_lowercase();
            if parts.len() == 2 && !text.ends_with(' ') {
                let mut channels: Vec<String> = Vec::new();
                for srv in &state.servers {
                    for ch in &srv.channels {
                        if ch.to_lowercase().starts_with(&partial) && !channels.contains(ch) {
                            channels.push(ch.clone());
                        }
                    }
                }
                for cfg in &state.config.servers {
                    for ch in &cfg.channels {
                        if ch.to_lowercase().starts_with(&partial) && !channels.contains(ch) {
                            channels.push(ch.clone());
                        }
                    }
                }
                if let Some(first) = channels.first() {
                    state.input.text = format!("/join {} ", first);
                    state.input.cursor = state.input.text.len();
                }
            }
        }
        _ => {}
    }
}

fn resolve_channel(state: &AppState, explicit: Option<String>) -> Option<String> {
    explicit.or_else(|| {
        match &state.active_buffer {
            Some(BufferKey::Channel(_, c)) => Some(c.clone()),
            _ => None,
        }
    })
}

fn handle_command(state: &mut AppState, text: &str) -> Vec<Action> {
    let server_id = state.active_server_id();
    match commands::parse_command(text) {
        Some(commands::ParsedCommand::ServerAdd { name, host, port, tls }) => {
            let nick = state.config.servers.first()
                .map(|s| s.nickname.clone())
                .unwrap_or_else(|| "crabchat_user".to_string());
            vec![Action::ConnectServer { name, host, port, tls, nick, accept_invalid_certs: false }]
        }
        Some(commands::ParsedCommand::ServerConnect { name }) => {
            if let Some(srv_cfg) = state.config.servers.iter().find(|s| s.name.eq_ignore_ascii_case(&name)) {
                vec![Action::ConnectServer {
                    name: srv_cfg.name.clone(),
                    host: srv_cfg.host.clone(),
                    port: srv_cfg.port,
                    tls: srv_cfg.tls,
                    nick: srv_cfg.nickname.clone(),
                    accept_invalid_certs: srv_cfg.accept_invalid_certs,
                }]
            } else {
                if let Some(ref key) = state.active_buffer.clone() {
                    let names: Vec<_> = state.config.servers.iter().map(|s| s.name.as_str()).collect();
                    state.error_message(key, format!("Unknown server '{}'. Available: {}", name, names.join(", ")));
                }
                vec![]
            }
        }
        Some(commands::ParsedCommand::ServerDisconnect) => {
            if let Some(sid) = server_id {
                vec![Action::DisconnectServer { server_id: sid }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::ServerList) => {
            if let Some(ref key) = state.active_buffer.clone() {
                if state.servers.is_empty() {
                    state.system_message(key, "No servers configured.".to_string());
                } else {
                    let lines: Vec<String> = state.servers.iter().map(|srv| {
                        let status = match srv.status {
                            ConnectionStatus::Connected => "connected",
                            ConnectionStatus::Connecting => "connecting",
                            ConnectionStatus::Disconnected => "disconnected",
                        };
                        format!("  {} ({}:{}) [{}]", srv.name, srv.host, srv.port, status)
                    }).collect();
                    for line in lines {
                        state.system_message(key, line);
                    }
                }
            }
            vec![]
        }
        Some(commands::ParsedCommand::Join { channel }) => {
            if let Some(sid) = server_id {
                vec![Action::JoinChannel { server_id: sid, channel }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Part { channel, reason }) => {
            if let Some(sid) = server_id {
                let ch = channel.unwrap_or_else(|| {
                    match &state.active_buffer {
                        Some(BufferKey::Channel(_, c)) => c.clone(),
                        _ => String::new(),
                    }
                });
                if ch.is_empty() {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                } else {
                    let reason = reason.or_else(|| Some(state.config.behavior.part_message.clone()));
                    vec![Action::PartChannel { server_id: sid, channel: ch, reason }]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Nick { nick }) => {
            if let Some(sid) = server_id {
                vec![Action::ChangeNick { server_id: sid, nick }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Msg { target, text }) => {
            if let Some(sid) = server_id {
                // Open/switch to query buffer
                let key = BufferKey::Query(sid, target.clone());
                state.ensure_buffer(key.clone());
                let nick = state.get_server(sid).map(|s| s.nickname.clone()).unwrap_or_default();
                let msg = Message {
                    timestamp: Local::now().format(&state.timestamp_format).to_string(),
                    sender: nick,
                    text: text.clone(),
                    kind: MessageKind::Normal,
                };
                state.add_message_to_buffer(&key, msg);
                state.set_active_buffer(key);
                vec![Action::SendPrivmsg { server_id: sid, target, text }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Me { text }) => {
            if let Some(sid) = server_id {
                if let Some(ref buf_key) = state.active_buffer.clone() {
                    let target = match buf_key {
                        BufferKey::Channel(_, c) => c.clone(),
                        BufferKey::Query(_, t) => t.clone(),
                        _ => {
                            state.status_message = Some("No active channel or query".to_string());
                            return vec![];
                        }
                    };
                    let nick = state.get_server(sid).map(|s| s.nickname.clone()).unwrap_or_default();
                    let msg = Message {
                        timestamp: Local::now().format(&state.timestamp_format).to_string(),
                        sender: nick,
                        text: text.clone(),
                        kind: MessageKind::Action,
                    };
                    state.add_message_to_buffer(buf_key, msg);
                    vec![Action::SendAction { server_id: sid, target, text }]
                } else {
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::DccList) => {
            if let Some(ref key) = state.active_buffer.clone() {
                if state.transfers.is_empty() {
                    state.system_message(key, "No DCC transfers.".to_string());
                } else {
                    let lines: Vec<String> = state.transfers.iter().map(|t| {
                        let pct = if t.size > 0 {
                            (t.received as f64 / t.size as f64 * 100.0) as u32
                        } else {
                            0
                        };
                        format!(
                            "  [{}] {} from {} — {} bytes ({}%) {:?}",
                            t.id, t.filename, t.from, t.size, pct, t.status
                        )
                    }).collect();
                    for line in lines {
                        state.system_message(key, line);
                    }
                }
            }
            vec![]
        }
        Some(commands::ParsedCommand::DccAccept { id }) => {
            vec![Action::DccAccept { transfer_id: id }]
        }
        Some(commands::ParsedCommand::DccCancel { id }) => {
            vec![Action::DccCancel { transfer_id: id }]
        }
        Some(commands::ParsedCommand::Quit { message }) => {
            vec![Action::Quit {
                message: message.or_else(|| Some(state.config.behavior.quit_message.clone())),
            }]
        }
        Some(commands::ParsedCommand::Kick { channel, user, reason }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendKick { server_id: sid, channel: ch, user, reason }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Ban { channel, mask }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendMode { server_id: sid, target: ch, modes: format!("+b {}", mask) }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Mode { target, modes }) => {
            if let Some(sid) = server_id {
                vec![Action::SendMode { server_id: sid, target, modes }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Op { channel, nick }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendMode { server_id: sid, target: ch, modes: format!("+o {}", nick) }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Deop { channel, nick }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendMode { server_id: sid, target: ch, modes: format!("-o {}", nick) }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Voice { channel, nick }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendMode { server_id: sid, target: ch, modes: format!("+v {}", nick) }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Devoice { channel, nick }) => {
            if let Some(sid) = server_id {
                if let Some(ch) = resolve_channel(state, channel) {
                    vec![Action::SendMode { server_id: sid, target: ch, modes: format!("-v {}", nick) }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Topic { text }) => {
            if let Some(sid) = server_id {
                if let Some(BufferKey::Channel(_, ref ch)) = state.active_buffer {
                    let ch = ch.clone();
                    vec![Action::SetTopic { server_id: sid, channel: ch, text }]
                } else {
                    state.status_message = Some("Not in a channel".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Notice { target, text }) => {
            if let Some(sid) = server_id {
                // Display our own notice locally
                if let Some(ref key) = state.active_buffer.clone() {
                    let nick = state.get_server(sid).map(|s| s.nickname.clone()).unwrap_or_default();
                    let msg = Message {
                        timestamp: Local::now().format(&state.timestamp_format).to_string(),
                        sender: nick,
                        text: format!("-{}> {}", target, text),
                        kind: MessageKind::Notice,
                    };
                    state.add_message_to_buffer(key, msg);
                }
                vec![Action::SendNotice { server_id: sid, target, text }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Whois { nick }) => {
            if let Some(sid) = server_id {
                vec![Action::SendWhois { server_id: sid, nick }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Who { target }) => {
            if let Some(sid) = server_id {
                vec![Action::SendWho { server_id: sid, target }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Away { message }) => {
            if let Some(sid) = server_id {
                if let Some(ref key) = state.active_buffer.clone() {
                    if message.is_some() {
                        state.system_message(key, format!("You are now marked as away: {}", message.as_deref().unwrap()));
                    } else {
                        state.system_message(key, "You are no longer marked as away.".to_string());
                    }
                }
                vec![Action::SetAway { server_id: sid, message }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Raw { command }) => {
            if let Some(sid) = server_id {
                vec![Action::SendRaw { server_id: sid, command }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::List) => {
            if let Some(sid) = server_id {
                if let Some(ref key) = state.active_buffer.clone() {
                    state.system_message(key, "Requesting channel list...".to_string());
                }
                vec![Action::SendList { server_id: sid }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Slap { nick }) => {
            if let Some(sid) = server_id {
                if let Some(ref buf_key) = state.active_buffer.clone() {
                    let target = match buf_key {
                        BufferKey::Channel(_, c) => c.clone(),
                        BufferKey::Query(_, t) => t.clone(),
                        _ => {
                            state.status_message = Some("No active channel or query".to_string());
                            return vec![];
                        }
                    };
                    let slap_text = format!("slaps {} around a bit with a large trout", nick);
                    let our_nick = state.get_server(sid).map(|s| s.nickname.clone()).unwrap_or_default();
                    let msg = Message {
                        timestamp: Local::now().format(&state.timestamp_format).to_string(),
                        sender: our_nick,
                        text: slap_text.clone(),
                        kind: MessageKind::Action,
                    };
                    state.add_message_to_buffer(buf_key, msg);
                    vec![Action::SendAction { server_id: sid, target, text: slap_text }]
                } else {
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Ignore { nick }) => {
            let lower = nick.to_lowercase();
            state.ignore_list.insert(lower);
            if let Some(ref key) = state.active_buffer.clone() {
                state.system_message(key, format!("Now ignoring: {}", nick));
            }
            vec![]
        }
        Some(commands::ParsedCommand::Unignore { nick }) => {
            let lower = nick.to_lowercase();
            state.ignore_list.remove(&lower);
            if let Some(ref key) = state.active_buffer.clone() {
                state.system_message(key, format!("No longer ignoring: {}", nick));
            }
            vec![]
        }
        Some(commands::ParsedCommand::IgnoreList) => {
            if let Some(ref key) = state.active_buffer.clone() {
                if state.ignore_list.is_empty() {
                    state.system_message(key, "Ignore list is empty.".to_string());
                } else {
                    let list: Vec<_> = state.ignore_list.iter().cloned().collect();
                    state.system_message(key, format!("Ignored: {}", list.join(", ")));
                }
            }
            vec![]
        }
        Some(commands::ParsedCommand::Notify { nick }) => {
            if let Some(ref key) = state.active_buffer.clone() {
                match nick {
                    Some(n) => {
                        let lower = n.to_lowercase();
                        if state.notify_list.contains(&lower) {
                            state.notify_list.remove(&lower);
                            state.system_message(key, format!("Removed {} from notify list.", n));
                        } else {
                            state.notify_list.insert(lower);
                            state.system_message(key, format!("Added {} to notify list.", n));
                        }
                    }
                    None => {
                        if state.notify_list.is_empty() {
                            state.system_message(key, "Notify list is empty.".to_string());
                        } else {
                            let list: Vec<_> = state.notify_list.iter().cloned().collect();
                            state.system_message(key, format!("Notify list: {}", list.join(", ")));
                        }
                    }
                }
            }
            vec![]
        }
        Some(commands::ParsedCommand::Ctcp { target, command }) => {
            if let Some(sid) = server_id {
                if let Some(ref key) = state.active_buffer.clone() {
                    state.system_message(key, format!("CTCP {} sent to {}", command, target));
                }
                vec![Action::SendCtcp { server_id: sid, target, command }]
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::ServerBrowser) => {
            state.server_browser.toggle();
            vec![]
        }
        Some(commands::ParsedCommand::ChannelBrowser) => {
            if let Some(sid) = server_id {
                if state.get_server(sid).map(|s| s.status == ConnectionStatus::Connected).unwrap_or(false) {
                    let needs_fetch = state.channel_browser.open(sid);
                    if needs_fetch {
                        vec![Action::SendList { server_id: sid }]
                    } else {
                        vec![]
                    }
                } else {
                    state.status_message = Some("Not connected to any server".to_string());
                    vec![]
                }
            } else {
                state.status_message = Some("No active server".to_string());
                vec![]
            }
        }
        Some(commands::ParsedCommand::Help) => {
            if let Some(ref key) = state.active_buffer.clone() {
                let help = vec![
                    "Available commands:",
                    "",
                    "  Connection:",
                    "  /server connect <name>          — Connect to a built-in server",
                    "  /server add <name> <host:port>  — Add and connect custom server",
                    "  /server list                    — List available servers",
                    "  /server disconnect              — Disconnect current server",
                    "  /servers                        — Open server browser (F2)",
                    "",
                    "  Channels:",
                    "  /join #channel                  — Join channel",
                    "  /part [#channel] [reason]       — Leave channel",
                    "  /topic <text>                   — Set channel topic",
                    "  /list                           — Request channel list",
                    "  /channels                       — Open channel browser (F3)",
                    "",
                    "  Communication:",
                    "  /msg <user> <text>              — Private message",
                    "  /me <action>                    — Action message",
                    "  /notice <target> <text>         — Send notice",
                    "  /ctcp <target> <command>        — Send CTCP request",
                    "  /slap <nick>                    — Slap someone with a trout",
                    "",
                    "  Channel ops:",
                    "  /kick [#chan] <nick> [reason]   — Kick user",
                    "  /ban [#chan] <mask>             — Ban user",
                    "  /mode <target> <modes>          — Set mode",
                    "  /op [#chan] <nick>              — Give operator status",
                    "  /deop [#chan] <nick>            — Remove operator status",
                    "  /voice [#chan] <nick>           — Give voice",
                    "  /devoice [#chan] <nick>         — Remove voice",
                    "",
                    "  Info:",
                    "  /whois <nick>                   — Query user info",
                    "  /who <target>                   — Query who",
                    "  /away [message]                 — Set/unset away",
                    "",
                    "  Social:",
                    "  /ignore <nick>                  — Ignore user",
                    "  /unignore <nick>                — Unignore user",
                    "  /ignorelist                     — Show ignore list",
                    "  /notify [nick]                  — Toggle/show notify list",
                    "",
                    "  Advanced:",
                    "  /nick <name>                    — Change nickname",
                    "  /raw <command>                  — Send raw IRC command",
                    "",
                    "  DCC:",
                    "  /dcc list                       — List transfers",
                    "  /dcc accept <id>                — Accept transfer",
                    "  /dcc cancel <id>                — Cancel transfer",
                    "",
                    "  /quit [message]                 — Exit",
                    "",
                    "Keybindings:",
                    "  Tab        — Cycle focus / nick completion",
                    "  PageUp/Dn  — Scroll messages",
                    "  Up/Down    — Command history",
                    "  F2         — Server browser",
                    "  F3         — Channel browser",
                    "  Ctrl+C     — Quit",
                ];
                for line in help {
                    state.system_message(key, line.to_string());
                }
            }
            vec![]
        }
        None => {
            if let Some(ref key) = state.active_buffer.clone() {
                state.error_message(key, format!("Unknown command: {}", text.split_whitespace().next().unwrap_or(text)));
            }
            vec![]
        }
    }
}

pub fn handle_irc_message(
    state: &mut AppState,
    server_id: ServerId,
    message: irc::client::prelude::Message,
) {
    use irc::client::prelude::{Command, Prefix};

    let nick_from = match &message.prefix {
        Some(Prefix::Nickname(nick, _, _)) => nick.clone(),
        Some(Prefix::ServerName(name)) => name.clone(),
        None => String::new(),
    };

    // Ignore list check: skip messages from ignored nicks
    if !nick_from.is_empty() && state.ignore_list.contains(&nick_from.to_lowercase()) {
        match &message.command {
            Command::PRIVMSG(_, _) | Command::NOTICE(_, _) | Command::KICK(_, _, _) => return,
            _ => {}
        }
    }

    match &message.command {
        Command::PRIVMSG(target, text) => {
            // Check for CTCP
            if text.starts_with('\x01') && text.ends_with('\x01') {
                let ctcp = &text[1..text.len() - 1];
                if ctcp.starts_with("ACTION ") {
                    let action_text = &ctcp[7..];
                    let key = if target.starts_with('#') || target.starts_with('&') {
                        BufferKey::Channel(server_id, target.clone())
                    } else {
                        BufferKey::Query(server_id, nick_from.clone())
                    };
                    let msg = Message {
                        timestamp: Local::now().format(&state.timestamp_format).to_string(),
                        sender: nick_from.clone(),
                        text: action_text.to_string(),
                        kind: MessageKind::Action,
                    };
                    state.add_message_to_buffer(&key, msg);

                    // Check for mention
                    if let Some(srv) = state.get_server(server_id) {
                        if action_text.to_lowercase().contains(&srv.nickname.to_lowercase()) {
                            if let Some(buf) = state.buffers.get_mut(&key) {
                                buf.has_mention = true;
                            }
                            if state.config.behavior.bell_on_mention {
                                state.pending_bell = true;
                            }
                        }
                    }
                } else if ctcp.starts_with("DCC ") {
                    // Handle DCC
                    crate::dcc::parser::handle_ctcp_dcc(state, server_id, &nick_from, ctcp);
                } else {
                    // CTCP request: VERSION, PING, TIME, FINGER
                    handle_ctcp_request(state, server_id, &nick_from, ctcp);
                }
                return;
            }

            let key = if target.starts_with('#') || target.starts_with('&') {
                BufferKey::Channel(server_id, target.clone())
            } else {
                // Private message — show in query buffer
                BufferKey::Query(server_id, nick_from.clone())
            };

            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from.clone(),
                text: text.clone(),
                kind: MessageKind::Normal,
            };
            state.add_message_to_buffer(&key, msg);

            // Check for mention
            if let Some(srv) = state.get_server(server_id) {
                if text.to_lowercase().contains(&srv.nickname.to_lowercase()) {
                    if let Some(buf) = state.buffers.get_mut(&key) {
                        buf.has_mention = true;
                    }
                    if state.config.behavior.bell_on_mention {
                        state.pending_bell = true;
                    }
                }
            }

            // Bell on PM
            if matches!(key, BufferKey::Query(_, _)) && state.config.behavior.bell_on_pm {
                state.pending_bell = true;
            }
        }

        Command::JOIN(channel, _, _) => {
            let key = BufferKey::Channel(server_id, channel.clone());
            state.ensure_buffer(key.clone());

            // If it's us joining, switch to the channel
            if let Some(srv) = state.get_server_mut(server_id) {
                if nick_from.eq_ignore_ascii_case(&srv.nickname) {
                    if !srv.channels.contains(channel) {
                        srv.channels.push(channel.clone());
                    }
                    state.set_active_buffer(key.clone());
                } else {
                    // Add user to channel user list
                    let users = srv.users.entry(channel.clone()).or_default();
                    if !users.iter().any(|u| u.nick == nick_from) {
                        users.push(ChannelUser {
                            nick: nick_from.clone(),
                            prefix: String::new(),
                        });
                    }
                }
            }

            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from.clone(),
                text: format!("has joined {}", channel),
                kind: MessageKind::Join,
            };
            state.add_message_to_buffer(&key, msg);

            // Notify list check
            if state.notify_list.contains(&nick_from.to_lowercase()) {
                let status_key = BufferKey::ServerStatus(server_id);
                state.system_message(&status_key, format!("Notify: {} is now online (joined {})", nick_from, channel));
                state.known_online.insert(nick_from.to_lowercase());
            }
        }

        Command::PART(channel, reason) => {
            let key = BufferKey::Channel(server_id, channel.clone());

            if let Some(srv) = state.get_server_mut(server_id) {
                if nick_from.eq_ignore_ascii_case(&srv.nickname) {
                    srv.channels.retain(|c| c != channel);
                    // Switch away from parted channel
                    if state.active_buffer.as_ref() == Some(&key) {
                        let fallback = BufferKey::ServerStatus(server_id);
                        state.set_active_buffer(fallback);
                    }
                } else {
                    if let Some(users) = srv.users.get_mut(channel) {
                        users.retain(|u| !u.nick.eq_ignore_ascii_case(&nick_from));
                    }
                }
            }

            let reason_text = reason.as_deref().unwrap_or("");
            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from,
                text: format!("has left {} {}", channel, reason_text),
                kind: MessageKind::Part,
            };
            state.add_message_to_buffer(&key, msg);
        }

        Command::QUIT(reason) => {
            let reason_text = reason.as_deref().unwrap_or("");

            // Notify list check
            if state.notify_list.contains(&nick_from.to_lowercase()) {
                let status_key = BufferKey::ServerStatus(server_id);
                state.system_message(&status_key, format!("Notify: {} is now offline (quit)", nick_from));
                state.known_online.remove(&nick_from.to_lowercase());
            }

            // Remove from all channel user lists on this server
            if let Some(srv) = state.get_server_mut(server_id) {
                for (_ch, users) in srv.users.iter_mut() {
                    users.retain(|u| !u.nick.eq_ignore_ascii_case(&nick_from));
                }
                // Post quit message in all channels user was in
                for ch in srv.channels.clone() {
                    let key = BufferKey::Channel(server_id, ch);
                    let msg = Message {
                        timestamp: Local::now().format(&state.timestamp_format).to_string(),
                        sender: nick_from.clone(),
                        text: format!("has quit ({})", reason_text),
                        kind: MessageKind::Quit,
                    };
                    state.add_message_to_buffer(&key, msg);
                }
            }
        }

        Command::NICK(new_nick) => {
            if let Some(srv) = state.get_server_mut(server_id) {
                if nick_from.eq_ignore_ascii_case(&srv.nickname) {
                    srv.nickname = new_nick.clone();
                }
                for (_ch, users) in srv.users.iter_mut() {
                    for u in users.iter_mut() {
                        if u.nick.eq_ignore_ascii_case(&nick_from) {
                            u.nick = new_nick.clone();
                        }
                    }
                }
            }
            // Post nick change in active buffer
            if let Some(ref key) = state.active_buffer.clone() {
                state.system_message(
                    key,
                    format!("{} is now known as {}", nick_from, new_nick),
                );
            }
        }

        Command::NOTICE(target, text) => {
            // Check for CTCP reply (wrapped in \x01)
            if text.starts_with('\x01') && text.ends_with('\x01') {
                let ctcp = &text[1..text.len() - 1];
                let key = BufferKey::ServerStatus(server_id);
                state.system_message(&key, format!("CTCP reply from {}: {}", nick_from, ctcp));
                return;
            }

            let key = if target.starts_with('#') || target.starts_with('&') {
                BufferKey::Channel(server_id, target.clone())
            } else {
                BufferKey::ServerStatus(server_id)
            };
            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from,
                text: text.clone(),
                kind: MessageKind::Notice,
            };
            state.add_message_to_buffer(&key, msg);
        }

        Command::TOPIC(channel, topic) => {
            if let Some(topic) = topic {
                if let Some(srv) = state.get_server_mut(server_id) {
                    srv.topics.insert(channel.clone(), topic.clone());
                }
                let key = BufferKey::Channel(server_id, channel.clone());
                state.system_message(
                    &key,
                    format!("{} set topic: {}", nick_from, topic),
                );
            }
        }

        Command::Response(ref resp, ref args) => {
            handle_numeric(state, server_id, *resp, args);
        }

        Command::KICK(channel, user, reason) => {
            let key = BufferKey::Channel(server_id, channel.clone());
            let reason_text = reason.as_deref().unwrap_or("");
            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from,
                text: format!("kicked {} ({})", user, reason_text),
                kind: MessageKind::System,
            };
            state.add_message_to_buffer(&key, msg);

            if let Some(srv) = state.get_server_mut(server_id) {
                if user.eq_ignore_ascii_case(&srv.nickname) {
                    srv.channels.retain(|c| c != channel);
                    if state.active_buffer.as_ref() == Some(&key) {
                        state.set_active_buffer(BufferKey::ServerStatus(server_id));
                    }
                    // Auto-rejoin on kick
                    if state.config.behavior.auto_rejoin_on_kick {
                        let delay = Duration::from_secs(state.config.behavior.rejoin_delay_secs);
                        state.pending_rejoins.push(PendingRejoin {
                            server_id,
                            channel: channel.clone(),
                            rejoin_at: Instant::now() + delay,
                        });
                    }
                } else {
                    if let Some(users) = srv.users.get_mut(channel) {
                        users.retain(|u| !u.nick.eq_ignore_ascii_case(user));
                    }
                }
            }
        }

        Command::ChannelMODE(ref target, ref modes) => {
            // Format mode changes for display
            let mode_text: String = modes.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(" ");

            let key = if target.starts_with('#') || target.starts_with('&') {
                BufferKey::Channel(server_id, target.clone())
            } else {
                BufferKey::ServerStatus(server_id)
            };
            state.system_message(&key, format!("{} sets mode {} on {}", nick_from, mode_text, target));

            // Update user prefixes from typed modes
            if target.starts_with('#') || target.starts_with('&') {
                update_channel_modes(state, server_id, target, modes);
            }
        }

        Command::UserMODE(ref target, ref modes) => {
            let mode_text: String = modes.iter().map(|m| format!("{}", m)).collect::<Vec<_>>().join(" ");
            let key = BufferKey::ServerStatus(server_id);
            state.system_message(&key, format!("{} sets mode {} on {}", nick_from, mode_text, target));
        }

        _ => {
            // Log unhandled messages to server status
            let key = BufferKey::ServerStatus(server_id);
            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: String::new(),
                text: format!("{}", message),
                kind: MessageKind::System,
            };
            state.add_message_to_buffer(&key, msg);
        }
    }
}

fn handle_ctcp_request(state: &mut AppState, server_id: ServerId, from: &str, ctcp: &str) {
    let parts: Vec<&str> = ctcp.splitn(2, ' ').collect();
    let command = parts[0].to_uppercase();
    let arg = parts.get(1).unwrap_or(&"");

    let key = BufferKey::ServerStatus(server_id);
    state.system_message(&key, format!("CTCP {} from {}", command, from));

    match command.as_str() {
        "VERSION" if state.config.ctcp.reply_version => {
            let response = format!("VERSION {}", state.config.ctcp.version_string);
            state.pending_actions.push(Action::SendCtcpReply {
                server_id,
                target: from.to_string(),
                response,
            });
        }
        "PING" if state.config.ctcp.reply_ping => {
            let response = format!("PING {}", arg);
            state.pending_actions.push(Action::SendCtcpReply {
                server_id,
                target: from.to_string(),
                response,
            });
        }
        "TIME" if state.config.ctcp.reply_time => {
            let response = format!("TIME {}", Local::now().format("%a %b %d %H:%M:%S %Y"));
            state.pending_actions.push(Action::SendCtcpReply {
                server_id,
                target: from.to_string(),
                response,
            });
        }
        "FINGER" if state.config.ctcp.reply_finger => {
            let response = format!("FINGER {}", state.config.ctcp.finger_string);
            state.pending_actions.push(Action::SendCtcpReply {
                server_id,
                target: from.to_string(),
                response,
            });
        }
        _ => {}
    }
}

fn update_channel_modes(
    state: &mut AppState,
    server_id: ServerId,
    channel: &str,
    modes: &[irc::client::prelude::Mode<irc::proto::ChannelMode>],
) {
    use irc::proto::ChannelMode;

    let srv = match state.get_server_mut(server_id) {
        Some(s) => s,
        None => return,
    };
    let users = match srv.users.get_mut(channel) {
        Some(u) => u,
        None => return,
    };

    for mode in modes {
        let (adding, mode_type, arg) = match mode {
            irc::client::prelude::Mode::Plus(m, a) => (true, m, a),
            irc::client::prelude::Mode::Minus(m, a) => (false, m, a),
            irc::client::prelude::Mode::NoPrefix(_) => continue,
        };

        let prefix = match mode_type {
            ChannelMode::Oper => Some("@"),
            ChannelMode::Voice => Some("+"),
            ChannelMode::Halfop => Some("%"),
            ChannelMode::Founder => Some("~"),
            ChannelMode::Admin => Some("&"),
            _ => None,
        };

        if let (Some(prefix_str), Some(nick)) = (prefix, arg) {
            if let Some(user) = users.iter_mut().find(|u| u.nick.eq_ignore_ascii_case(nick)) {
                if adding {
                    user.prefix = prefix_str.to_string();
                } else if user.prefix == prefix_str {
                    user.prefix = String::new();
                }
            }
        }
    }
}

fn handle_numeric(state: &mut AppState, server_id: ServerId, resp: irc::client::prelude::Response, args: &[String]) {
    use irc::client::prelude::Response;
    let key = BufferKey::ServerStatus(server_id);

    match resp {
        // RPL_WELCOME
        Response::RPL_WELCOME => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
            // Auto-identify with NickServ
            let nick_password = state.config.servers.iter()
                .find(|s| {
                    state.get_server(server_id)
                        .map(|srv| s.name.eq_ignore_ascii_case(&srv.name))
                        .unwrap_or(false)
                })
                .and_then(|s| s.nick_password.clone());
            if let Some(pass) = nick_password {
                state.pending_actions.push(Action::SendPrivmsg {
                    server_id,
                    target: "NickServ".to_string(),
                    text: format!("IDENTIFY {}", pass),
                });
                state.system_message(&key, "Auto-identifying with NickServ...".to_string());
            }
        }
        // RPL_TOPIC
        Response::RPL_TOPIC => {
            if args.len() >= 3 {
                let channel = &args[1];
                let topic = &args[2];
                if let Some(srv) = state.get_server_mut(server_id) {
                    srv.topics.insert(channel.clone(), topic.clone());
                }
            }
        }
        // RPL_NAMREPLY
        Response::RPL_NAMREPLY => {
            if args.len() >= 4 {
                let channel = &args[2];
                let names = &args[3];
                if let Some(srv) = state.get_server_mut(server_id) {
                    let users = srv.users.entry(channel.clone()).or_default();
                    for name in names.split_whitespace() {
                        let (prefix, nick) = if name.starts_with('@')
                            || name.starts_with('+')
                            || name.starts_with('%')
                            || name.starts_with('~')
                            || name.starts_with('&')
                        {
                            (name[..1].to_string(), name[1..].to_string())
                        } else {
                            (String::new(), name.to_string())
                        };
                        if !users.iter().any(|u| u.nick.eq_ignore_ascii_case(&nick)) {
                            users.push(ChannelUser { nick, prefix });
                        }
                    }
                }
            }
        }
        // RPL_ENDOFNAMES
        Response::RPL_ENDOFNAMES => {}

        // RPL_WHOISUSER (311)
        Response::RPL_WHOISUSER => {
            if args.len() >= 6 {
                state.system_message(&key, format!(
                    "WHOIS {} ({}@{}) — {}",
                    args[1], args[2], args[3], args[5]
                ));
            }
        }
        // RPL_WHOISCHANNELS (319)
        Response::RPL_WHOISCHANNELS => {
            if args.len() >= 3 {
                state.system_message(&key, format!("  Channels: {}", args[2]));
            }
        }
        // RPL_WHOISSERVER (312)
        Response::RPL_WHOISSERVER => {
            if args.len() >= 4 {
                state.system_message(&key, format!("  Server: {} ({})", args[2], args[3]));
            }
        }
        // RPL_ENDOFWHOIS (318)
        Response::RPL_ENDOFWHOIS => {
            state.system_message(&key, "End of WHOIS.".to_string());
        }

        // RPL_WHOREPLY (352)
        Response::RPL_WHOREPLY => {
            if args.len() >= 8 {
                state.system_message(&key, format!(
                    "  {} {} {}@{} ({}) — {}",
                    args[5], args[1], args[2], args[3], args[6], args[7]
                ));
            } else if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
        // RPL_ENDOFWHO (315)
        Response::RPL_ENDOFWHO => {
            state.system_message(&key, "End of WHO.".to_string());
        }

        // RPL_NOWAWAY (306)
        Response::RPL_NOWAWAY => {
            if let Some(srv) = state.get_server_mut(server_id) {
                srv.is_away = true;
            }
            state.system_message(&key, "You are now marked as away.".to_string());
        }
        // RPL_UNAWAY (305)
        Response::RPL_UNAWAY => {
            if let Some(srv) = state.get_server_mut(server_id) {
                srv.is_away = false;
            }
            state.system_message(&key, "You are no longer marked as away.".to_string());
        }
        // RPL_AWAY (301) — target user is away
        Response::RPL_AWAY => {
            if args.len() >= 3 {
                state.system_message(&key, format!("{} is away: {}", args[1], args[2]));
            }
        }

        // RPL_LISTSTART (321)
        Response::RPL_LISTSTART => {
            // Only show in buffer if channel browser is NOT handling it
            if !(state.channel_browser.loading && state.channel_browser.server_id == Some(server_id)) {
                state.system_message(&key, "Channel list:".to_string());
            }
        }
        // RPL_LIST (322)
        Response::RPL_LIST => {
            if args.len() >= 4 {
                let ch_name = args[1].clone();
                let user_count: usize = args[2].parse().unwrap_or(0);
                let topic = if args.len() > 3 { args[3].clone() } else { String::new() };

                // Feed channel browser if it's loading for this server
                if state.channel_browser.loading && state.channel_browser.server_id == Some(server_id) {
                    state.channel_browser.add_channel(ch_name, user_count, topic);
                    // Don't mark dirty on every single entry — only periodically
                    if state.channel_browser.channels.len() % 200 == 0 {
                        state.dirty = true;
                    }
                } else {
                    // Only dump into buffer when not using the browser
                    state.system_message(&key, format!("  {} ({} users) — {}", ch_name, user_count, topic));
                }
            }
        }
        // RPL_LISTEND (323)
        Response::RPL_LISTEND => {
            if state.channel_browser.loading && state.channel_browser.server_id == Some(server_id) {
                state.channel_browser.finish_loading();
            } else {
                state.system_message(&key, "End of channel list.".to_string());
            }
        }

        // RPL_MOTDSTART (375)
        Response::RPL_MOTDSTART => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
        // RPL_MOTD (372)
        Response::RPL_MOTD => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
        // RPL_ENDOFMOTD (376)
        Response::RPL_ENDOFMOTD => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }

        // ERR_NICKNAMEINUSE (433)
        Response::ERR_NICKNAMEINUSE => {
            state.system_message(&key, "Nickname already in use, trying alternative...".to_string());
            // Try alt nicks
            let alt_nick = {
                let srv_cfg = state.config.servers.iter().find(|s| {
                    state.get_server(server_id)
                        .map(|srv| s.name.eq_ignore_ascii_case(&srv.name))
                        .unwrap_or(false)
                });
                let alt_idx = state.get_server(server_id).map(|s| s.alt_nick_index).unwrap_or(0);
                if let Some(cfg) = srv_cfg {
                    if alt_idx < cfg.alt_nicks.len() {
                        Some(cfg.alt_nicks[alt_idx].clone())
                    } else {
                        // Fallback: append _ to current nick
                        state.get_server(server_id).map(|s| format!("{}_", s.nickname))
                    }
                } else {
                    state.get_server(server_id).map(|s| format!("{}_", s.nickname))
                }
            };
            if let Some(new_nick) = alt_nick {
                if let Some(srv) = state.get_server_mut(server_id) {
                    srv.alt_nick_index += 1;
                }
                state.pending_actions.push(Action::ChangeNick {
                    server_id,
                    nick: new_nick,
                });
            }
        }

        // RPL_ISON (303)
        Response::RPL_ISON => {
            if let Some(nicks_str) = args.last() {
                let online: std::collections::HashSet<String> = nicks_str
                    .split_whitespace()
                    .map(|n| n.to_lowercase())
                    .collect();

                // Check for newly online
                for nick in &online {
                    if !state.known_online.contains(nick) && state.notify_list.contains(nick) {
                        state.system_message(&key, format!("Notify: {} is now online", nick));
                    }
                }
                // Check for newly offline
                let old_online = state.known_online.clone();
                for nick in &old_online {
                    if !online.contains(nick) && state.notify_list.contains(nick) {
                        state.system_message(&key, format!("Notify: {} is now offline", nick));
                    }
                }
                state.known_online = online;
            }
        }

        // Other numerics — show in server status
        _ => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
    }
}
