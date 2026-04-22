//! Application Configuration Module
//!
//! Handles persistent settings for Botarr including connection, IRC, DCC, and search settings.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

/// Network-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// IRC server hostname
    pub host: String,
    /// IRC server port (default: 6667 or 6697 for SSL)
    #[serde(default = "default_port")]
    pub port: u16,
    /// Use SSL/TLS for this network
    #[serde(default = "default_true")]
    pub ssl: bool,
    /// Channels to join on connect (e.g. for idle requirements)
    #[serde(default)]
    pub autojoin_channels: Vec<String>,
    /// Seconds to wait after joining before requesting download
    #[serde(default = "default_join_delay_secs")]
    pub join_delay_secs: u64,
}

/// Complete application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // === Connection Settings ===
    /// Enable SSL/TLS by default
    #[serde(default = "default_true")]
    pub use_ssl: bool,
    /// TCP connection timeout in seconds
    #[serde(default = "default_connect_timeout")]
    pub connect_timeout: u64,
    /// General timeout for IRC operations in seconds
    #[serde(default = "default_general_timeout")]
    pub general_timeout: u64,
    /// Enable SOCKS5 proxy
    #[serde(default)]
    pub proxy_enabled: bool,
    /// SOCKS5 proxy URL (e.g., socks5://127.0.0.1:1080)
    #[serde(default)]
    pub proxy_url: String,

    // === IRC Identity ===
    /// Primary nickname
    #[serde(default = "default_nickname")]
    pub nickname: String,
    /// Username/ident
    #[serde(default = "default_username")]
    pub username: String,
    /// Real name (GECOS field)
    #[serde(default = "default_realname")]
    pub realname: String,

    // === IRC Behavior ===
    /// Maximum retry attempts per download
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
    /// Delay between retries in seconds
    #[serde(default = "default_retry_delay")]
    pub retry_delay: u64,
    /// Maximum concurrent requests to same bot
    #[serde(default = "default_queue_limit")]
    pub queue_limit: u32,

    // === DCC Settings ===
    /// Accept passive/reverse DCC connections
    #[serde(default)]
    pub passive_dcc: bool,
    /// Minimum port for passive DCC
    #[serde(default = "default_dcc_port_min")]
    pub dcc_port_min: u16,
    /// Maximum port for passive DCC
    #[serde(default = "default_dcc_port_max")]
    pub dcc_port_max: u16,
    /// Resume incomplete downloads
    #[serde(default = "default_true")]
    pub resume_enabled: bool,

    // === Search Settings ===
    /// Enabled search providers
    #[serde(default = "default_providers")]
    pub enabled_providers: Vec<String>,
    /// Results per page
    #[serde(default = "default_results_per_page")]
    pub results_per_page: u32,
    /// Search provider timeout in seconds
    #[serde(default = "default_search_timeout")]
    pub search_timeout: u64,

    // === Network Configuration ===
    /// Network name -> NetworkConfig mapping
    #[serde(default)]
    pub networks: HashMap<String, NetworkConfig>,

    // === Download Settings ===
    /// Download directory (set via env, not config file)
    #[serde(skip)]
    pub download_dir: String,
}

// Default value functions
fn default_true() -> bool {
    true
}
fn default_port() -> u16 {
    6697
}
fn default_connect_timeout() -> u64 {
    15
}
fn default_general_timeout() -> u64 {
    120
}
fn default_nickname() -> String {
    "botarr".to_string()
}
fn default_username() -> String {
    "botarr".to_string()
}
fn default_realname() -> String {
    "Botarr XDCC Client".to_string()
}
fn default_max_retries() -> u32 {
    3
}
fn default_retry_delay() -> u64 {
    30
}
fn default_queue_limit() -> u32 {
    2
}
fn default_dcc_port_min() -> u16 {
    49152
}
fn default_dcc_port_max() -> u16 {
    65535
}
fn default_providers() -> Vec<String> {
    vec![
        "SkullXDCC".to_string(),
        "XDCC.rocks".to_string(),
        "XDCC.eu".to_string(),
    ]
}
fn default_results_per_page() -> u32 {
    50
}
fn default_search_timeout() -> u64 {
    30
}
fn default_join_delay_secs() -> u64 {
    6
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            use_ssl: true,
            connect_timeout: 15,
            general_timeout: 120,
            proxy_enabled: false,
            proxy_url: String::new(),
            nickname: "botarr".to_string(),
            username: "botarr".to_string(),
            realname: "Botarr XDCC Client".to_string(),
            max_retries: 3,
            retry_delay: 30,
            queue_limit: 2,
            passive_dcc: false,
            dcc_port_min: 49152,
            dcc_port_max: 65535,
            resume_enabled: true,
            enabled_providers: vec![
                "SkullXDCC".to_string(),
                "XDCC.rocks".to_string(),
                "XDCC.eu".to_string(),
            ],
            results_per_page: 50,
            search_timeout: 30,
            networks: Self::default_networks(),
            download_dir: "./downloads".to_string(),
        }
    }
}

