use crate::config::AppConfig;
use crate::plugin::EventData;
use crate::plugin::PluginManager;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::RwLock;

use serde::Serialize;

use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize)]
pub struct MonitorStatus {
    pub plugin: String,
    pub network: String,
    pub channel: String,
    pub status: String,
}

pub struct IrcMonitor {
    config: Arc<RwLock<AppConfig>>,
    plugin_manager: Arc<PluginManager>,
    pub active_monitors: Arc<RwLock<Vec<MonitorStatus>>>,
    pub raw_logs: Arc<RwLock<VecDeque<String>>>,
}

impl IrcMonitor {
    pub fn new(config: Arc<RwLock<AppConfig>>, plugin_manager: Arc<PluginManager>) -> Self {
        Self {
            config,
            plugin_manager,
            active_monitors: Arc::new(RwLock::new(Vec::new())),
            raw_logs: Arc::new(RwLock::new(VecDeque::with_capacity(500))),
        }
    }

    pub fn start_monitoring(&self, plugin_name: String, network_name: String, channel: String) {
        let config = self.config.clone();
        let plugin_manager = self.plugin_manager.clone();
        let active_monitors = self.active_monitors.clone();
        let raw_logs = self.raw_logs.clone();

        tokio::spawn(async move {
            tracing::info!(
                "[{}] Starting persistent IRC monitor for {} on {}",
                plugin_name,
                channel,
                network_name
            );

            // Register monitor
            {
                let mut monitors = active_monitors.write().await;
                monitors.push(MonitorStatus {
                    plugin: plugin_name.clone(),
                    network: network_name.clone(),
                    channel: channel.clone(),
                    status: "Connecting...".to_string(),
                });
            }

            let update_status = |status: &str| {
                let net = network_name.clone();
                let chan = channel.clone();
                let stat = status.to_string();
                let monitors = active_monitors.clone();
                tokio::spawn(async move {
                    let mut lock = monitors.write().await;
                    if let Some(m) = lock
                        .iter_mut()
                        .find(|m| m.network == net && m.channel == chan)
                    {
                        m.status = stat;
                    }
                });
            };

            loop {
                // 1. Resolve network
                let cfg = config.read().await;
                let (host, port, ssl, _autojoin, _delay) = cfg.resolve_network(&network_name);
                let nickname = cfg.nickname.clone();
                let username = cfg.username.clone();
                let realname = cfg.realname.clone();
                drop(cfg);

                let server = format!("{}:{}", host, port);
                tracing::info!("Monitor connecting to {}", server);

                // 2. Connect
                let connect_res = match tokio::time::timeout(
                    Duration::from_secs(15),
                    TcpStream::connect(&server),
                )
                .await
                {
                    Ok(res) => res,
                    Err(_) => {
                        tracing::error!("Monitor connection timeout to {}", server);
                        update_status("Connection Timeout. Retrying...");
                        tokio::time::sleep(Duration::from_secs(15)).await;
                        continue;
                    }
                };

                let tcp_stream = match connect_res {
                    Ok(s) => s,
                    Err(e) => {
                        tracing::error!("Monitor connection error to {}: {}", server, e);
                        update_status("Connection Error. Retrying...");
                        tokio::time::sleep(Duration::from_secs(15)).await;
                        continue;
                    }
                };

                // 3. Setup TLS or Plain
                let (reader, mut writer): (
                    Box<dyn tokio::io::AsyncBufRead + Unpin + Send>,
                    Box<dyn tokio::io::AsyncWrite + Unpin + Send>,
                ) = if ssl {
                    let connector = native_tls::TlsConnector::builder().build().unwrap();
                    let connector = tokio_native_tls::TlsConnector::from(connector);
                    match connector.connect(&host, tcp_stream).await {
                        Ok(tls_stream) => {
                            let (r, w) = tokio::io::split(tls_stream);
                            (Box::new(BufReader::new(r)), Box::new(w))
                        }
                        Err(e) => {
                            tracing::error!("Monitor TLS error on {}: {}", server, e);
                            update_status("TLS Error. Retrying...");
                            tokio::time::sleep(Duration::from_secs(15)).await;
                            continue;
                        }
                    }
                } else {
                    let (r, w) = tokio::io::split(tcp_stream);
                    (Box::new(BufReader::new(r)), Box::new(w))
                };

                let mut reader = reader;

                // 4. Login
                let mut current_nick = nickname.clone();
                let _ = writer
                    .write_all(format!("NICK {}\r\n", current_nick).as_bytes())
                    .await;
                let _ = writer
                    .write_all(format!("USER {} 0 * :{}\r\n", username, realname).as_bytes())
                    .await;

                // 5. Read loop
                let mut buf = Vec::new();
                let mut joined = false;

                loop {
                    buf.clear();
                    let read_res = tokio::time::timeout(
                        Duration::from_secs(240),
                        reader.read_until(b'\n', &mut buf),
                    )
                    .await;

                    match read_res {
                        Ok(Ok(0)) => {
                            tracing::warn!("Monitor disconnected from {}", server);
                            update_status("Disconnected. Retrying...");
                            break; // Reconnect
                        }
                        Ok(Ok(_)) => {
                            let line = String::from_utf8_lossy(&buf);
                            let line = line.trim();

                            if line.starts_with("PING") {
                                let pong = line.replace("PING", "PONG");
                                let _ = writer.write_all(format!("{}\r\n", pong).as_bytes()).await;
                                continue;
                            }

                            if line.contains(" 433 ") {
                                current_nick.push('_');
                                let _ = writer
                                    .write_all(format!("NICK {}\r\n", current_nick).as_bytes())
                                    .await;
                                continue;
                            }

                            if line.contains(" 001 ") && !joined {
                                tracing::info!(
                                    "Monitor joined server {}, joining {}",
                                    server,
                                    channel
                                );
                                let _ = writer
                                    .write_all(format!("JOIN {}\r\n", channel).as_bytes())
                                    .await;
                                joined = true;
                                update_status("Connected");
                                continue;
                            }

                            // Parse PRIVMSG and NOTICE for plugins
                            if line.contains("PRIVMSG") || line.contains("NOTICE") {
                                if let Some((nick, cmd, target, msg)) =
                                    Self::parse_irc_message(line)
                                {
                                    // Log to raw_logs buffer
                                    {
                                        let timestamp = chrono::Local::now()
                                            .format("%Y-%m-%d %H:%M:%S")
                                            .to_string();
                                        let mut logs = raw_logs.write().await;
                                        logs.push_back(format!(
                                            "[{}] [{}] <{}> {}",
                                            timestamp, network_name, nick, msg
                                        ));
                                        if logs.len() > 500 {
                                            logs.pop_front();
                                        }
                                    }

                                    if cmd == "PRIVMSG" && !msg.starts_with("\x01") {
                                        plugin_manager.emit_signal(
                                            "irc_message",
                                            EventData::Tuple4(
                                                network_name.clone(),
                                                target,
                                                nick,
                                                msg,
                                            ),
                                        );
                                    } else if cmd == "NOTICE" {
                                        plugin_manager.emit_signal(
                                            "irc_notice",
                                            EventData::Tuple2(nick, msg),
                                        );
                                    }
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            tracing::error!("Monitor read error from {}: {}", server, e);
                            update_status("Read Error. Retrying...");
                            break; // Reconnect
                        }
                        Err(_) => {
                            // Timeout (no ping for 4 mins)
                            tracing::warn!("Monitor timed out from {}", server);
                            update_status("Ping Timeout. Retrying...");
                            break; // Reconnect
                        }
                    }
                }

                tracing::info!(
                    "Reconnecting monitor for {} on {} in 15s...",
                    channel,
                    network_name
                );
                tokio::time::sleep(Duration::from_secs(15)).await;
            }
        });
    }

    /// Parse a generic IRC message
    /// Format: :nick!user@host CMD target :message
    fn parse_irc_message(line: &str) -> Option<(String, String, String, String)> {
        if !line.starts_with(':') {
            return None;
        }

        let space1 = line.find(' ')?;
        let prefix = &line[1..space1];

        let nick = if let Some(bang) = prefix.find('!') {
            prefix[..bang].to_string()
        } else {
            prefix.to_string()
        };

        let rest = &line[space1 + 1..];
        let space2 = rest.find(' ')?;
        let cmd = rest[..space2].to_string();

        let rest2 = &rest[space2 + 1..];

        let (target, msg) = if let Some(colon) = rest2.find(" :") {
            (rest2[..colon].to_string(), rest2[colon + 2..].to_string())
        } else {
            // No message part?
            let space3 = rest2.find(' ').unwrap_or(rest2.len());
            (rest2[..space3].to_string(), String::new())
        };

        Some((nick, cmd, target, msg))
    }
}
