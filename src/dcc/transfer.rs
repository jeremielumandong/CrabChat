use crate::app::event::{AppEvent, TransferId};
use anyhow::Result;
use std::net::IpAddr;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

/// Spawn a DCC file receive task
pub async fn spawn_receive(
    transfer_id: TransferId,
    ip: IpAddr,
    port: u16,
    size: u64,
    download_path: PathBuf,
    event_tx: mpsc::UnboundedSender<AppEvent>,
) -> Result<()> {
    tokio::spawn(async move {
        if let Err(e) = receive_file(transfer_id, ip, port, size, &download_path, &event_tx).await {
            let _ = event_tx.send(AppEvent::DccFailed {
                transfer_id,
                error: e.to_string(),
            });
        }
    });
    Ok(())
}

async fn receive_file(
    transfer_id: TransferId,
    ip: IpAddr,
    port: u16,
    total_size: u64,
    path: &PathBuf,
    event_tx: &mpsc::UnboundedSender<AppEvent>,
) -> Result<()> {
    use tokio::io::AsyncReadExt;

    // Create parent directory if needed
    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    let mut stream = TcpStream::connect((ip, port)).await?;
    let mut file = tokio::fs::File::create(path).await?;

    let mut received: u64 = 0;
    let mut buf = [0u8; 8192];
    let mut last_progress = std::time::Instant::now();

    loop {
        let n = stream.read(&mut buf).await?;
        if n == 0 {
            break;
        }

        file.write_all(&buf[..n]).await?;
        received += n as u64;

        // Send acknowledgment (DCC protocol: send 4 bytes of total received, big-endian)
        let ack = (received as u32).to_be_bytes();
        // Ignore write errors for ack — some implementations don't expect it
        let _ = stream.write_all(&ack).await;

        // Report progress at most every 250ms
        if last_progress.elapsed() >= std::time::Duration::from_millis(250) {
            let _ = event_tx.send(AppEvent::DccProgress {
                transfer_id,
                bytes_received: received,
                total: total_size,
            });
            last_progress = std::time::Instant::now();
        }

        if total_size > 0 && received >= total_size {
            break;
        }
    }

    file.flush().await?;

    let _ = event_tx.send(AppEvent::DccComplete { transfer_id });
    Ok(())
}
