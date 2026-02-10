use crossterm::event::Event as CrosstermEvent;

pub type ServerId = usize;
pub type TransferId = usize;

#[derive(Debug)]
pub enum AppEvent {
    /// Terminal input event
    Terminal(CrosstermEvent),

    /// IRC message received from a server
    IrcMessage {
        server_id: ServerId,
        message: irc::client::prelude::Message,
    },

    /// IRC connection state changed
    IrcConnected {
        server_id: ServerId,
    },
    IrcDisconnected {
        server_id: ServerId,
        reason: String,
    },
    IrcError {
        server_id: ServerId,
        error: String,
    },

    /// DCC events
    DccOfferReceived {
        server_id: ServerId,
        from: String,
        filename: String,
        size: u64,
        ip: std::net::IpAddr,
        port: u16,
        transfer_id: TransferId,
    },
    DccProgress {
        transfer_id: TransferId,
        bytes_received: u64,
        total: u64,
    },
    DccComplete {
        transfer_id: TransferId,
    },
    DccFailed {
        transfer_id: TransferId,
        error: String,
    },

    /// Tick for UI refresh
    Tick,
}
