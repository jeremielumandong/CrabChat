#[derive(Debug)]
pub enum ParsedCommand {
    ServerAdd { name: String, host: String, port: u16, tls: bool },
    ServerConnect { name: String },
    ServerList,
    ServerDisconnect,
    Join { channel: String },
    Part { channel: Option<String> },
    Nick { nick: String },
    Msg { target: String, text: String },
    Me { text: String },
    DccList,
    DccAccept { id: usize },
    DccCancel { id: usize },
    Quit,
    Help,
}

pub fn parse_command(input: &str) -> Option<ParsedCommand> {
    let input = input.trim();
    if !input.starts_with('/') {
        return None;
    }

    let parts: Vec<&str> = input[1..].splitn(3, ' ').collect();
    let cmd = parts.first()?.to_lowercase();

    match cmd.as_str() {
        "server" => {
            let subcmd = parts.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
            match subcmd.as_str() {
                "add" => {
                    let rest = parts.get(2)?;
                    let mut rest_parts = rest.splitn(2, ' ');
                    let name = rest_parts.next()?.to_string();
                    let addr = rest_parts.next().unwrap_or(&name);
                    let (host, port, tls) = parse_host_port(addr);
                    Some(ParsedCommand::ServerAdd { name, host, port, tls })
                }
                "connect" => {
                    let name = parts.get(2)?.trim().to_string();
                    Some(ParsedCommand::ServerConnect { name })
                }
                "list" | "ls" => Some(ParsedCommand::ServerList),
                "disconnect" | "dc" => Some(ParsedCommand::ServerDisconnect),
                _ => None,
            }
        }
        "join" | "j" => {
            let channel = parts.get(1)?.to_string();
            let channel = if !channel.starts_with('#') && !channel.starts_with('&') {
                format!("#{}", channel)
            } else {
                channel
            };
            Some(ParsedCommand::Join { channel })
        }
        "part" | "leave" => {
            let channel = parts.get(1).map(|s| s.to_string());
            Some(ParsedCommand::Part { channel })
        }
        "nick" => {
            let nick = parts.get(1)?.to_string();
            Some(ParsedCommand::Nick { nick })
        }
        "msg" | "query" => {
            let rest = &input[1..];
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let target = parts.get(1)?.to_string();
            let text = parts.get(2).unwrap_or(&"").to_string();
            Some(ParsedCommand::Msg { target, text })
        }
        "me" => {
            let text = if input.len() > 4 { input[4..].to_string() } else { String::new() };
            Some(ParsedCommand::Me { text })
        }
        "dcc" => {
            let subcmd = parts.get(1).map(|s| s.to_lowercase()).unwrap_or_default();
            match subcmd.as_str() {
                "list" | "ls" => Some(ParsedCommand::DccList),
                "accept" | "get" => {
                    let id_str = parts.get(2)?.trim();
                    let id = id_str.parse().ok()?;
                    Some(ParsedCommand::DccAccept { id })
                }
                "cancel" | "close" => {
                    let id_str = parts.get(2)?.trim();
                    let id = id_str.parse().ok()?;
                    Some(ParsedCommand::DccCancel { id })
                }
                _ => None,
            }
        }
        "quit" | "exit" => Some(ParsedCommand::Quit),
        "help" | "h" => Some(ParsedCommand::Help),
        _ => None,
    }
}

fn parse_host_port(addr: &str) -> (String, u16, bool) {
    // Handle host:port or host:+port (TLS)
    if let Some(colon_pos) = addr.rfind(':') {
        let host = addr[..colon_pos].to_string();
        let port_str = &addr[colon_pos + 1..];
        let (port_str, tls) = if port_str.starts_with('+') {
            (&port_str[1..], true)
        } else {
            (port_str, port_str.parse::<u16>().map_or(true, |p| p == 6697))
        };
        let port = port_str.parse().unwrap_or(6697);
        (host, port, tls)
    } else {
        (addr.to_string(), 6697, true)
    }
}
