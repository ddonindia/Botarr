use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::{broadcast, mpsc, RwLock};

#[derive(serde::Serialize, Clone, Debug)]
pub struct WsMessage {
    pub r#type: String, // "message", "error", "status"
    pub network: String,
    pub target: Option<String>, // channel or nick
    pub message: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct WsCommand {
    pub action: String, // "connect", "send", "disconnect"
    pub network: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub ssl: Option<bool>,
    pub nick: Option<String>,
    pub message: Option<String>, // raw IRC command to send (e.g. "JOIN #channel", "PRIVMSG #channel :hello")
}

pub struct IrcClientConnection {
    pub tx: mpsc::Sender<String>,
}

#[derive(Clone)]
pub struct InteractiveClientManager {
    connections: Arc<RwLock<HashMap<String, IrcClientConnection>>>,
    pub ws_tx: broadcast::Sender<WsMessage>,
}

impl InteractiveClientManager {
    pub fn new() -> Self {
        let (ws_tx, _) = broadcast::channel(1024);
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            ws_tx,
        }
    }

    pub async fn handle_command(&self, cmd: WsCommand) {
        match cmd.action.as_str() {
            "connect" => {
                if let (Some(network), Some(host), Some(port), Some(ssl), Some(nick)) =
                    (cmd.network, cmd.host, cmd.port, cmd.ssl, cmd.nick)
                {
                    self.connect(network, host, port, ssl, nick).await;
                }
            }
            "send" => {
                if let (Some(network), Some(message)) = (cmd.network, cmd.message) {
                    let conns = self.connections.read().await;
                    if let Some(conn) = conns.get(&network) {
                        let _ = conn.tx.send(format!("{}\r\n", message)).await;
                    }
                }
            }
            "disconnect" => {
                if let Some(network) = cmd.network {
                    let mut conns = self.connections.write().await;
                    if let Some(conn) = conns.remove(&network) {
                        let _ = conn.tx.send("QUIT :Leaving\r\n".to_string()).await;
                    }
                }
            }
            _ => {}
        }
    }

    async fn connect(&self, network: String, host: String, port: u16, ssl: bool, nick: String) {
        let mut conns = self.connections.write().await;
        if conns.contains_key(&network) {
            let _ = self.ws_tx.send(WsMessage {
                r#type: "error".to_string(),
                network: network.clone(),
                target: None,
                message: "Already connected to this network".to_string(),
            });
            return;
        }

        let (tx, mut rx) = mpsc::channel::<String>(100);
        conns.insert(network.clone(), IrcClientConnection { tx });

        let ws_tx = self.ws_tx.clone();
        let connections = self.connections.clone();

        tokio::spawn(async move {
            let server_addr = format!("{}:{}", host, port);

            let _ = ws_tx.send(WsMessage {
                r#type: "status".to_string(),
                network: network.clone(),
                target: None,
                message: format!("Connecting to {}...", server_addr),
            });

            let tcp_stream = match tokio::time::timeout(
                Duration::from_secs(15),
                TcpStream::connect(&server_addr),
            )
            .await
            {
                Ok(Ok(s)) => s,
                _ => {
                    let _ = ws_tx.send(WsMessage {
                        r#type: "error".to_string(),
                        network: network.clone(),
                        target: None,
                        message: "Connection failed or timed out".to_string(),
                    });
                    connections.write().await.remove(&network);
                    return;
                }
            };

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
                        let _ = ws_tx.send(WsMessage {
                            r#type: "error".to_string(),
                            network: network.clone(),
                            target: None,
                            message: format!("TLS Error: {}", e),
                        });
                        connections.write().await.remove(&network);
                        return;
                    }
                }
            } else {
                let (r, w) = tokio::io::split(tcp_stream);
                (Box::new(BufReader::new(r)), Box::new(w))
            };

            let mut reader = reader;

            // Login
            let _ = writer
                .write_all(format!("NICK {}\r\n", nick).as_bytes())
                .await;
            let _ = writer
                .write_all(format!("USER {} 0 * :Botarr Web Client\r\n", nick).as_bytes())
                .await;

            let _ = ws_tx.send(WsMessage {
                r#type: "status".to_string(),
                network: network.clone(),
                target: None,
                message: "Connected. Registering...".to_string(),
            });

            let mut buf = Vec::new();

            loop {
                tokio::select! {
                    read_res = reader.read_until(b'\n', &mut buf) => {
                        match read_res {
                            Ok(0) => break, // EOF
                            Ok(_) => {
                                let line = String::from_utf8_lossy(&buf);
                                let line = line.trim();

                                if line.starts_with("PING") {
                                    let pong = line.replace("PING", "PONG");
                                    let _ = writer.write_all(format!("{}\r\n", pong).as_bytes()).await;
                                } else {
                                    // Parse for target (channel or nick)
                                    let mut target = None;
                                    if line.contains(" PRIVMSG ") || line.contains(" NOTICE ") || line.contains(" JOIN ") || line.contains(" PART ") {
                                        // Simple parse to find the target buffer
                                        let parts: Vec<&str> = line.splitn(4, ' ').collect();
                                        if parts.len() >= 3 {
                                            target = Some(parts[2].to_string().replace(":", ""));
                                        }
                                    }

                                    let _ = ws_tx.send(WsMessage {
                                        r#type: "message".to_string(),
                                        network: network.clone(),
                                        target,
                                        message: line.to_string(),
                                    });
                                }
                                buf.clear();
                            }
                            Err(_) => break, // Error
                        }
                    }
                    Some(msg) = rx.recv() => {
                        if writer.write_all(msg.as_bytes()).await.is_err() {
                            break;
                        }
                    }
                }
            }

            let _ = ws_tx.send(WsMessage {
                r#type: "status".to_string(),
                network: network.clone(),
                target: None,
                message: "Disconnected.".to_string(),
            });
            connections.write().await.remove(&network);
        });
    }
}
