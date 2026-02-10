# CrabChat Architecture

## System Overview

CrabChat is an event-driven, async IRC client built on Tokio. The architecture cleanly separates concerns into five layers:

```
┌─────────────────────────────────────────────────────────────┐
│                    Terminal (crossterm)                      │
│              keyboard, mouse, resize events                 │
└──────────────────────────┬──────────────────────────────────┘
                           │ AppEvent::Terminal
┌──────────────────────────▼──────────────────────────────────┐
│                     Main Event Loop                         │
│                      (src/main.rs)                          │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Event Recv  │──│ Handler      │──│ Action Dispatch   │  │
│  │ (mpsc rx)   │  │ (state mut)  │  │ (IRC/DCC/Quit)    │  │
│  └─────────────┘  └──────────────┘  └───────────────────┘  │
│         │                │                    │             │
│         │          ┌─────▼─────┐        ┌────▼─────┐       │
│         │          │ AppState  │        │ Logging  │       │
│         │          │ (central) │        └──────────┘       │
│         │          └─────┬─────┘                           │
│         │                │ dirty flag                       │
│         │          ┌─────▼─────┐                           │
│         │          │ UI Render │                           │
│         │          │ (ratatui) │                           │
│         │          └───────────┘                           │
└─────────┼──────────────────────────────────────────────────┘
          │
          │ AppEvent::IrcMessage / IrcConnected / DccProgress / Tick
          │
┌─────────┴──────────────────────────────────────────────────┐
│                   Background Tasks (tokio::spawn)           │
│                                                             │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────┐  │
│  │ IRC Reader   │  │ IRC Reader   │  │ DCC Receive      │  │
│  │ (server 1)   │  │ (server 2)   │  │ (transfer N)     │  │
│  └──────────────┘  └──────────────┘  └──────────────────┘  │
│  ┌──────────────┐  ┌──────────────┐                        │
│  │ Terminal     │  │ Tick Timer   │                        │
│  │ Input Reader │  │ (20 FPS)     │                        │
│  └──────────────┘  └──────────────┘                        │
└─────────────────────────────────────────────────────────────┘
```

## Core Design Principles

1. **Single event channel** — All async producers (terminal, IRC, DCC, timer) send events through one `mpsc::unbounded_channel`. The main loop is the sole consumer.

2. **State + Handler separation** — `AppState` holds all mutable data. The `handler` module is a pure function `(state, event) → (state', actions)` that never touches I/O directly.

3. **Action dispatch** — Side effects (sending IRC commands, starting DCC downloads) are represented as `Action` values returned by the handler. The main loop processes them, keeping I/O out of the handler.

4. **Lazy rendering** — The UI only re-renders when `state.dirty == true`, avoiding unnecessary terminal writes on every tick.

## Module Map

```
src/
├── main.rs              ← Entry point, event loop, action dispatch
├── app/
│   ├── state.rs         ← AppState, Buffer, ServerState, InputState
│   ├── event.rs         ← AppEvent enum, ServerId/TransferId types
│   ├── action.rs        ← Action enum (handler → main loop)
│   └── handler.rs       ← Event processing, command handling, IRC message parsing
├── config/
│   ├── mod.rs           ← load_config() / save_config()
│   ├── model.rs         ← AppConfig, ServerConfig, UiConfig, DccConfig, ...
│   └── nickname.rs      ← Random nickname generator (AdjectiveNounNN)
├── irc/
│   ├── manager.rs       ← IrcManager: owns connections, sends commands
│   ├── connection.rs    ← spawn_connection(): client setup + message reader
│   └── commands.rs      ← Slash-command parser (/join, /msg, /kick, ...)
├── dcc/
│   ├── manager.rs       ← DccManager: accept/cancel transfers
│   ├── parser.rs        ← DCC SEND CTCP parsing
│   ├── transfer.rs      ← TCP file receive with DCC acknowledgment protocol
│   └── security.rs      ← Filename sanitization, path traversal prevention, IP checks
├── ui/
│   ├── mod.rs           ← render(): top-level composition
│   ├── layout.rs        ← compute_layout(): responsive panel sizing
│   ├── theme.rs         ← Midnight Ocean color palette and style helpers
│   ├── message_area.rs  ← Scrollable message display with mIRC colors
│   ├── mirc_colors.rs   ← mIRC \x03NN,MM color code parser
│   ├── server_tree.rs   ← Left panel: servers, channels, queries, badges
│   ├── user_list.rs     ← Right panel: channel members with mode prefixes
│   ├── input_box.rs     ← Bottom input field with cursor
│   ├── topic_bar.rs     ← Channel topic display
│   ├── status_bar.rs    ← Connection summary line
│   ├── server_browser.rs← Modal: built-in server list
│   └── channel_browser.rs← Modal: searchable channel list with caching
└── logging/
    └── mod.rs           ← ChatLogger: per-channel daily log files
```

