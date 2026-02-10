mod app;
mod config;
mod dcc;
mod irc;
mod ui;

use crate::app::action::Action;
use crate::app::event::AppEvent;
use crate::app::handler;
use crate::app::state::*;
use crate::dcc::manager::DccManager;
use crate::irc::manager::IrcManager;
use anyhow::Result;
use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, EventStream},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use futures::StreamExt;
use ratatui::prelude::*;
use std::io;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<()> {
    // Install panic hook to restore terminal
    let original_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        original_hook(info);
    }));

    // Load config
    let cfg = config::load_config()?;

    // Ensure download directory exists
    std::fs::create_dir_all(&cfg.dcc.download_dir)?;

    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Run app
    let result = run_app(&mut terminal, cfg).await;

    // Restore terminal
    restore_terminal()?;

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        std::process::exit(1);
    }

    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(
        io::stdout(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    Ok(())
}

async fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    cfg: config::AppConfig,
) -> Result<()> {
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<AppEvent>();

    let mut state = AppState::new(cfg.clone());
    let mut irc_manager = IrcManager::new(event_tx.clone());
    let dcc_manager = DccManager::new(event_tx.clone());

    // Spawn terminal input task
    let term_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        loop {
            match reader.next().await {
                Some(Ok(event)) => {
                    if term_tx.send(AppEvent::Terminal(event)).is_err() {
                        break;
                    }
                }
                Some(Err(_)) => break,
                None => break,
            }
        }
    });

    // Spawn tick task (20 FPS = 50ms)
    let tick_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_millis(50));
        loop {
            interval.tick().await;
            if tick_tx.send(AppEvent::Tick).is_err() {
                break;
            }
        }
    });

    // Auto-connect servers from config
    for srv_cfg in &cfg.servers {
        if srv_cfg.auto_connect {
            let server_id = state.allocate_server_id();
            let server = ServerState {
                id: server_id,
                name: srv_cfg.name.clone(),
                host: srv_cfg.host.clone(),
                port: srv_cfg.port,
                tls: srv_cfg.tls,
                nickname: srv_cfg.nickname.clone(),
                status: ConnectionStatus::Connecting,
                channels: srv_cfg.channels.clone(),
                users: Default::default(),
                topics: Default::default(),
            };
            state.add_server(server);
            let srv = state.get_server(server_id).unwrap();
            if let Err(e) = irc_manager.connect(srv).await {
                let key = BufferKey::ServerStatus(server_id);
                state.error_message(&key, format!("Connection failed: {}", e));
                if let Some(srv) = state.get_server_mut(server_id) {
                    srv.status = ConnectionStatus::Disconnected;
                }
            }
        }
    }

    // If no active connections, show a welcome buffer
    if state.servers.is_empty() {
        let server_id = state.allocate_server_id();
        let server = ServerState {
            id: server_id,
            name: "welcome".to_string(),
            host: String::new(),
            port: 0,
            tls: false,
            nickname: "user".to_string(),
            status: ConnectionStatus::Disconnected,
            channels: Vec::new(),
            users: Default::default(),
            topics: Default::default(),
        };
        state.add_server(server);
        let key = BufferKey::ServerStatus(server_id);
        state.system_message(&key, "Welcome to ircchat!".to_string());
        state.system_message(&key, String::new());
        state.system_message(&key, "Built-in servers:".to_string());
        for srv in &cfg.servers {
            state.system_message(&key, format!("  {}  ({}:{})", srv.name, srv.host, srv.port));
        }
        state.system_message(&key, String::new());
        state.system_message(&key, "Quick connect:  /server connect <name>".to_string());
        state.system_message(&key, "Custom server:  /server add <name> <host:port>".to_string());
        state.system_message(&key, "Help:           /help".to_string());
    }

    // Initial render
    terminal.draw(|f| ui::render(f, &state))?;

    // Main event loop
    loop {
        let event = event_rx.recv().await;
        let Some(event) = event else { break };

        let actions = handler::handle_event(&mut state, event);

        // Process actions
        for action in actions {
            match action {
                Action::SendMessage {
                    server_id,
                    target,
                    text,
                } => {
                    if let Err(e) = irc_manager.send_privmsg(server_id, &target, &text) {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Send failed: {}", e));
                    }
                }
                Action::SendAction {
                    server_id,
                    target,
                    text,
                } => {
                    if let Err(e) = irc_manager.send_action(server_id, &target, &text) {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Send failed: {}", e));
                    }
                }
                Action::JoinChannel {
                    server_id,
                    channel,
                } => {
                    if let Err(e) = irc_manager.send_join(server_id, &channel) {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Join failed: {}", e));
                    }
                }
                Action::PartChannel {
                    server_id,
                    channel,
                    reason,
                } => {
                    if let Err(e) =
                        irc_manager.send_part(server_id, &channel, reason.as_deref())
                    {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Part failed: {}", e));
                    }
                }
                Action::ChangeNick { server_id, nick } => {
                    if let Err(e) = irc_manager.send_nick(server_id, &nick) {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Nick change failed: {}", e));
                    }
                }
                Action::SendPrivmsg {
                    server_id,
                    target,
                    text,
                } => {
                    if let Err(e) = irc_manager.send_privmsg(server_id, &target, &text) {
                        let key = BufferKey::ServerStatus(server_id);
                        state.error_message(&key, format!("Send failed: {}", e));
                    }
                }
                Action::ConnectServer {
                    name,
                    host,
                    port,
                    tls,
                    nick,
                } => {
                    let server_id = state.allocate_server_id();
                    let server = ServerState {
                        id: server_id,
                        name: name.clone(),
                        host: host.clone(),
                        port,
                        tls,
                        nickname: nick.clone(),
                        status: ConnectionStatus::Connecting,
                        channels: Vec::new(),
                        users: Default::default(),
                        topics: Default::default(),
                    };
                    state.add_server(server);
                    let key = BufferKey::ServerStatus(server_id);
                    state.system_message(&key, format!("Connecting to {}:{}...", host, port));
                    state.set_active_buffer(key.clone());

                    let srv = state.get_server(server_id).unwrap();
                    if let Err(e) = irc_manager.connect(srv).await {
                        state.error_message(&key, format!("Connection failed: {}", e));
                        if let Some(srv) = state.get_server_mut(server_id) {
                            srv.status = ConnectionStatus::Disconnected;
                        }
                    }
                }
                Action::DisconnectServer { server_id } => {
                    irc_manager.disconnect(server_id);
                    if let Some(srv) = state.get_server_mut(server_id) {
                        srv.status = ConnectionStatus::Disconnected;
                    }
                    let key = BufferKey::ServerStatus(server_id);
                    state.system_message(&key, "Disconnected.".to_string());
                }
                Action::DccAccept { transfer_id } => {
                    if let Err(e) = dcc_manager.accept_transfer(&mut state, transfer_id).await {
                        if let Some(ref key) = state.active_buffer.clone() {
                            state.error_message(key, format!("DCC accept failed: {}", e));
                        }
                    }
                }
                Action::DccCancel { transfer_id } => {
                    if let Err(e) = dcc_manager.cancel_transfer(&mut state, transfer_id) {
                        if let Some(ref key) = state.active_buffer.clone() {
                            state.error_message(key, format!("DCC cancel failed: {}", e));
                        }
                    }
                }
                Action::Quit => {
                    state.should_quit = true;
                }
            }
        }

        if state.should_quit {
            irc_manager.send_quit_all();
            break;
        }

        // Conditional render (only if dirty)
        if state.dirty {
            terminal.draw(|f| ui::render(f, &state))?;
            state.dirty = false;
        }
    }

    Ok(())
}
