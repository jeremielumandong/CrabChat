//! Application state management.
//!
//! [`AppState`] is the single source of truth for the entire application. It
//! holds all server connections, message buffers, UI focus state, DCC transfers,
//! ignore/notify lists, and modal browser state. The event handler mutates
//! `AppState` in response to incoming events and the UI reads it to render.

use crate::app::action::Action;
use crate::app::event::{ServerId, TransferId};
use crate::config::AppConfig;
use chrono::Local;
use std::collections::{BTreeMap, HashMap, HashSet, VecDeque};
use std::net::IpAddr;
use std::time::Instant;

/// Identifies a message buffer — either a server status window, a channel, or
/// a private query (DM) conversation.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum BufferKey {
    /// Global highlights buffer — mentions, PMs, and notices. Sorts first.
    Highlights,
    /// The server status/console window.
    ServerStatus(ServerId),
    /// A channel conversation (e.g. `#rust`).
    Channel(ServerId, String),
    /// A private message conversation with another user.
    Query(ServerId, String),
}

/// A single chat message displayed in a buffer.
#[derive(Debug, Clone)]
pub struct Message {
    /// Formatted timestamp string (e.g. `"14:32"`).
    pub timestamp: String,
    /// The nick of the sender, or `"***"` / `"!!!"` for system/error messages.
    pub sender: String,
    /// The message body text.
    pub text: String,
    /// The type of message, which determines rendering style.
    pub kind: MessageKind,
}

/// The semantic kind of a chat message, controlling how it is styled in the UI.
#[derive(Debug, Clone, PartialEq)]
pub enum MessageKind {
    /// A regular chat message (PRIVMSG).
    Normal,
    /// A `/me` action (CTCP ACTION).
    Action,
    /// An informational system notice.
    System,
    /// An error notification.
    Error,
    /// A user join notification.
    Join,
    /// A user part (leave) notification.
    Part,
    /// A user quit notification.
    Quit,
    /// An IRC NOTICE message.
    Notice,
}

/// A scrollable message buffer for a server, channel, or query window.
///
/// Each buffer maintains its own scroll position and unread/mention indicators
/// that the server tree panel uses to show activity badges.
#[derive(Debug)]
pub struct Buffer {
    /// All messages in this buffer, bounded by `max_scrollback`.
    pub messages: VecDeque<Message>,
    /// Lines scrolled up from the bottom (0 = pinned to newest).
    pub scroll_offset: usize,
    /// Number of messages received while this buffer was not active.
    pub unread_count: usize,
    /// Whether the user's nick was mentioned while this buffer was inactive.
    pub has_mention: bool,
    /// When true, new messages don't auto-scroll — the view stays in place.
    pub paused: bool,
}

impl Buffer {
    pub fn new() -> Self {
        Self {
            messages: VecDeque::new(),
            scroll_offset: 0,
            unread_count: 0,
            has_mention: false,
            paused: false,
        }
    }

