//! User slash-command parser.
//!
//! Parses `/command arg1 arg2 ...` input lines into typed [`ParsedCommand`]
//! values that the event handler can act on.

/// A parsed user command. Each variant corresponds to a `/command`.
#[derive(Debug)]
pub enum ParsedCommand {
    ServerAdd { name: String, host: String, port: u16, tls: bool },
    ServerConnect { name: String },
    ServerList,
    ServerDisconnect,
    Join { channel: String },
    Part { channel: Option<String>, reason: Option<String> },
    Nick { nick: String },
    Msg { target: String, text: String },
    Me { text: String },
    DccList,
    DccAccept { id: usize },
    DccCancel { id: usize },
    Quit { message: Option<String> },
    Help,
    Kick { channel: Option<String>, user: String, reason: Option<String> },
    Ban { channel: Option<String>, mask: String },
    Mode { target: String, modes: String },
    Op { channel: Option<String>, nick: String },
    Deop { channel: Option<String>, nick: String },
    Voice { channel: Option<String>, nick: String },
    Devoice { channel: Option<String>, nick: String },
    Topic { text: String },
    Notice { target: String, text: String },
    Whois { nick: String },
    Who { target: String },
    Away { message: Option<String> },
    Raw { command: String },
    List,
    Slap { nick: String },
    Ignore { nick: String },
    Unignore { nick: String },
    IgnoreList,
    Notify { nick: Option<String> },
    Ctcp { target: String, command: String },
    ServerBrowser,
    ChannelBrowser,
}

/// Parse a slash-command string into a [`ParsedCommand`].
///
/// Returns `None` if the input does not start with `/` or is not a recognized
/// command. Commands are case-insensitive.
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
            let arg1 = parts.get(1).map(|s| s.to_string());
            let rest = parts.get(2).map(|s| s.to_string());
            let (channel, reason) = match arg1 {
                Some(ref a) if a.starts_with('#') || a.starts_with('&') => (Some(a.clone()), rest),
                Some(a) => {
                    // First arg is reason text (not a channel), combine with rest
                    let full_reason = if let Some(r) = rest {
                        format!("{} {}", a, r)
                    } else {
                        a
                    };
                    (None, Some(full_reason))
                }
                None => (None, None),
            };
            Some(ParsedCommand::Part { channel, reason })
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
        "quit" | "exit" => {
            let message = if parts.len() > 1 {
                Some(input[1..].splitn(2, ' ').nth(1).unwrap_or("").to_string())
            } else {
                None
            };
            Some(ParsedCommand::Quit { message })
        }
        "help" | "h" => Some(ParsedCommand::Help),
        "kick" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                // /kick #channel user [reason]
                let (user, reason) = match rest {
                    Some(r) => {
                        let mut sp = r.splitn(2, ' ');
                        let u = sp.next().unwrap_or("").to_string();
                        let re = sp.next().map(|s| s.to_string());
                        (u, re)
                    }
                    None => return None,
                };
                Some(ParsedCommand::Kick { channel: Some(arg1), user, reason })
            } else {
                // /kick user [reason]
                Some(ParsedCommand::Kick { channel: None, user: arg1, reason: rest })
            }
        }
        "ban" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                let mask = rest?;
                Some(ParsedCommand::Ban { channel: Some(arg1), mask })
            } else {
                Some(ParsedCommand::Ban { channel: None, mask: arg1 })
            }
        }
        "mode" => {
            let target = parts.get(1)?.to_string();
            let modes = parts.get(2).map(|s| s.to_string()).unwrap_or_default();
            Some(ParsedCommand::Mode { target, modes })
        }
        "op" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.trim().to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                Some(ParsedCommand::Op { channel: Some(arg1), nick: rest? })
            } else {
                Some(ParsedCommand::Op { channel: None, nick: arg1 })
            }
        }
        "deop" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.trim().to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                Some(ParsedCommand::Deop { channel: Some(arg1), nick: rest? })
            } else {
                Some(ParsedCommand::Deop { channel: None, nick: arg1 })
            }
        }
        "voice" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.trim().to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                Some(ParsedCommand::Voice { channel: Some(arg1), nick: rest? })
            } else {
                Some(ParsedCommand::Voice { channel: None, nick: arg1 })
            }
        }
        "devoice" => {
            let arg1 = parts.get(1)?.to_string();
            let rest = parts.get(2).map(|s| s.trim().to_string());
            if arg1.starts_with('#') || arg1.starts_with('&') {
                Some(ParsedCommand::Devoice { channel: Some(arg1), nick: rest? })
            } else {
                Some(ParsedCommand::Devoice { channel: None, nick: arg1 })
            }
        }
        "topic" | "t" => {
            let text = if input.len() > cmd.len() + 2 {
                input[cmd.len() + 2..].to_string()
            } else {
                String::new()
            };
            Some(ParsedCommand::Topic { text })
        }
        "notice" => {
            let rest = &input[1..];
            let parts: Vec<&str> = rest.splitn(3, ' ').collect();
            let target = parts.get(1)?.to_string();
            let text = parts.get(2).unwrap_or(&"").to_string();
            Some(ParsedCommand::Notice { target, text })
        }
        "whois" | "wi" => {
            let nick = parts.get(1)?.to_string();
            Some(ParsedCommand::Whois { nick })
        }
        "who" => {
            let target = parts.get(1)?.to_string();
            Some(ParsedCommand::Who { target })
        }
        "away" => {
            let message = if parts.len() > 1 {
                Some(input[1..].splitn(2, ' ').nth(1).unwrap_or("").to_string())
            } else {
                None
            };
            Some(ParsedCommand::Away { message })
        }
        "raw" | "quote" => {
            let command = if input.len() > cmd.len() + 2 {
                input[cmd.len() + 2..].to_string()
            } else {
                return None;
            };
            Some(ParsedCommand::Raw { command })
        }
        "list" => Some(ParsedCommand::List),
        "slap" => {
            let nick = parts.get(1)?.to_string();
            Some(ParsedCommand::Slap { nick })
        }
        "ignore" => {
            let nick = parts.get(1)?.to_string();
            Some(ParsedCommand::Ignore { nick })
        }
        "unignore" => {
            let nick = parts.get(1)?.to_string();
            Some(ParsedCommand::Unignore { nick })
        }
        "ignorelist" => Some(ParsedCommand::IgnoreList),
        "notify" => {
            let nick = parts.get(1).map(|s| s.to_string());
            Some(ParsedCommand::Notify { nick })
        }
        "ctcp" => {
            let target = parts.get(1)?.to_string();
            let command = parts.get(2).map(|s| s.to_string()).unwrap_or_else(|| "VERSION".to_string());
            Some(ParsedCommand::Ctcp { target, command })
        }
        "servers" | "browse" => Some(ParsedCommand::ServerBrowser),
        "channels" => Some(ParsedCommand::ChannelBrowser),
        _ => None,
    }
}

/// Parse a `host:port` or `host:+port` address string.
///
/// The `+` prefix on the port indicates explicit TLS. Port 6697 also implies
/// TLS. Returns `(host, port, tls)`. Defaults to port 6697 with TLS.
fn parse_host_port(addr: &str) -> (String, u16, bool) {
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
