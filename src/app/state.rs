use crate::app::event::{ServerId, TransferId};
use crate::config::AppConfig;
use chrono::Local;
use std::collections::{BTreeMap, HashMap};
use std::net::IpAddr;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BufferKey {
    ServerStatus(ServerId),
    Channel(ServerId, String),
    Query(ServerId, String),
}

#[derive(Debug, Clone)]
pub struct Message {
    pub timestamp: String,
    pub sender: String,
    pub text: String,
    pub kind: MessageKind,
}

#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    Normal,
    Action,
    System,
    Error,
    Join,
    Part,
    Quit,
}

#[derive(Debug)]
pub struct Buffer {
    pub messages: Vec<Message>,
    pub scroll_offset: usize,
    pub unread_count: usize,
    pub has_mention: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            messages: Vec::new(),
            scroll_offset: 0,
            unread_count: 0,
            has_mention: false,
        }
    }

    pub fn add_message(&mut self, msg: Message, max_scrollback: usize) {
        self.messages.push(msg);
        if self.messages.len() > max_scrollback {
            self.messages.remove(0);
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

#[derive(Debug)]
pub struct ServerState {
    pub id: ServerId,
    pub name: String,
    pub host: String,
    pub port: u16,
    pub tls: bool,
    pub nickname: String,
    pub status: ConnectionStatus,
    pub channels: Vec<String>,
    pub users: HashMap<String, Vec<ChannelUser>>,
    pub topics: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct ChannelUser {
    pub nick: String,
    pub prefix: String, // "@", "+", etc.
}

impl ChannelUser {
    pub fn display_name(&self) -> String {
        format!("{}{}", self.prefix, self.nick)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DccTransferStatus {
    Pending,
    Active,
    Completed,
    Failed(String),
    Cancelled,
}

#[derive(Debug, Clone)]
pub struct DccTransfer {
    pub id: TransferId,
    pub server_id: ServerId,
    pub from: String,
    pub filename: String,
    pub size: u64,
    pub received: u64,
    pub ip: IpAddr,
    pub port: u16,
    pub status: DccTransferStatus,
}

#[derive(Debug)]
pub struct InputState {
    pub text: String,
    pub cursor: usize,
    pub history: Vec<String>,
    pub history_index: Option<usize>,
}

impl InputState {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
            history: Vec::new(),
            history_index: None,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            let prev = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
            self.text.drain(prev..self.cursor);
            self.cursor = prev;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            let next = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
            self.text.drain(self.cursor..next);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            self.cursor = self.text[..self.cursor]
                .char_indices()
                .next_back()
                .map(|(i, _)| i)
                .unwrap_or(0);
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            self.cursor = self.text[self.cursor..]
                .char_indices()
                .nth(1)
                .map(|(i, _)| self.cursor + i)
                .unwrap_or(self.text.len());
        }
    }

    pub fn move_home(&mut self) {
        self.cursor = 0;
    }

    pub fn move_end(&mut self) {
        self.cursor = self.text.len();
    }

    pub fn take_text(&mut self) -> String {
        let text = self.text.clone();
        self.text.clear();
        self.cursor = 0;
        self.history_index = None;
        if !text.is_empty() {
            self.history.push(text.clone());
        }
        text
    }

    pub fn history_up(&mut self) {
        if self.history.is_empty() {
            return;
        }
        let idx = match self.history_index {
            Some(i) if i > 0 => i - 1,
            Some(_) => return,
            None => self.history.len() - 1,
        };
        self.history_index = Some(idx);
        self.text = self.history[idx].clone();
        self.cursor = self.text.len();
    }

    pub fn history_down(&mut self) {
        match self.history_index {
            Some(i) if i + 1 < self.history.len() => {
                let idx = i + 1;
                self.history_index = Some(idx);
                self.text = self.history[idx].clone();
                self.cursor = self.text.len();
            }
            Some(_) => {
                self.history_index = None;
                self.text.clear();
                self.cursor = 0;
            }
            None => {}
        }
    }

    pub fn delete_word_back(&mut self) {
        if self.cursor == 0 {
            return;
        }
        let mut pos = self.cursor;
        // Skip trailing whitespace
        while pos > 0 && self.text.as_bytes().get(pos - 1) == Some(&b' ') {
            pos -= 1;
        }
        // Skip word characters
        while pos > 0 && self.text.as_bytes().get(pos - 1) != Some(&b' ') {
            pos -= 1;
        }
        self.text.drain(pos..self.cursor);
        self.cursor = pos;
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPanel {
    ServerTree,
    MessageArea,
    Input,
    UserList,
}

pub struct AppState {
    pub config: AppConfig,
    pub servers: Vec<ServerState>,
    pub buffers: BTreeMap<BufferKey, Buffer>,
    pub active_buffer: Option<BufferKey>,
    pub input: InputState,
    pub focus: FocusPanel,
    pub transfers: Vec<DccTransfer>,
    pub next_server_id: ServerId,
    pub next_transfer_id: TransferId,
    pub should_quit: bool,
    pub dirty: bool,
    pub status_message: Option<String>,
    pub timestamp_format: String,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let timestamp_format = config.ui.timestamp_format.clone();
        Self {
            config,
            servers: Vec::new(),
            buffers: BTreeMap::new(),
            active_buffer: None,
            input: InputState::new(),
            focus: FocusPanel::Input,
            transfers: Vec::new(),
            next_server_id: 0,
            next_transfer_id: 0,
            should_quit: false,
            dirty: true,
            status_message: None,
            timestamp_format,
        }
    }

    pub fn allocate_server_id(&mut self) -> ServerId {
        let id = self.next_server_id;
        self.next_server_id += 1;
        id
    }

    pub fn allocate_transfer_id(&mut self) -> TransferId {
        let id = self.next_transfer_id;
        self.next_transfer_id += 1;
        id
    }

    pub fn add_server(&mut self, server: ServerState) {
        let key = BufferKey::ServerStatus(server.id);
        self.buffers.entry(key.clone()).or_insert_with(Buffer::new);
        if self.active_buffer.is_none() {
            self.active_buffer = Some(key);
        }
        self.servers.push(server);
        self.dirty = true;
    }

    pub fn get_server(&self, id: ServerId) -> Option<&ServerState> {
        self.servers.iter().find(|s| s.id == id)
    }

    pub fn get_server_mut(&mut self, id: ServerId) -> Option<&mut ServerState> {
        self.servers.iter_mut().find(|s| s.id == id)
    }

    pub fn ensure_buffer(&mut self, key: BufferKey) -> &mut Buffer {
        self.buffers.entry(key).or_insert_with(Buffer::new)
    }

    pub fn add_message_to_buffer(&mut self, key: &BufferKey, msg: Message) {
        let max = self.config.ui.max_scrollback;
        let is_active = self.active_buffer.as_ref() == Some(key);
        let buf = self.buffers.entry(key.clone()).or_insert_with(Buffer::new);
        buf.add_message(msg, max);
        if !is_active {
            buf.unread_count += 1;
        }
        self.dirty = true;
    }

    pub fn system_message(&mut self, key: &BufferKey, text: String) {
        let msg = Message {
            timestamp: Local::now().format(&self.timestamp_format).to_string(),
            sender: "***".to_string(),
            text,
            kind: MessageKind::System,
        };
        self.add_message_to_buffer(key, msg);
    }

    pub fn error_message(&mut self, key: &BufferKey, text: String) {
        let msg = Message {
            timestamp: Local::now().format(&self.timestamp_format).to_string(),
            sender: "!!!".to_string(),
            text,
            kind: MessageKind::Error,
        };
        self.add_message_to_buffer(key, msg);
    }

    pub fn set_active_buffer(&mut self, key: BufferKey) {
        if let Some(buf) = self.buffers.get_mut(&key) {
            buf.unread_count = 0;
            buf.has_mention = false;
        }
        self.active_buffer = Some(key);
        self.dirty = true;
    }

    pub fn active_server_id(&self) -> Option<ServerId> {
        self.active_buffer.as_ref().map(|k| match k {
            BufferKey::ServerStatus(id) => *id,
            BufferKey::Channel(id, _) => *id,
            BufferKey::Query(id, _) => *id,
        })
    }

    pub fn cycle_focus(&mut self) {
        self.focus = match self.focus {
            FocusPanel::Input => FocusPanel::ServerTree,
            FocusPanel::ServerTree => FocusPanel::MessageArea,
            FocusPanel::MessageArea => FocusPanel::UserList,
            FocusPanel::UserList => FocusPanel::Input,
        };
        self.dirty = true;
    }

    pub fn status_line(&self) -> String {
        if let Some(ref msg) = self.status_message {
            return msg.clone();
        }
        let connected = self.servers.iter().filter(|s| s.status == ConnectionStatus::Connected).count();
        let total = self.servers.len();
        let active = self
            .transfers
            .iter()
            .filter(|t| t.status == DccTransferStatus::Active)
            .count();
        let mut s = format!("Servers: {}/{}", connected, total);
        if active > 0 {
            s.push_str(&format!(" | Transfers: {}", active));
        }
        s
    }

    pub fn select_next_buffer(&mut self) {
        let keys: Vec<_> = self.buffers.keys().cloned().collect();
        if keys.is_empty() {
            return;
        }
        let current_idx = self
            .active_buffer
            .as_ref()
            .and_then(|k| keys.iter().position(|x| x == k))
            .unwrap_or(0);
        let next = (current_idx + 1) % keys.len();
        self.set_active_buffer(keys[next].clone());
    }

    pub fn select_prev_buffer(&mut self) {
        let keys: Vec<_> = self.buffers.keys().cloned().collect();
        if keys.is_empty() {
            return;
        }
        let current_idx = self
            .active_buffer
            .as_ref()
            .and_then(|k| keys.iter().position(|x| x == k))
            .unwrap_or(0);
        let prev = if current_idx == 0 {
            keys.len() - 1
        } else {
            current_idx - 1
        };
        self.set_active_buffer(keys[prev].clone());
    }
}