impl AppConfig {
    /// Load config from file, or create default if not exists
    pub fn load(path: &str) -> Self {
        match std::fs::read_to_string(path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => {
                    tracing::info!("Loaded config from {}", path);
                    config
                }
                Err(e) => {
                    tracing::warn!("Failed to parse config {}: {}, using defaults", path, e);
                    Self::default()
                }
            },
            Err(_) => {
                tracing::info!("No config file found at {}, using defaults", path);
                Self::default()
            }
        }
    }

    /// Save config to file
    pub fn save(&self, path: &str) -> Result<(), std::io::Error> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Create parent directory if needed
        if let Some(parent) = Path::new(path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::write(path, content)?;
        tracing::info!("Saved config to {}", path);
        Ok(())
    }

    /// Get default network configurations
    fn default_networks() -> HashMap<String, NetworkConfig> {
        let mut networks = HashMap::new();

        // Networks that require SSL
        networks.insert(
            "SceneP2P".to_string(),
            NetworkConfig {
                host: "irc.scenep2p.net".to_string(),
                port: 6697,
                ssl: true,
                autojoin_channels: Vec::new(),
                join_delay_secs: 6,
            },
        );

        // Networks that work without SSL
        networks.insert(
            "Rizon".to_string(),
            NetworkConfig {
                host: "irc.rizon.net".to_string(),
                port: 6667,
                ssl: false,
                autojoin_channels: Vec::new(),
                join_delay_secs: 6,
            },
        );

        networks.insert(
            "Abjects".to_string(),
            NetworkConfig {
                host: "irc.abjects.net".to_string(),
                port: 6667,
                ssl: false,
                autojoin_channels: Vec::new(),
                join_delay_secs: 6,
            },
        );

        networks
    }

    /// Resolve network name to connection details
    pub fn resolve_network(&self, network: &str) -> (String, u16, bool, Vec<String>, u64) {
        // Check explicit mapping (case-insensitive)
        for (key, config) in &self.networks {
            if key.eq_ignore_ascii_case(network) {
                return (
                    config.host.clone(),
                    config.port,
                    config.ssl,
                    config.autojoin_channels.clone(),
                    config.join_delay_secs,
                );
            }
        }

        // If it looks like a hostname (contains a dot), use as-is
        if network.contains('.') {
            let port = if self.use_ssl { 6697 } else { 6667 };
            return (network.to_string(), port, self.use_ssl, Vec::new(), 6);
        }

        let host = format!("irc.{}.net", network.to_lowercase());
        let port = if self.use_ssl { 6697 } else { 6667 };
        (host, port, self.use_ssl, Vec::new(), 6)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default_values() {
        let config = AppConfig::default();
        assert!(config.use_ssl);
        assert_eq!(config.connect_timeout, 15);
        assert_eq!(config.general_timeout, 120);
        assert_eq!(config.nickname, "botarr");
        assert_eq!(config.max_retries, 3);
        assert!(config.resume_enabled);
    }

    #[test]
    fn test_network_resolution_explicit() {
        let config = AppConfig::default();
        let (host, port, ssl, _, _) = config.resolve_network("SceneP2P");
        assert_eq!(host, "irc.scenep2p.net");
        assert_eq!(port, 6697);
        assert!(ssl);
    }

    #[test]
    fn test_network_resolution_hostname() {
        let config = AppConfig::default();
        let (host, port, ssl, _, _) = config.resolve_network("irc.example.com");
        assert_eq!(host, "irc.example.com");
        assert_eq!(port, 6697); // Default SSL port
        assert!(ssl);
    }

    #[test]
    fn test_network_resolution_heuristic() {
        let config = AppConfig::default();
        let (host, _port, _ssl, _, _) = config.resolve_network("UnknownNet");
        assert_eq!(host, "irc.unknownnet.net");
    }

    #[test]
    fn test_config_serialization() {
        let config = AppConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let loaded: AppConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.nickname, loaded.nickname);
        assert_eq!(config.use_ssl, loaded.use_ssl);
    }

    #[test]
    fn test_port_range_validation() {
        let config = AppConfig::default();
        assert!(config.dcc_port_min < config.dcc_port_max);
        assert!(config.dcc_port_min >= 1024); // Above privileged ports
    }

    #[test]
    fn test_timeout_bounds() {
        let config = AppConfig::default();
        assert!(config.connect_timeout > 0);
        assert!(config.connect_timeout <= 60);
        assert!(config.general_timeout >= config.connect_timeout);
    }
}
