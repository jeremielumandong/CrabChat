//! Application event types.
//!
//! All asynchronous inputs (terminal keypresses, IRC messages, DCC progress,
//! periodic ticks) are funnelled through [`AppEvent`] into the main event loop
//! via a `tokio::sync::mpsc` channel.

use crossterm::event::Event as CrosstermEvent;

/// Unique identifier for an IRC server connection within this session.
pub type ServerId = usize;

/// Unique identifier for a DCC file transfer within this session.
pub type TransferId = usize;

/// Events processed by the main event loop.
///
/// Producers include the terminal input reader task, per-server IRC message
/// reader tasks, DCC transfer tasks, and the periodic tick generator.
#[derive(Debug)]
pub enum AppEvent {
    /// A keyboard, mouse, or resize event from the terminal.
    Terminal(CrosstermEvent),

    /// A raw IRC protocol message received from a connected server.
    IrcMessage {
        server_id: ServerId,
        message: irc::client::prelude::Message,
    },

    /// The IRC client successfully connected and identified with the server.
    IrcConnected {
        server_id: ServerId,
    },

    /// The connection to an IRC server was lost or closed.
    IrcDisconnected {
        server_id: ServerId,
        reason: String,
    },

    /// A non-fatal error occurred on an IRC connection.
    IrcError {
        server_id: ServerId,
        error: String,
    },

    /// A DCC SEND offer was received from another user.
    DccOfferReceived {
        server_id: ServerId,
        from: String,
        filename: String,
        size: u64,
        ip: std::net::IpAddr,
        port: u16,
        transfer_id: TransferId,
    },

    /// Progress update for an active DCC file download.
    DccProgress {
        transfer_id: TransferId,
        bytes_received: u64,
        total: u64,
    },

    /// A DCC file transfer completed successfully.
    DccComplete {
        transfer_id: TransferId,
    },

    /// A DCC file transfer failed.
    DccFailed {
        transfer_id: TransferId,
        error: String,
    },

    /// Periodic tick (20 FPS) used to drive UI refresh and timed operations
    /// such as pending rejoins and ISON checks.
    Tick,
}
