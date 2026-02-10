//! Single IRC server connection management.
//!
//! Each connection creates an [`irc::client::Client`], performs NICK/USER
//! identification, and spawns an async task that reads messages from the server
//! and forwards them as [`AppEvent::IrcMessage`] to the main event channel.

use crate::app::event::{AppEvent, ServerId};
use anyhow::Result;
use futures::StreamExt;
use irc::client::prelude::*;
use tokio::sync::mpsc;

/// A live IRC connection, holding the sender half used to write commands.
pub struct IrcConnection {
    #[allow(dead_code)]
    pub server_id: ServerId,
    pub sender: irc::client::Sender,
}

/// Create an IRC client, identify with the server, and spawn a background
/// message reader task. Returns the [`IrcConnection`] for sending commands.
#[allow(clippy::too_many_arguments)]
pub async fn spawn_connection(
    server_id: ServerId,
    host: String,
    port: u16,
    tls: bool,
    nickname: String,
    username: Option<String>,
    realname: Option<String>,
    password: Option<String>,
    nick_password: Option<String>,
    channels: Vec<String>,
    accept_invalid_certs: bool,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> Result<IrcConnection> {
    let config = Config {
        server: Some(host),
        port: Some(port),
        use_tls: Some(tls),
        nickname: Some(nickname),
        username,
        realname,
        password,
        nick_password,
        channels,
        dangerously_accept_invalid_certs: Some(accept_invalid_certs),
        ..Config::default()
    };

    let mut client = Client::from_config(config).await?;
    client.identify()?;

    let sender = client.sender();
    let mut stream = client.stream()?;

    let event_tx_clone = event_tx.clone();
    let _ = event_tx.send(AppEvent::IrcConnected { server_id });

    tokio::spawn(async move {
        while let Some(result) = stream.next().await {
            match result {
                Ok(message) => {
                    if event_tx_clone
                        .send(AppEvent::IrcMessage { server_id, message })
                        .is_err()
                    {
                        break;
                    }
                }
                Err(e) => {
                    let _ = event_tx_clone.send(AppEvent::IrcError {
                        server_id,
                        error: e.to_string(),
                    });
                    break;
                }
            }
        }
        let _ = event_tx_clone.send(AppEvent::IrcDisconnected {
            server_id,
            reason: "Connection closed".to_string(),
        });
    });

    Ok(IrcConnection { server_id, sender })
}
