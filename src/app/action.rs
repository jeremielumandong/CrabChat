use crate::app::event::ServerId;

#[derive(Debug)]
pub enum Action {
    SendMessage { server_id: ServerId, target: String, text: String },
    SendAction { server_id: ServerId, target: String, text: String },
    JoinChannel { server_id: ServerId, channel: String },
    PartChannel { server_id: ServerId, channel: String, reason: Option<String> },
    ChangeNick { server_id: ServerId, nick: String },
    SendPrivmsg { server_id: ServerId, target: String, text: String },
    ConnectServer { name: String, host: String, port: u16, tls: bool, nick: String },
    DisconnectServer { server_id: ServerId },
    DccAccept { transfer_id: usize },
    DccCancel { transfer_id: usize },
    Quit,
}