## Component Relationships

```
                    ┌────────────────┐
                    │   AppConfig    │
                    │ (config/model) │
                    └───────┬────────┘
                            │ read at startup
                    ┌───────▼────────┐
                    │   AppState     │◄──── handler mutates
                    │  (app/state)   │
                    └┬───┬───┬───┬───┘
                     │   │   │   │
        ┌────────────┘   │   │   └────────────┐
        │                │   │                │
   ┌────▼───┐    ┌──────▼───▼──┐      ┌──────▼──────┐
   │Servers │    │  Buffers    │      │ Transfers   │
   │Vec     │    │  BTreeMap   │      │ Vec         │
   └────────┘    └─────────────┘      └─────────────┘
        │                │                    │
        │    read by     │     read by        │
   ┌────▼────────────────▼────────────────────▼────┐
   │              UI Render Layer                   │
   │  server_tree │ message_area │ status_panel     │
   └────────────────────────────────────────────────┘
```

## Data Flows

### 1. User Sends a Chat Message

```
User types "hello" and presses Enter
    │
    ▼
handler::handle_input_key()
    ├── Extracts target from active_buffer (e.g. Channel(1, "#rust"))
    ├── Creates Message { sender: own_nick, text: "hello", kind: Normal }
    ├── Adds message to buffer (optimistic local echo)
    └── Returns Action::SendMessage { server_id: 1, target: "#rust", text: "hello" }
         │
         ▼
    main loop processes action
         │
         ▼
    IrcManager::send_privmsg(1, "#rust", "hello")
         │
         ▼
    irc::Sender writes PRIVMSG to TCP socket
         │
         ▼
    Server relays to other channel members
```

### 2. Incoming IRC Message

```
IRC server sends: :alice!user@host PRIVMSG #rust :hi everyone
    │
    ▼
Background IRC reader task (spawned per connection)
    │
    ▼
event_tx.send(AppEvent::IrcMessage { server_id: 1, message })
    │
    ▼
Main event loop receives event
    │
    ▼
handler::handle_event() → handle_irc_message()
    ├── Extracts sender ("alice"), target ("#rust"), text ("hi everyone")
    ├── Checks ignore list — skip if ignored
    ├── Detects CTCP wrapping — handle ACTION, DCC, VERSION, etc.
    ├── Checks for nick mention → sets has_mention, triggers bell
    ├── Creates Message and adds to Buffer for Channel(1, "#rust")
    └── Sets state.dirty = true
         │
         ▼
    Main loop: terminal.draw(|f| ui::render(f, &state))
```

### 3. DCC File Transfer

