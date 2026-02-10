use crate::app::event::ServerId;
use crate::app::state::AppState;
use std::net::{IpAddr, Ipv4Addr};

/// Parse and handle incoming DCC CTCP messages
/// Format: DCC SEND <filename> <ip_decimal> <port> <filesize>
pub fn handle_ctcp_dcc(
    state: &mut AppState,
    server_id: ServerId,
    from: &str,
    ctcp_content: &str,
) {
    if let Some(offer) = parse_dcc_send(ctcp_content) {
        let transfer_id = state.allocate_transfer_id();

        let key = state
            .active_buffer
            .clone()
            .unwrap_or(crate::app::state::BufferKey::ServerStatus(server_id));

        // Check security
        if state.config.dcc.reject_private_ips && crate::dcc::security::is_private_ip(&offer.ip) {
            state.error_message(
                &key,
                format!(
                    "DCC SEND from {} rejected: private/loopback IP {} (set reject_private_ips = false in config to allow)",
                    from, offer.ip
                ),
            );
            return;
        }

        if offer.size > state.config.dcc.max_file_size {
            state.error_message(
                &key,
                format!(
                    "DCC SEND from {} rejected: file size {} exceeds limit {}",
                    from, offer.size, state.config.dcc.max_file_size
                ),
            );
            return;
        }

        let transfer = crate::app::state::DccTransfer {
            id: transfer_id,
            server_id,
            from: from.to_string(),
            filename: offer.filename.clone(),
            size: offer.size,
            received: 0,
            ip: offer.ip,
            port: offer.port,
            status: crate::app::state::DccTransferStatus::Pending,
        };
        state.transfers.push(transfer);
        state.system_message(
            &key,
            format!(
                "DCC SEND offer from {}: \"{}\" ({} bytes) [id: {}] — /dcc accept {} to download",
                from, offer.filename, offer.size, transfer_id, transfer_id
            ),
        );
    }
}

pub struct DccSendOffer {
    pub filename: String,
    pub ip: IpAddr,
    pub port: u16,
    pub size: u64,
}

pub fn parse_dcc_send(ctcp: &str) -> Option<DccSendOffer> {
    // DCC SEND <filename> <ip> <port> <size>
    // Filename may be quoted
    let content = ctcp.strip_prefix("DCC SEND ")?;

    let (filename, rest) = if content.starts_with('"') {
        let end_quote = content[1..].find('"')?;
        let filename = &content[1..=end_quote];
        let rest = content[end_quote + 2..].trim();
        (filename.to_string(), rest)
    } else {
        let space = content.find(' ')?;
        let filename = &content[..space];
        let rest = content[space + 1..].trim();
        (filename.to_string(), rest)
    };

    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let ip_decimal: u32 = parts[0].parse().ok()?;
    let ip = IpAddr::V4(Ipv4Addr::from(ip_decimal));
    let port: u16 = parts[1].parse().ok()?;
    let size: u64 = parts[2].parse().ok()?;

    // Sanitize filename
    let filename = crate::dcc::security::sanitize_filename(&filename)?;

    Some(DccSendOffer {
        filename,
        ip,
        port,
        size,
    })
}
