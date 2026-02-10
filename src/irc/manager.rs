use crate::app::event::{AppEvent, ServerId};
use crate::app::state::ServerState;
use crate::irc::connection::{spawn_connection, IrcConnection};
use anyhow::Result;
use std::collections::HashMap;
use tokio::sync::mpsc;

pub struct IrcManager {
    connections: HashMap<ServerId, IrcConnection>,
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl IrcManager {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self {
            connections: HashMap::new(),
            event_tx,
        }
    }

    pub async fn connect(&mut self, server: &ServerState) -> Result<()> {
        let conn = spawn_connection(
            server.id,
            server.host.clone(),
            server.port,
            server.tls,
            server.nickname.clone(),
            None,
            None,
            None,
            None,
            server.channels.clone(),
            self.event_tx.clone(),
        )
        .await?;

        self.connections.insert(server.id, conn);
        Ok(())
    }

    pub fn disconnect(&mut self, server_id: ServerId, message: Option<&str>) {
        if let Some(conn) = self.connections.get(&server_id) {
            let _ = conn.sender.send_quit(message.unwrap_or("Leaving"));
        }
        self.connections.remove(&server_id);
    }

    pub fn get_sender(&self, server_id: ServerId) -> Option<&irc::client::Sender> {
        self.connections.get(&server_id).map(|c| &c.sender)
    }

    pub fn send_privmsg(&self, server_id: ServerId, target: &str, text: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            // Validate: no CTCP injection in outbound messages
            let clean = text.replace('\x01', "");
            sender.send_privmsg(target, &clean)?;
        }
        Ok(())
    }

    pub fn send_action(&self, server_id: ServerId, target: &str, text: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            let clean = text.replace('\x01', "");
            let ctcp = format!("\x01ACTION {}\x01", clean);
            sender.send_privmsg(target, &ctcp)?;
        }
        Ok(())
    }

    pub fn send_join(&self, server_id: ServerId, channel: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send_join(channel)?;
        }
        Ok(())
    }

    pub fn send_part(&self, server_id: ServerId, channel: &str, reason: Option<&str>) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::PART(
                channel.to_string(),
                reason.map(|r| r.to_string()),
            ))?;
        }
        Ok(())
    }

    pub fn send_nick(&self, server_id: ServerId, nick: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::NICK(nick.to_string()))?;
        }
        Ok(())
    }

    pub fn send_kick(&self, server_id: ServerId, channel: &str, user: &str, reason: Option<&str>) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::KICK(
                channel.to_string(),
                user.to_string(),
                reason.map(|r| r.to_string()),
            ))?;
        }
        Ok(())
    }

    pub fn send_mode(&self, server_id: ServerId, target: &str, modes: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            // Build raw MODE command since the irc crate uses typed ChannelMODE/UserMODE
            let raw = format!("MODE {} {}", target, modes);
            sender.send(irc::client::prelude::Command::Raw(raw, vec![]))?;
        }
        Ok(())
    }

    pub fn send_topic(&self, server_id: ServerId, channel: &str, text: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::TOPIC(
                channel.to_string(),
                Some(text.to_string()),
            ))?;
        }
        Ok(())
    }

    pub fn send_notice(&self, server_id: ServerId, target: &str, text: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::NOTICE(
                target.to_string(),
                text.to_string(),
            ))?;
        }
        Ok(())
    }

    pub fn send_whois(&self, server_id: ServerId, nick: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::WHOIS(
                None,
                nick.to_string(),
            ))?;
        }
        Ok(())
    }

    pub fn send_who(&self, server_id: ServerId, target: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::WHO(
                Some(target.to_string()),
                None,
            ))?;
        }
        Ok(())
    }

    pub fn send_away(&self, server_id: ServerId, message: Option<&str>) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::AWAY(
                message.map(|m| m.to_string()),
            ))?;
        }
        Ok(())
    }

    pub fn send_raw(&self, server_id: ServerId, command: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::Raw(
                command.to_string(),
                vec![],
            ))?;
        }
        Ok(())
    }

    pub fn send_list(&self, server_id: ServerId) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::LIST(
                None,
                None,
            ))?;
        }
        Ok(())
    }

    pub fn send_ctcp(&self, server_id: ServerId, target: &str, command: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            let ctcp = format!("\x01{}\x01", command);
            sender.send_privmsg(target, &ctcp)?;
        }
        Ok(())
    }

    pub fn send_ctcp_reply(&self, server_id: ServerId, target: &str, response: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            let ctcp = format!("\x01{}\x01", response);
            sender.send(irc::client::prelude::Command::NOTICE(
                target.to_string(),
                ctcp,
            ))?;
        }
        Ok(())
    }

    pub fn send_ison(&self, server_id: ServerId, nicks: &str) -> Result<()> {
        if let Some(sender) = self.get_sender(server_id) {
            sender.send(irc::client::prelude::Command::ISON(
                nicks.split_whitespace().map(|s| s.to_string()).collect(),
            ))?;
        }
        Ok(())
    }

    pub fn send_quit_all(&mut self, message: Option<&str>) {
        let msg = message.unwrap_or("Leaving");
        for (_id, conn) in self.connections.drain() {
            let _ = conn.sender.send_quit(msg);
        }
    }
}
