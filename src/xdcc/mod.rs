//! XDCC IRC Bot Download Module
//!
//! This module provides XDCC download support including:
//! - IRC URL parsing (irc://network/channel/bot/slot)
//! - Search providers (SunXDCC, SkullXDCC, etc.)
//! - IRC client connection and channel management
//! - DCC file transfer with progress tracking

mod client;
mod search;
mod transfer;

// Re-export public API items
pub use client::{XdccClient, XdccConfig, XdccEvent};
pub use search::SearchAggregator;
pub use transfer::{EnhancedTransferManager as TransferManager, TransferPriority};

use serde::{Deserialize, Serialize};
use std::fmt;

/// Parsed XDCC IRC URL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct XdccUrl {
    pub network: String,
    pub channel: String,
    pub bot: String,
    pub slot: i32,
}

impl XdccUrl {
    /// Parse an IRC URL in the format: irc://network/channel/bot/slot
    pub fn parse(url: &str) -> Result<Self, XdccError> {
        if !url.starts_with("irc://") {
            return Err(XdccError::InvalidUrl("URL must start with irc://".into()));
        }

        let path = url.trim_start_matches("irc://");
        let parts: Vec<&str> = path.split('/').collect();

        if parts.len() != 4 {
            return Err(XdccError::InvalidUrl(
                "URL must have format: irc://network/channel/bot/slot".into(),
            ));
        }

        let network = parts[0].to_string();
        let mut channel = parts[1].to_string();
        let bot = parts[2].to_string();
        let slot_str = parts[3];

        // Ensure channel starts with #
        if !channel.starts_with('#') {
            channel = format!("#{}", channel);
        }

        // Parse slot (may have # prefix)
        let slot_clean = slot_str.trim_start_matches('#');
        let slot = slot_clean
            .parse::<i32>()
            .map_err(|_| XdccError::InvalidUrl(format!("Invalid slot number: {}", slot_str)))?;

        Ok(Self {
            network,
            channel,
            bot,
            slot,
        })
    }

    /// Convert back to URL string
    pub fn to_url(&self) -> String {
        format!(
            "irc://{}/{}/{}/{}",
            self.network,
            self.channel.trim_start_matches('#'),
            self.bot,
            self.slot
        )
    }
}

impl fmt::Display for XdccUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_url())
    }
}

/// XDCC search result from search providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdccSearchResult {
    pub url: XdccUrl,
    #[serde(rename = "file_name")]
    pub filename: String,
    #[serde(rename = "file_size")]
    pub size: Option<u64>,
    pub size_str: String,
    pub bot: String,
    #[serde(rename = "server")]
    pub network: String,
    pub channel: String,
    #[serde(rename = "pack_number")]
    pub slot: i32,
    /// Additional metadata from the search provider
    #[serde(rename = "downloads")]
    pub gets: Option<u32>,
}

/// Transfer status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum TransferStatus {
    Pending,
    Connecting,
    Joining,
    Requesting,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

/// Active or completed XDCC transfer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct XdccTransfer {
    pub id: String,
    pub url: XdccUrl,
    pub status: TransferStatus,
    #[serde(rename = "file_name")]
    pub filename: Option<String>,
    pub size: Option<u64>,
    pub downloaded: u64,
    pub speed: f64,
    pub progress: f64,
    pub error: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

impl XdccTransfer {
    pub fn new(id: String, url: XdccUrl) -> Self {
        let now = chrono::Utc::now();
        Self {
            id,
            url,
            status: TransferStatus::Pending,
            filename: None,
            size: None,
            downloaded: 0,
            speed: 0.0,
            progress: 0.0,
            error: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// XDCC module errors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum XdccError {
    InvalidUrl(String),
    ConnectionFailed(String),
    ChannelJoinFailed(String),
    TransferFailed(String),
    SearchFailed(String),
    Timeout(String),
}

impl fmt::Display for XdccError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            XdccError::InvalidUrl(msg) => write!(f, "Invalid URL: {}", msg),
            XdccError::ConnectionFailed(msg) => write!(f, "Connection failed: {}", msg),
            XdccError::ChannelJoinFailed(msg) => write!(f, "Channel join failed: {}", msg),
            XdccError::TransferFailed(msg) => write!(f, "Transfer failed: {}", msg),
            XdccError::SearchFailed(msg) => write!(f, "Search failed: {}", msg),
            XdccError::Timeout(msg) => write!(f, "Timeout: {}", msg),
        }
    }
}

impl std::error::Error for XdccError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_xdcc_url() {
        let url = XdccUrl::parse("irc://irc.rizon.net/some-channel/TestBot/123").unwrap();
        assert_eq!(url.network, "irc.rizon.net");
        assert_eq!(url.channel, "#some-channel");
        assert_eq!(url.bot, "TestBot");
        assert_eq!(url.slot, 123);
    }

    #[test]
    fn test_parse_xdcc_url_with_hash() {
        let url = XdccUrl::parse("irc://irc.rizon.net/#test/Bot/#42").unwrap();
        assert_eq!(url.channel, "#test");
        assert_eq!(url.slot, 42);
    }

    #[test]
    fn test_invalid_url() {
        assert!(XdccUrl::parse("http://example.com").is_err());
        assert!(XdccUrl::parse("irc://network/channel").is_err());
    }

    #[test]
    fn test_url_roundtrip() {
        let url = XdccUrl::parse("irc://irc.rizon.net/test/Bot/1").unwrap();
        let str = url.to_url();
        let url2 = XdccUrl::parse(&str).unwrap();
        assert_eq!(url, url2);
    }
}