    /// Append a message, evicting the oldest if the scrollback limit is reached.
    /// When paused, increments scroll_offset to keep the view position stable.
    pub fn add_message(&mut self, msg: Message, max_scrollback: usize) {
        if self.paused && self.scroll_offset == 0 {
            // First message while paused at bottom — start offsetting
            self.scroll_offset = 1;
        } else if self.paused {
            self.scroll_offset += 1;
        }
        self.messages.push_back(msg);
        if self.messages.len() > max_scrollback {
            self.messages.pop_front();
            if self.scroll_offset > 0 {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }
}

/// The connection lifecycle state of an IRC server.
#[derive(Debug, Clone, PartialEq)]
pub enum ConnectionStatus {
    Disconnected,
    Connecting,
    Connected,
}

/// Runtime state for a single IRC server connection.
///
/// Tracks the connection status, current nickname, joined channels, per-channel
/// user lists, and topic strings.
#[derive(Debug)]
pub struct ServerState {
    pub id: ServerId,
    /// User-facing label (e.g. `"libera"`).
    pub name: String,
    /// Hostname or IP address of the IRC server.
    pub host: String,
    pub port: u16,
    /// Whether to use TLS for this connection.
    pub tls: bool,
    /// The nickname currently registered on this server.
    pub nickname: String,
    /// Pre-lowercased nickname for case-insensitive comparisons.
    pub nickname_lower: String,
    pub status: ConnectionStatus,
    /// Channels the user is currently in (or intending to join).
    pub channels: Vec<String>,
    /// Per-channel user lists, keyed by channel name.
    pub users: HashMap<String, Vec<ChannelUser>>,
    /// Per-channel topic strings, keyed by channel name.
    pub topics: HashMap<String, String>,
    /// Whether the user has set themselves as away on this server.
    pub is_away: bool,
    /// Index into the `alt_nicks` list for nickname collision fallback.
    pub alt_nick_index: usize,
    /// Whether to accept invalid/self-signed TLS certificates.
    pub accept_invalid_certs: bool,
}

/// A user present in a channel, with their mode prefix.
#[derive(Debug, Clone)]
pub struct ChannelUser {
    pub nick: String,
    /// Mode prefix character: `"@"` (op), `"+"` (voice), `"%"` (halfop), etc.
    pub prefix: String,
}

impl ChannelUser {
    /// Returns the nick prefixed with its mode symbol (e.g. `"@alice"`).
    #[allow(dead_code)]
    pub fn display_name(&self) -> String {
        format!("{}{}", self.prefix, self.nick)
    }
}

/// Lifecycle state of a DCC file transfer.
#[derive(Debug, Clone, PartialEq)]
pub enum DccTransferStatus {
    /// Offer received but not yet accepted by the user.
    Pending,
    /// Transfer in progress — bytes are being received.
    Active,
    /// Transfer finished successfully.
    Completed,
    /// Transfer failed with an error description.
    Failed(String),
    /// Transfer was cancelled by the user.
    Cancelled,
}

/// State of a single DCC file transfer (currently receive-only).
#[derive(Debug, Clone)]
pub struct DccTransfer {
    pub id: TransferId,
    /// The server on which this DCC offer was received.
    pub server_id: ServerId,
    /// The nick that sent the DCC SEND offer.
    pub from: String,
    /// Sanitized filename for the download.
    pub filename: String,
    /// Total file size in bytes as advertised by the sender.
    pub size: u64,
    /// Bytes received so far.
    pub received: u64,
    /// IP address to connect to for the transfer.
    pub ip: IpAddr,
    /// TCP port to connect to for the transfer.
    pub port: u16,
    pub status: DccTransferStatus,
}

/// The user's text input field state, with cursor movement and command history.
#[derive(Debug)]
pub struct InputState {
    /// The current input text (UTF-8).
    pub text: String,
    /// Byte offset of the cursor within `text`.
    pub cursor: usize,
    /// Previously submitted input lines (newest at the end).
    pub history: Vec<String>,
    /// Current position when browsing history (`None` = editing new text).
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

    /// Insert a character at the cursor position and advance the cursor.
    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    /// Delete the character before the cursor (Backspace).
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

    /// Delete the character after the cursor (Delete key).
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

    /// Extract the current text, reset the input, and push it to history.
    /// Returns the submitted text (may be empty).
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

    /// Navigate to the previous entry in command history (Up arrow).
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

    /// Navigate to the next entry in command history (Down arrow).
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

    /// Delete the word before the cursor (Alt+Backspace / Ctrl+W).
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

/// Which panel currently has keyboard focus (cycled with Tab).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FocusPanel {
    /// The server/channel tree on the left.
    ServerTree,
    /// The main message area (scrollable).
    MessageArea,
    /// The text input box at the bottom.
    Input,
    /// The user list on the right.
    UserList,
}

/// Modal overlay for browsing and connecting to built-in/custom IRC servers.
/// Toggled with `F2` or `/servers`.
#[derive(Debug)]
pub struct ServerBrowser {
    pub visible: bool,
    pub selected: usize,
    pub scroll_offset: usize,
}

impl ServerBrowser {
    pub fn new() -> Self {
        Self {
            visible: false,
            selected: 0,
            scroll_offset: 0,
        }
    }

    pub fn toggle(&mut self) {
        self.visible = !self.visible;
        if self.visible {
            self.selected = 0;
            self.scroll_offset = 0;
        }
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
            if self.selected < self.scroll_offset {
                self.scroll_offset = self.selected;
            }
        }
    }

    pub fn move_down(&mut self, total: usize) {
        if total > 0 && self.selected + 1 < total {
            self.selected += 1;
        }
    }

    pub fn ensure_visible(&mut self, visible_rows: usize) {
        if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected.saturating_sub(visible_rows - 1);
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }
}

/// Strip mIRC color codes (\x03NN,MM) from a string
fn strip_color_codes(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x03' {
            // Skip up to 2 digits (foreground)
            for _ in 0..2 {
                if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                    chars.next();
                } else {
                    break;
                }
            }
            // Skip optional comma + up to 2 digits (background)
            if chars.peek() == Some(&',') {
                chars.next();
                for _ in 0..2 {
                    if chars.peek().map(|c| c.is_ascii_digit()).unwrap_or(false) {
                        chars.next();
                    } else {
                        break;
                    }
                }
            }
        } else {
            result.push(c);
        }
    }
    result
}

/// A single entry in the channel list returned by the IRC LIST command.
#[derive(Debug, Clone)]
pub struct ChannelListEntry {
    /// Channel name (e.g. `"#rust"`).
    pub name: String,
    /// Number of users currently in the channel.
    pub users: usize,
    /// The channel topic (with IRC formatting codes stripped).
    pub topic: String,
}