```
Remote user sends: CTCP DCC SEND "photo.jpg" 3232235777 5001 1048576
    │
    ▼
handle_irc_message() detects CTCP DCC
    │
    ▼
dcc::parser::handle_ctcp_dcc()
    ├── Parses: filename="photo.jpg", ip=192.168.1.1, port=5001, size=1MB
    ├── Security: checks reject_private_ips, max_file_size
    ├── Sanitizes filename (path traversal protection)
    ├── Creates DccTransfer { status: Pending, id: 0 }
    └── Shows notification: "DCC SEND offer ... /dcc accept 0"

User types: /dcc accept 0
    │
    ▼
handler::handle_command() → Action::DccAccept { transfer_id: 0 }
    │
    ▼
DccManager::accept_transfer()
    ├── security::safe_download_path() validates destination
    ├── Sets transfer.status = Active
    └── transfer::spawn_receive() spawns TCP receive task
         │
         ▼
    Background task:
    ├── TcpStream::connect(192.168.1.1:5001)
    ├── Loop: read 8KB → write to file → send 4-byte ack
    ├── Every 250ms: emit DccProgress event
    └── On complete: emit DccComplete event
```

### 4. UI Rendering Pipeline

```
state.dirty == true
    │
    ▼
terminal.draw(|frame| ui::render(frame, &state))
    │
    ▼
ui::render()
    ├── Fill background with Theme::BG_DARK
    ├── layout::compute_layout(area) → AppLayout { rects... }
    ├── server_tree::render()    ← left panel
    ├── topic_bar::render()      ← top center
    ├── message_area::render()   ← center (main)
    ├── input_box::render()      ← bottom center
    ├── user_list::render()      ← right panel
    ├── render_status_panel()    ← bottom left (DCC gauges)
    ├── status_bar::render()     ← bottom row
    ├── server_browser::render() ← modal overlay (if visible)
    └── channel_browser::render()← modal overlay (if visible)
```

## UI Layout

```
┌─────────────────┬──────────────────────────────────┬───────────┐
│                 │       Topic / MOTD               │           │
│  Server Tree    ├──────────────────────────────────┤ User List │
│                 │                                  │           │
│  ▸ libera   [2]│  14:32 <alice> hello everyone     │ @alice    │
│    #rust        │  14:32 <bob> hey!                 │ +bob      │
│    #crabchat [!]│  14:33 *** carol has joined       │  carol    │
│  ▸ oftc        │  14:33 <carol> hi :)              │           │
│                 │                                  │           │
│─────────────────│──────────────────────────────────│           │
│  Transfers      │                                  │           │
│  No active      ├──────────────────────────────────┤           │
│  transfers      │ > type here...            │      │           │
├─────────────────┴──────────────────────────────────┴───────────┤
│ Servers: 2/2 | #rust @libera                                   │
└────────────────────────────────────────────────────────────────┘
```

## Security Model

### DCC Protections

| Threat | Mitigation |
|--------|-----------|
| Path traversal (`../../etc/passwd`) | `sanitize_filename()` strips paths, `safe_download_path()` verifies resolved path stays within download dir |
| Hidden files (`.ssh/config`) | Leading dots stripped from filenames |
| Filename collision | Automatic numeric suffix (`file_1.txt`, `file_2.txt`) |
| Private IP abuse | Optional `reject_private_ips` blocks loopback, RFC 1918, link-local |
| Oversized files | Configurable `max_file_size` limit |

### IRC Protections

| Threat | Mitigation |
|--------|-----------|
| CTCP injection via outbound PRIVMSG | `\x01` bytes stripped from all outbound text |
| Nick collision | Alt-nick fallback with `_` suffix |
| Unwanted messages | Per-session ignore list silently drops PRIVMSG/NOTICE/KICK |

## Technology Stack

| Component | Crate | Purpose |
|-----------|-------|---------|
| Async runtime | `tokio` | Task spawning, TCP, timers |
| TUI framework | `ratatui` + `crossterm` | Terminal rendering and input |
| IRC protocol | `irc` | Client library with SASL support |
| TLS | `tokio-rustls` + `rustls` | Secure connections |
| Configuration | `serde` + `toml` | TOML serialization |
| Timestamps | `chrono` | Formatted time strings |
| Error handling | `anyhow` + `thiserror` | Ergonomic error types |
| Text width | `unicode-width` | Correct CJK/emoji column widths |
| Random | `rand` | Nickname generation |
