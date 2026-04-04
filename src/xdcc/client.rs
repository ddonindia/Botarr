//! XDCC IRC Client
//!
//! Handles IRC connection, channel joining, and XDCC transfer requests.

use super::{XdccError, XdccUrl};
use std::collections::HashMap;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio::time::timeout;
use tokio_native_tls::TlsStream;

/// Events emitted during XDCC transfer
#[derive(Debug, Clone)]
pub enum XdccEvent {
    Connecting,
    Connected,
    Joining(String),
    Joined(String),
    Requesting(String, i32),
    DccSend {
        filename: String,
        ip: String,
        port: u16,
        size: u64,
    },
    Progress {
        downloaded: u64,
        total: u64,
        speed: f64,
    },
    Completed,
    Error(String),
}

/// Configuration for XDCC client
#[derive(Debug, Clone)]
pub struct XdccConfig {
    /// Nickname to use on IRC
    pub nickname: String,
    /// Username/ident
    pub username: String,
    /// Real name (GECOS)
    pub realname: String,
    /// Enable SSL/TLS
    pub use_ssl: bool,
    /// Connection timeout in seconds (for initial TCP connect)
    pub connect_timeout_secs: u64,
    /// General timeout in seconds (for IRC handshake, DCC response, etc.)
    pub timeout_secs: u64,
    /// Download directory
    pub download_dir: String,
    /// Network name to (host, port, ssl) mapping
    pub networks: HashMap<String, (String, u16, bool)>,
    /// Enable SOCKS5 proxy
    pub proxy_enabled: bool,
    /// SOCKS5 proxy URL (e.g., socks5://127.0.0.1:1080)
    pub proxy_url: String,
}

impl Default for XdccConfig {
    fn default() -> Self {
        use std::time::{SystemTime, UNIX_EPOCH};
        let random_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| (d.as_nanos() % 10000) as u16)
            .unwrap_or(1234);

        Self {
            nickname: format!("botarr{}", random_suffix),
            username: "botarr".to_string(),
            realname: "Botarr XDCC Client".to_string(),
            use_ssl: true, // Default to SSL
            connect_timeout_secs: 15,
            timeout_secs: 120,
            download_dir: "./downloads".to_string(),
            networks: HashMap::new(),
            proxy_enabled: false,
            proxy_url: String::new(),
        }
    }
}

impl XdccConfig {
    /// Resolve network name to (host, port, use_ssl)
    pub fn resolve_network(&self, network: &str) -> (String, u16, bool) {
        // Check explicit mapping (case-insensitive)
        for (key, value) in &self.networks {
            if key.eq_ignore_ascii_case(network) {
                return value.clone();
            }
        }

        // If it looks like a hostname (contains a dot), use as-is
        if network.contains('.') {
            let port = if self.use_ssl { 6697 } else { 6667 };
            return (network.to_string(), port, self.use_ssl);
        }

        // Try common heuristics
        let lower = network.to_lowercase();
        let port = if self.use_ssl { 6697 } else { 6667 };
        (format!("irc.{}.net", lower), port, self.use_ssl)
    }
}

/// XDCC IRC Client for downloading files
pub struct XdccClient {
    config: XdccConfig,
}

impl XdccClient {
    pub fn new(config: XdccConfig) -> Self {
        Self { config }
    }

    /// Start an XDCC download and return an event channel
    pub async fn start_download(
        &self,
        url: XdccUrl,
    ) -> Result<mpsc::Receiver<XdccEvent>, XdccError> {
        let (tx, rx) = mpsc::channel(100);
        let config = self.config.clone();

        // Spawn the download task
        tokio::spawn(async move {
            if let Err(e) = Self::download_task(url, config, tx.clone()).await {
                tracing::error!("XDCC download failed: {}", e);
                let _ = tx.send(XdccEvent::Error(e.to_string())).await;
            }
        });

        Ok(rx)
    }