/// Modal overlay for browsing the server's channel list.
/// Toggled with `F3` or `/channels`. Supports filtering and caching.
#[derive(Debug)]
pub struct ChannelBrowser {
    pub visible: bool,
    pub loading: bool,
    pub channels: Vec<ChannelListEntry>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub server_id: Option<ServerId>,
    /// Cached channel lists per server, keyed by ServerId
    pub cache: HashMap<ServerId, Vec<ChannelListEntry>>,
}

impl ChannelBrowser {
    pub fn new() -> Self {
        Self {
            visible: false,
            loading: false,
            channels: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            server_id: None,
            cache: HashMap::new(),
        }
    }

    /// Open the browser. Returns true if we need to fetch (no cache), false if cache hit.
    pub fn open(&mut self, server_id: ServerId) -> bool {
        self.visible = true;
        self.selected = 0;
        self.scroll_offset = 0;
        self.filter.clear();
        self.server_id = Some(server_id);

        // Check cache
        if let Some(cached) = self.cache.get(&server_id) {
            self.channels = cached.clone();
            self.loading = false;
            self.apply_filter();
            false // no fetch needed
        } else {
            self.channels.clear();
            self.filtered.clear();
            self.loading = true;
            true // fetch needed
        }
    }

    pub fn close(&mut self) {
        self.visible = false;
        self.loading = false;
        // Keep channels for cache, don't clear
    }

    pub fn add_channel(&mut self, name: String, users: usize, topic: String) {
        // Strip IRC formatting codes from topic for clean display
        let clean_topic: String = topic
            .chars()
            .filter(|&c| !matches!(c, '\x02' | '\x1D' | '\x1F' | '\x0F' | '\x16'))
            .collect();
        // Strip color codes (\x03 followed by optional digits)
        let clean_topic = strip_color_codes(&clean_topic);
        self.channels.push(ChannelListEntry {
            name,
            users,
            topic: clean_topic,
        });
    }

    pub fn finish_loading(&mut self) {
        self.loading = false;
        // Sort by user count descending
        self.channels.sort_by(|a, b| b.users.cmp(&a.users));
        // Cache the result
        if let Some(sid) = self.server_id {
            self.cache.insert(sid, self.channels.clone());
        }
        self.apply_filter();
    }

    /// Force refresh: clear cache for this server and re-fetch
    pub fn refresh(&mut self) {
        if let Some(sid) = self.server_id {
            self.cache.remove(&sid);
        }
        self.channels.clear();
        self.filtered.clear();
        self.selected = 0;
        self.scroll_offset = 0;
        self.loading = true;
    }

