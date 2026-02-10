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
    Quit { message: Option<String> },
    SendKick { server_id: ServerId, channel: String, user: String, reason: Option<String> },
    SendMode { server_id: ServerId, target: String, modes: String },
    SetTopic { server_id: ServerId, channel: String, text: String },
    SendNotice { server_id: ServerId, target: String, text: String },
    SendWhois { server_id: ServerId, nick: String },
    SendWho { server_id: ServerId, target: String },
    SetAway { server_id: ServerId, message: Option<String> },
    SendRaw { server_id: ServerId, command: String },
    SendList { server_id: ServerId },
    SendCtcp { server_id: ServerId, target: String, command: String },
    SendCtcpReply { server_id: ServerId, target: String, response: String },
    SendIson { server_id: ServerId, nicks: String },
}
