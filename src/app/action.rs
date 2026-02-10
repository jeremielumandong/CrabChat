//! Actions produced by the event handler.
//!
//! When the event handler processes an [`AppEvent`](super::event::AppEvent), it
//! may return one or more [`Action`] values. The main loop then dispatches each
//! action to the appropriate subsystem (IRC manager, DCC manager, etc.).

use crate::app::event::ServerId;

/// A side-effect requested by the event handler.
///
/// Actions decouple the event handler (pure state mutation) from I/O operations
/// so that the handler never touches network sockets directly.
#[derive(Debug)]
#[allow(clippy::enum_variant_names)]
pub enum Action {
    /// Send a PRIVMSG to a channel or user (user-typed text).
    SendMessage {
        server_id: ServerId,
        target: String,
        text: String,
    },
    /// Send a CTCP ACTION (/me) to a channel or user.
    SendAction {
        server_id: ServerId,
        target: String,
        text: String,
    },
    /// Join an IRC channel.
    JoinChannel {
        server_id: ServerId,
        channel: String,
    },
    /// Leave an IRC channel with an optional reason.
    PartChannel {
        server_id: ServerId,
        channel: String,
        reason: Option<String>,
    },
    /// Request a nickname change.
    ChangeNick { server_id: ServerId, nick: String },
    /// Send a PRIVMSG (programmatic, e.g. from /msg command).
    SendPrivmsg {
        server_id: ServerId,
        target: String,
        text: String,
    },
    /// Establish a new IRC connection.
    ConnectServer {
        name: String,
        host: String,
        port: u16,
        tls: bool,
        nick: String,
        accept_invalid_certs: bool,
    },
    /// Disconnect from an IRC server.
    DisconnectServer { server_id: ServerId },
    /// Accept a pending DCC file transfer.
    DccAccept { transfer_id: usize },
    /// Cancel a DCC file transfer.
    DccCancel { transfer_id: usize },
    /// Quit the application with an optional quit message.
    Quit { message: Option<String> },
    /// Kick a user from a channel.
    SendKick {
        server_id: ServerId,
        channel: String,
        user: String,
        reason: Option<String>,
    },
    /// Set channel or user modes.
    SendMode {
        server_id: ServerId,
        target: String,
        modes: String,
    },
    /// Change a channel's topic.
    SetTopic {
        server_id: ServerId,
        channel: String,
        text: String,
    },
    /// Send a NOTICE to a channel or user.
    SendNotice {
        server_id: ServerId,
        target: String,
        text: String,
    },
    /// Query WHOIS information for a user.
    SendWhois { server_id: ServerId, nick: String },
    /// Send a WHO query.
    SendWho { server_id: ServerId, target: String },
    /// Set or clear the away status.
    SetAway {
        server_id: ServerId,
        message: Option<String>,
    },
    /// Send a raw IRC command.
    SendRaw {
        server_id: ServerId,
        command: String,
    },
    /// Request the channel list from the server.
    SendList { server_id: ServerId },
    /// Send a CTCP request to a user.
    SendCtcp {
        server_id: ServerId,
        target: String,
        command: String,
    },
    /// Send a CTCP reply (via NOTICE) to a user.
    SendCtcpReply {
        server_id: ServerId,
        target: String,
        response: String,
    },
    /// Send an ISON query to check if nicks are online.
    SendIson { server_id: ServerId, nicks: String },
}
