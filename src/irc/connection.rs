use crate::app::event::{AppEvent, ServerId};
use anyhow::Result;
use futures::StreamExt;
use irc::client::prelude::*;
use tokio::sync::mpsc;

pub struct IrcConnection {
    pub server_id: ServerId,
    pub sender: irc::client::Sender,
}

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
        username: username,
        realname: realname,
        password: password,
        nick_password: nick_password,
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
