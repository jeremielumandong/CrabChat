//! DCC file transfer manager.
//!
//! Handles accepting and cancelling DCC SEND transfers. When a transfer is
//! accepted, the download path is validated through the security module and
//! a background TCP receive task is spawned.

use crate::app::event::AppEvent;
use crate::app::state::{AppState, DccTransferStatus};
use crate::dcc::security;
use crate::dcc::transfer;
use anyhow::Result;
use tokio::sync::mpsc;

/// Coordinates DCC file transfers — validates paths, spawns download tasks,
/// and manages transfer lifecycle.
pub struct DccManager {
    event_tx: mpsc::UnboundedSender<AppEvent>,
}

impl DccManager {
    pub fn new(event_tx: mpsc::UnboundedSender<AppEvent>) -> Self {
        Self { event_tx }
    }

    /// Accept a pending DCC transfer: validate the download path, mark the
    /// transfer as active, and spawn a background TCP receive task.
    pub async fn accept_transfer(&self, state: &mut AppState, transfer_id: usize) -> Result<()> {
        let transfer = state
            .transfers
            .iter_mut()
            .find(|t| t.id == transfer_id)
            .ok_or_else(|| anyhow::anyhow!("Transfer {} not found. Use /dcc list to see available transfers.", transfer_id))?;

        if transfer.status != DccTransferStatus::Pending {
            return Err(anyhow::anyhow!("Transfer {} is not pending (status: {:?})", transfer_id, transfer.status));
        }

        let download_dir = &state.config.dcc.download_dir;
        let path = security::safe_download_path(download_dir, &transfer.filename)
            .ok_or_else(|| anyhow::anyhow!("Could not create safe download path for '{}'", transfer.filename))?;

        transfer.status = DccTransferStatus::Active;

        let ip = transfer.ip;
        let port = transfer.port;
        let size = transfer.size;
        let filename = transfer.filename.clone();
        let server_id = transfer.server_id;

        let key = state
            .active_buffer
            .clone()
            .unwrap_or(crate::app::state::BufferKey::ServerStatus(server_id));
        state.system_message(
            &key,
            format!("DCC: downloading \"{}\" ({} bytes) to {}", filename, size, path.display()),
        );

        transfer::spawn_receive(transfer_id, ip, port, size, path, self.event_tx.clone()).await?;
        Ok(())
    }

    /// Cancel a transfer by marking it as [`DccTransferStatus::Cancelled`].
    pub fn cancel_transfer(&self, state: &mut AppState, transfer_id: usize) -> Result<()> {
        let transfer = state
            .transfers
            .iter_mut()
            .find(|t| t.id == transfer_id)
            .ok_or_else(|| anyhow::anyhow!("Transfer {} not found", transfer_id))?;

        transfer.status = DccTransferStatus::Cancelled;
        Ok(())
    }
}