    async fn download_task(
        url: XdccUrl,
        config: XdccConfig,
        tx: mpsc::Sender<XdccEvent>,
    ) -> Result<(), XdccError> {
        let _ = tx.send(XdccEvent::Connecting).await;

        // Resolve network to (host, port, use_ssl)
        let (host, port, use_ssl) = config.resolve_network(&url.network);
        let server = format!("{}:{}", host, port);

        tracing::info!("Connecting to IRC server: {} (SSL: {})", server, use_ssl);

        // Connect with timeout (use shorter connect timeout for fast failure)
        let connect_future = async {
            if config.proxy_enabled && !config.proxy_url.is_empty() {
                // Parse proxy string "host:port" or "socks5://host:port"
                let proxy_addr = config.proxy_url.trim_start_matches("socks5://");
                tracing::info!("Connecting via SOCKS5 proxy: {} -> {}", proxy_addr, server);

                match tokio_socks::tcp::Socks5Stream::connect(proxy_addr, server.as_str()).await {
                    Ok(s) => Ok(s.into_inner()), // Unwrap to get the raw tunnelled TcpStream
                    Err(e) => Err(std::io::Error::other(e)),
                }
            } else {
                TcpStream::connect(&server).await
            }
        };

        let tcp_stream = timeout(
            Duration::from_secs(config.connect_timeout_secs),
            connect_future,
        )
        .await
        .map_err(|_| {
            XdccError::Timeout(format!(
                "Connection to {} timed out after {}s",
                server, config.connect_timeout_secs
            ))
        })?
        .map_err(|e| XdccError::ConnectionFailed(format!("Connection failed: {}", e)))?;

        tracing::info!("TCP connected to {}", server);

        // Perform TLS handshake if SSL is enabled
        if use_ssl {
            tracing::info!("Performing TLS handshake...");
            let connector = native_tls::TlsConnector::builder()
                .danger_accept_invalid_certs(true) // Some IRC servers have self-signed certs
                .build()
                .map_err(|e| XdccError::ConnectionFailed(format!("TLS setup failed: {}", e)))?;

            let connector = tokio_native_tls::TlsConnector::from(connector);
            let tls_stream = connector
                .connect(&host, tcp_stream)
                .await
                .map_err(|e| XdccError::ConnectionFailed(format!("TLS handshake failed: {}", e)))?;

            tracing::info!("TLS connection established to {}", server);
            let _ = tx.send(XdccEvent::Connected).await;

            // Run IRC session over TLS
            Self::irc_session_tls(tls_stream, url, config, tx).await
        } else {
            let _ = tx.send(XdccEvent::Connected).await;
            // Run IRC session over plain TCP
            Self::irc_session_plain(tcp_stream, url, config, tx).await
        }
    }

    /// IRC session over plain TCP
    async fn irc_session_plain(
        stream: TcpStream,
        url: XdccUrl,
        config: XdccConfig,
        tx: mpsc::Sender<XdccEvent>,
    ) -> Result<(), XdccError> {
        let (reader, writer) = stream.into_split();
        let reader = BufReader::new(reader);
        Self::irc_session_inner(reader, writer, url, config, tx).await
    }

    /// IRC session over TLS
    async fn irc_session_tls(
        stream: TlsStream<TcpStream>,
        url: XdccUrl,
        config: XdccConfig,
        tx: mpsc::Sender<XdccEvent>,
    ) -> Result<(), XdccError> {
        let (reader, writer) = tokio::io::split(stream);
        let reader = BufReader::new(reader);
        Self::irc_session_inner(reader, writer, url, config, tx).await
    }