    pub fn apply_filter(&mut self) {
        let filter_lower = self.filter.to_lowercase();
        self.filtered = self
            .channels
            .iter()
            .enumerate()
            .filter(|(_, ch)| {
                if filter_lower.is_empty() {
                    return true;
                }
                ch.name.to_lowercase().contains(&filter_lower)
                    || ch.topic.to_lowercase().contains(&filter_lower)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn ensure_visible(&mut self, visible_rows: usize) {
        if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected.saturating_sub(visible_rows - 1);
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn selected_channel(&self) -> Option<&ChannelListEntry> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.channels.get(idx))
    }
}

/// A single parsed search result from a SearchBot results file.
#[derive(Debug, Clone)]
pub struct SearchResultItem {
    /// The full command to send (e.g. "!Bsk Adventures of Captain Underpants.epub").
    pub command: String,
    /// The bot name (e.g. "Bsk").
    pub bot: String,
    /// The filename (e.g. "Adventures of Captain Underpants.epub").
    pub filename: String,
    /// The file size string from ::INFO:: (e.g. "24.0MB").
    pub size: String,
}

/// Modal overlay for browsing search results from a SearchBot results file.
#[derive(Debug)]
pub struct SearchResultsBrowser {
    pub visible: bool,
    pub items: Vec<SearchResultItem>,
    pub filtered: Vec<usize>,
    pub selected: usize,
    pub scroll_offset: usize,
    pub filter: String,
    pub title: String,
}

impl SearchResultsBrowser {
    pub fn new() -> Self {
        Self {
            visible: false,
            items: Vec::new(),
            filtered: Vec::new(),
            selected: 0,
            scroll_offset: 0,
            filter: String::new(),
            title: String::new(),
        }
    }

    pub fn open(&mut self, items: Vec<SearchResultItem>, title: String) {
        self.items = items;
        self.title = title;
        self.visible = true;
        self.selected = 0;
        self.scroll_offset = 0;
        self.filter.clear();
        self.apply_filter();
    }

    pub fn close(&mut self) {
        self.visible = false;
    }

    pub fn apply_filter(&mut self) {
        let filter_lower = self.filter.to_lowercase();
        self.filtered = self
            .items
            .iter()
            .enumerate()
            .filter(|(_, item)| {
                if filter_lower.is_empty() {
                    return true;
                }
                item.bot.to_lowercase().contains(&filter_lower)
                    || item.filename.to_lowercase().contains(&filter_lower)
            })
            .map(|(i, _)| i)
            .collect();
        self.selected = 0;
        self.scroll_offset = 0;
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        if !self.filtered.is_empty() && self.selected + 1 < self.filtered.len() {
            self.selected += 1;
        }
    }

    pub fn ensure_visible(&mut self, visible_rows: usize) {
        if self.selected >= self.scroll_offset + visible_rows {
            self.scroll_offset = self.selected.saturating_sub(visible_rows - 1);
        }
        if self.selected < self.scroll_offset {
            self.scroll_offset = self.selected;
        }
    }

    pub fn selected_item(&self) -> Option<&SearchResultItem> {
        self.filtered
            .get(self.selected)
            .and_then(|&idx| self.items.get(idx))
    }
}

/// State for the command autocomplete popup shown when typing `/`.
#[derive(Debug)]
pub struct AutocompleteState {
    /// Filtered command suggestions matching the current input.
    pub suggestions: Vec<String>,
    /// Index of the currently highlighted suggestion.
    pub selected: usize,
    /// Whether the autocomplete popup is currently visible.
    pub visible: bool,
}

impl AutocompleteState {
    pub fn new() -> Self {
        Self {
            suggestions: Vec::new(),
            selected: 0,
            visible: false,
        }
    }
}

/// A delayed channel rejoin scheduled after being kicked.
#[derive(Debug)]
pub struct PendingRejoin {
    pub server_id: ServerId,
    pub channel: String,
    /// When to execute the rejoin (after the configured delay).
    pub rejoin_at: Instant,
}

/// The central application state — single source of truth for the entire UI and
/// all IRC/DCC subsystems. Mutated exclusively by the event handler.
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
    pub pending_actions: Vec<Action>,
    pub pending_rejoins: Vec<PendingRejoin>,
    pub ignore_list: HashSet<String>,
    pub notify_list: HashSet<String>,
    pub known_online: HashSet<String>,
    pub pending_bell: bool,
    pub quit_message: Option<String>,
    pub new_messages: Vec<(BufferKey, Message)>,
    pub last_ison_check: Instant,
    pub server_browser: ServerBrowser,
    pub channel_browser: ChannelBrowser,
    pub search_results: SearchResultsBrowser,
    pub autocomplete: AutocompleteState,
    pub tick_count: u64,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        let timestamp_format = config.ui.timestamp_format.clone();
        let mut buffers = BTreeMap::new();
        buffers.insert(BufferKey::Highlights, Buffer::new());
        Self {
            config,
            servers: Vec::new(),
            buffers,
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
            pending_actions: Vec::new(),
            pending_rejoins: Vec::new(),
            ignore_list: HashSet::new(),
            notify_list: HashSet::new(),
            known_online: HashSet::new(),
            pending_bell: false,
            quit_message: None,
            new_messages: Vec::new(),
            last_ison_check: Instant::now(),
            server_browser: ServerBrowser::new(),
            channel_browser: ChannelBrowser::new(),
            search_results: SearchResultsBrowser::new(),
            autocomplete: AutocompleteState::new(),
            tick_count: 0,
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
        self.new_messages.push((key.clone(), msg.clone()));
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
        self.active_buffer.as_ref().and_then(|k| match k {
            BufferKey::Highlights => None,
            BufferKey::ServerStatus(id) => Some(*id),
            BufferKey::Channel(id, _) => Some(*id),
            BufferKey::Query(id, _) => Some(*id),
        })
    }

    /// Add a message to the global Highlights buffer with a source label prefix.
    pub fn add_highlight(&mut self, source_label: &str, msg: &Message) {
        let highlight_msg = Message {
            timestamp: msg.timestamp.clone(),
            sender: msg.sender.clone(),
            text: format!("[{}] {}", source_label, msg.text),
            kind: msg.kind.clone(),
        };
        let key = BufferKey::Highlights;
        let max = self.config.ui.max_scrollback;
        let is_active = self.active_buffer.as_ref() == Some(&key);
        let buf = self.buffers.entry(key).or_insert_with(Buffer::new);
        buf.add_message(highlight_msg, max);
        if !is_active {
            buf.unread_count += 1;
            buf.has_mention = true;
        }
        self.dirty = true;
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
        let connected = self
            .servers
            .iter()
            .filter(|s| s.status == ConnectionStatus::Connected)
            .count();
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
