use std::io::SeekFrom;
use std::time::Duration;
use tokio::fs::OpenOptions;
use tokio::io::AsyncSeekExt;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

use crate::xdcc::{XdccError, XdccEvent};

#[derive(Debug)]
pub struct DccInfo {
    pub filename: String,
    pub ip: String,
    pub port: u16,
    pub size: u64,
}

pub struct DccResumeInfo {
    pub dcc_info: DccInfo,
    pub offset: u64,
}

/// Parse DCC SEND message
/// Format: :bot!... PRIVMSG nick :\x01DCC SEND filename ip port size\x01
pub fn parse_dcc_send(line: &str) -> Option<DccInfo> {
    let dcc_start = line.find("DCC SEND")?;
    let dcc_part = &line[dcc_start..];

    // Remove CTCP markers
    let cleaned = dcc_part
        .trim_start_matches("DCC SEND")
        .trim()
        .trim_end_matches('\x01')
        .trim();

    // Handle quoted filenames
    let (filename, rest) = if let Some(stripped) = cleaned.strip_prefix('"') {
        let end_quote = stripped.find('"')? + 1;
        let name = stripped[..end_quote - 1].to_string();
        (name, stripped[end_quote..].trim())
    } else {
        let parts: Vec<&str> = cleaned.splitn(2, ' ').collect();
        if parts.len() < 2 {
            return None;
        }
        (parts[0].to_string(), parts[1])
    };

    let parts: Vec<&str> = rest.split_whitespace().collect();
    if parts.len() < 3 {
        return None;
    }

    let ip_int: u32 = parts[0].parse().ok()?;
    let port: u16 = parts[1].parse().ok()?;
    let size: u64 = parts[2].parse().ok()?;

    // Convert IP from integer to dotted format
    let ip = format!(
        "{}.{}.{}.{}",
        (ip_int >> 24) & 0xFF,
        (ip_int >> 16) & 0xFF,
        (ip_int >> 8) & 0xFF,
        ip_int & 0xFF
    );

    Some(DccInfo {
        filename,
        ip,
        port,
        size,
    })
}

pub async fn dcc_receive(
    info: DccInfo,
    download_dir: &str,
    seek_offset: u64,
    tx: mpsc::Sender<XdccEvent>,
) -> Result<(), XdccError> {
    let addr = format!("{}:{}", info.ip, info.port);
    tracing::info!("Connecting to DCC: {} for file: {}", addr, info.filename);

    let mut stream = timeout(Duration::from_secs(30), TcpStream::connect(&addr))
        .await
        .map_err(|_| XdccError::TransferFailed("DCC connection timed out".into()))?
        .map_err(|e| XdccError::TransferFailed(format!("DCC connection failed: {}", e)))?;

    // Create download directory if needed
    tokio::fs::create_dir_all(download_dir).await.ok();

    // Sanitize filename
    let safe_filename = info
        .filename
        .replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
    let file_path = format!("{}/{}", download_dir, safe_filename);

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(seek_offset == 0) // Only truncate if starting fresh
        .open(&file_path)
        .await
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::PermissionDenied
            | std::io::ErrorKind::NotFound
            | std::io::ErrorKind::StorageFull
            | std::io::ErrorKind::WriteZero => {
                XdccError::FatalIo(format!("Failed to create/open file: {}", e))
            }
            _ => XdccError::TransferFailed(format!("Failed to create/open file: {}", e)),
        })?;

    if seek_offset > 0 {
        tracing::info!("Resuming file at offset {}", seek_offset);
        if let Err(e) = file.seek(SeekFrom::Start(seek_offset)).await {
            return Err(XdccError::TransferFailed(format!(
                "Failed to seek file: {}",
                e
            )));
        }
    }

    tracing::info!("Saving to: {}", file_path);

    let mut downloaded: u64 = seek_offset;
    let mut buf = [0u8; 16384];
    let mut last_update = std::time::Instant::now();
    let mut bytes_since_update: u64 = 0;
    let start_time = std::time::Instant::now();
    let mut last_log_update = std::time::Instant::now(); // Added for log throttling

    loop {
        match stream.read(&mut buf).await {
            Ok(0) => break,
            Ok(n) => {
                file.write_all(&buf[..n])
                    .await
                    .map_err(|e| match e.kind() {
                        std::io::ErrorKind::StorageFull
                        | std::io::ErrorKind::WriteZero
                        | std::io::ErrorKind::PermissionDenied => {
                            XdccError::FatalIo(format!("Write error: {}", e))
                        }
                        _ => XdccError::TransferFailed(format!("Write error: {}", e)),
                    })?;
                downloaded += n as u64;
                bytes_since_update += n as u64;

                // Send DCC acknowledgment (required by protocol)
                let ack = (downloaded as u32).to_be_bytes();
                let _ = stream.write_all(&ack).await;

                // Send progress update every 500ms (for UI)
                let elapsed = last_update.elapsed();
                if elapsed.as_millis() >= 500 {
                    let speed = bytes_since_update as f64 / elapsed.as_secs_f64();
                    let _ = tx
                        .send(XdccEvent::Progress {
                            downloaded,
                            total: info.size,
                            speed,
                        })
                        .await;
                    last_update = std::time::Instant::now();
                    bytes_since_update = 0;

                    // Log progress only every 5 seconds to reduce verbosity
                    if last_log_update.elapsed().as_secs() >= 5 {
                        let percent = if info.size > 0 {
                            (downloaded as f64 / info.size as f64) * 100.0
                        } else {
                            0.0
                        };
                        tracing::debug!(
                            "Progress: {:.1}% ({} / {} bytes) @ {:.1} KB/s",
                            percent,
                            downloaded,
                            info.size,
                            speed / 1024.0
                        );
                        last_log_update = std::time::Instant::now();
                    }
                }
            }
            Err(e) => {
                return Err(XdccError::TransferFailed(format!("Read error: {}", e)));
            }
        }
    }

    // Final progress update
    let total_time = start_time.elapsed().as_secs_f64();
    let avg_speed = if total_time > 0.0 {
        downloaded as f64 / total_time
    } else {
        0.0
    };
    let _ = tx
        .send(XdccEvent::Progress {
            downloaded,
            total: info.size,
            speed: avg_speed,
        })
        .await;

    tracing::info!(
        "DCC transfer complete: {} bytes in {:.1}s ({:.1} KB/s)",
        downloaded,
        total_time,
        avg_speed / 1024.0
    );

    Ok(())
}