    /// Core IRC session logic (works with any AsyncRead/AsyncWrite)
    async fn irc_session_inner<R, W>(
        mut reader: BufReader<R>,
        mut writer: W,
        url: XdccUrl,
        config: XdccConfig,
        tx: mpsc::Sender<XdccEvent>,
    ) -> Result<(), XdccError>
    where
        R: tokio::io::AsyncRead + Unpin,
        W: tokio::io::AsyncWrite + Unpin,
    {
        // Send NICK and USER commands
        let nick = &config.nickname;
        Self::send_raw(&mut writer, &format!("NICK {}", nick)).await?;
        Self::send_raw(
            &mut writer,
            &format!("USER {} 0 * :{}", config.username, config.realname),
        )
        .await?;

        let mut joined = false;
        let mut requested = false;
        let mut buf = Vec::with_capacity(1024);

        loop {
            buf.clear();

            // Read line as bytes (until \n) with timeout
            // This handles non-UTF-8 IRC data gracefully
            let read_result = timeout(
                Duration::from_secs(config.timeout_secs),
                reader.read_until(b'\n', &mut buf),
            )
            .await;

            // Convert bytes to string with lossy UTF-8 handling
            let line = String::from_utf8_lossy(&buf);

            match read_result {
                Ok(Ok(0)) => {
                    return Err(XdccError::ConnectionFailed(
                        "Connection closed by server".into(),
                    ));
                }
                Ok(Ok(_)) => {
                    let line = line.trim();
                    tracing::debug!("IRC < {}", line);

                    // Handle PING
                    if line.starts_with("PING") {
                        let pong = line.replace("PING", "PONG");
                        Self::send_raw(&mut writer, &pong).await?;
                        continue;
                    }

                    // Check for successful connection (001 numeric = RPL_WELCOME)
                    if line.contains(" 001 ") && !joined {
                        tracing::info!("Received welcome, joining channel {}", url.channel);
                        let _ = tx.send(XdccEvent::Joining(url.channel.clone())).await;
                        Self::send_raw(&mut writer, &format!("JOIN {}", url.channel)).await?;
                    }

                    // Check for successful join (366 = RPL_ENDOFNAMES)
                    if (line.contains(" 366 ") || line.contains(&format!("JOIN :{}", url.channel)))
                        && !joined
                    {
                        joined = true;
                        tracing::info!("Joined channel {}", url.channel);
                        let _ = tx.send(XdccEvent::Joined(url.channel.clone())).await;
                    }

                    // After joining, send XDCC request
                    if joined && !requested {
                        requested = true;
                        tracing::info!("Requesting pack #{} from {}", url.slot, url.bot);
                        let _ = tx
                            .send(XdccEvent::Requesting(url.bot.clone(), url.slot))
                            .await;
                        Self::send_raw(
                            &mut writer,
                            &format!("PRIVMSG {} :xdcc send #{}", url.bot, url.slot),
                        )
                        .await?;
                    }

                    // Check for DCC SEND (CTCP)
                    if line.contains("DCC SEND") {
                        if let Some(dcc_info) = Self::parse_dcc_send(line) {
                            tracing::info!(
                                "Received DCC SEND: {} from {}:{} ({} bytes)",
                                dcc_info.filename,
                                dcc_info.ip,
                                dcc_info.port,
                                dcc_info.size
                            );
                            let _ = tx
                                .send(XdccEvent::DccSend {
                                    filename: dcc_info.filename.clone(),
                                    ip: dcc_info.ip.clone(),
                                    port: dcc_info.port,
                                    size: dcc_info.size,
                                })
                                .await;

                            // Start DCC transfer
                            Self::dcc_receive(dcc_info, &config.download_dir, tx.clone()).await?;

                            // Quit IRC after transfer
                            Self::send_raw(&mut writer, "QUIT :Transfer complete").await?;
                            let _ = tx.send(XdccEvent::Completed).await;
                            return Ok(());
                        }
                    }

                    // Check for errors
                    if line.contains("No such nick")
                        || line.contains("is not online")
                        || line.contains("Invalid Pack Number")
                        || line.contains("You already requested")
                        || line.contains("Closing Link")
                    {
                        return Err(XdccError::TransferFailed(format!("Server error: {}", line)));
                    }

                    // Check for NOTICE messages from bot
                    if line.contains("NOTICE") && line.contains(&config.nickname) {
                        tracing::info!("Bot notice: {}", line);
                    }
                }
                Ok(Err(e)) => {
                    return Err(XdccError::ConnectionFailed(format!("Read error: {}", e)));
                }
                Err(_) => {
                    if !joined {
                        return Err(XdccError::Timeout(
                            "Timed out waiting to join channel".into(),
                        ));
                    }
                    if !requested {
                        continue;
                    }
                    return Err(XdccError::Timeout(
                        "Timed out waiting for DCC response from bot".into(),
                    ));
                }
            }
        }
    }

    async fn send_raw<W: tokio::io::AsyncWrite + Unpin>(
        writer: &mut W,
        msg: &str,
    ) -> Result<(), XdccError> {
        tracing::debug!("IRC > {}", msg);
        writer
            .write_all(format!("{}\r\n", msg).as_bytes())
            .await
            .map_err(|e| XdccError::ConnectionFailed(format!("Write error: {}", e)))
    }

    /// Parse DCC SEND message
    /// Format: :bot!... PRIVMSG nick :\x01DCC SEND filename ip port size\x01
    fn parse_dcc_send(line: &str) -> Option<DccInfo> {
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

    async fn dcc_receive(
        info: DccInfo,
        download_dir: &str,
        tx: mpsc::Sender<XdccEvent>,
    ) -> Result<(), XdccError> {
        use tokio::fs::File;

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

        let mut file = File::create(&file_path)
            .await
            .map_err(|e| XdccError::TransferFailed(format!("Failed to create file: {}", e)))?;

        tracing::info!("Saving to: {}", file_path);

        let mut downloaded: u64 = 0;
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
                        .map_err(|e| XdccError::TransferFailed(format!("Write error: {}", e)))?;
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
}

#[derive(Debug)]
struct DccInfo {
    filename: String,
    ip: String,
    port: u16,
    size: u64,
}
