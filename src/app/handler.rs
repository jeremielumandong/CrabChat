use crate::app::action::Action;
use crate::app::event::{AppEvent, ServerId};
use crate::app::state::*;
use crate::irc::commands;
use chrono::Local;
use crossterm::event::{Event as CEvent, KeyCode, KeyEvent, KeyModifiers};

pub fn handle_event(state: &mut AppState, event: AppEvent) -> Vec<Action> {
    state.dirty = true;
    match event {
        AppEvent::Terminal(cevent) => handle_terminal(state, cevent),
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
            }
            // Don't mark dirty on every progress tick to avoid excessive redraws
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
            // Just mark dirty for periodic refresh
            vec![]
        }
    }
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
        return vec![Action::Quit];
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
                        // Add our own message to the buffer
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
            } else {
                // Nick completion
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

fn handle_command(state: &mut AppState, text: &str) -> Vec<Action> {
    let server_id = state.active_server_id();
    match commands::parse_command(text) {
        Some(commands::ParsedCommand::ServerAdd { name, host, port, tls }) => {
            let nick = state.config.servers.first()
                .map(|s| s.nickname.clone())
                .unwrap_or_else(|| "crabchat_user".to_string());
            vec![Action::ConnectServer { name, host, port, tls, nick }]
        }
        Some(commands::ParsedCommand::ServerConnect { name }) => {
            if let Some(srv_cfg) = state.config.servers.iter().find(|s| s.name.eq_ignore_ascii_case(&name)) {
                vec![Action::ConnectServer {
                    name: srv_cfg.name.clone(),
                    host: srv_cfg.host.clone(),
                    port: srv_cfg.port,
                    tls: srv_cfg.tls,
                    nick: srv_cfg.nickname.clone(),
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
        Some(commands::ParsedCommand::Part { channel }) => {
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
                    vec![Action::PartChannel { server_id: sid, channel: ch, reason: None }]
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
        Some(commands::ParsedCommand::Quit) => {
            vec![Action::Quit]
        }
        Some(commands::ParsedCommand::Help) => {
            if let Some(ref key) = state.active_buffer.clone() {
                let help = vec![
                    "Available commands:",
                    "  /server connect <name>          — Connect to a built-in server",
                    "  /server add <name> <host:port>  — Add and connect custom server",
                    "  /server list                    — List available servers",
                    "  /server disconnect              — Disconnect current server",
                    "  /join #channel                  — Join channel",
                    "  /part [#channel]                — Leave channel",
                    "  /nick <name>                    — Change nickname",
                    "  /msg <user> <text>              — Private message",
                    "  /me <action>                    — Action message",
                    "  /dcc list                       — List transfers",
                    "  /dcc accept <id>                — Accept transfer",
                    "  /dcc cancel <id>                — Cancel transfer",
                    "  /quit                           — Exit",
                    "",
                    "Keybindings:",
                    "  Tab        — Cycle focus / nick completion",
                    "  PageUp/Dn  — Scroll messages",
                    "  Up/Down    — Command history",
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
                        // Action in query
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
                        }
                    }
                } else if ctcp.starts_with("DCC ") {
                    // Handle DCC - parsed elsewhere via dcc module
                    crate::dcc::parser::handle_ctcp_dcc(state, server_id, &nick_from, ctcp);
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
                }
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
                sender: nick_from,
                text: format!("has joined {}", channel),
                kind: MessageKind::Join,
            };
            state.add_message_to_buffer(&key, msg);
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
            let key = if target.starts_with('#') || target.starts_with('&') {
                BufferKey::Channel(server_id, target.clone())
            } else {
                BufferKey::ServerStatus(server_id)
            };
            let msg = Message {
                timestamp: Local::now().format(&state.timestamp_format).to_string(),
                sender: nick_from,
                text: text.clone(),
                kind: MessageKind::System,
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

        // RPL_TOPIC (332) - topic on join
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
                } else {
                    if let Some(users) = srv.users.get_mut(channel) {
                        users.retain(|u| !u.nick.eq_ignore_ascii_case(user));
                    }
                }
            }
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

fn handle_numeric(state: &mut AppState, server_id: ServerId, resp: irc::client::prelude::Response, args: &[String]) {
    use irc::client::prelude::Response;
    let key = BufferKey::ServerStatus(server_id);

    match resp {
        // RPL_WELCOME
        Response::RPL_WELCOME => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
        // RPL_TOPIC
        Response::RPL_TOPIC => {
            // args: [nick, #channel, topic]
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
            // args: [nick, =/*/@, #channel, "nick1 @nick2 +nick3"]
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
        Response::RPL_ENDOFNAMES => {
            // Nothing specific needed
        }
        // Other numerics — show in server status
        _ => {
            if let Some(text) = args.last() {
                state.system_message(&key, text.clone());
            }
        }
    }
}
