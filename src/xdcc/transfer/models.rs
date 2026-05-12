use crate::xdcc::XdccTransfer;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Transfer priority levels
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "lowercase")]
pub enum TransferPriority {
    Low = 0,
    #[default]
    Normal = 1,
    High = 2,
    Urgent = 3,
}

/// Bot reliability statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BotStats {
    pub bot_name: String,
    pub network: String,
    pub total_downloads: u32,
    pub successful_downloads: u32,
    pub failed_downloads: u32,
    pub total_bytes: u64,
    pub average_speed: f64,
    pub last_seen: DateTime<Utc>,
    pub reliability_score: f64, // 0.0 to 1.0
}

impl BotStats {
    pub fn new(bot_name: String, network: String) -> Self {
        Self {
            bot_name,
            network,
            total_downloads: 0,
            successful_downloads: 0,
            failed_downloads: 0,
            total_bytes: 0,
            average_speed: 0.0,
            last_seen: Utc::now(),
            reliability_score: 0.5, // Start with neutral score
        }
    }

    pub fn record_success(&mut self, bytes: u64, speed: f64) {
        self.total_downloads += 1;
        self.successful_downloads += 1;
        self.total_bytes += bytes;

        // Update average speed (exponential moving average)
        if self.average_speed == 0.0 {
            self.average_speed = speed;
        } else {
            self.average_speed = self.average_speed * 0.7 + speed * 0.3;
        }

        self.last_seen = Utc::now();
        self.update_reliability_score();
    }

    pub fn record_failure(&mut self) {
        self.total_downloads += 1;
        self.failed_downloads += 1;
        self.last_seen = Utc::now();
        self.update_reliability_score();
    }

    fn update_reliability_score(&mut self) {
        if self.total_downloads == 0 {
            self.reliability_score = 0.5;
            return;
        }

        let success_rate = self.successful_downloads as f64 / self.total_downloads as f64;

        // Factor in recency - older stats matter less
        let days_since_last = (Utc::now() - self.last_seen).num_days() as f64;
        let recency_factor = (-days_since_last / 30.0).exp(); // Decay over 30 days

        // Combine success rate with recency
        self.reliability_score = success_rate * 0.7 + recency_factor * 0.3;
    }
}

/// Enhanced transfer with queue management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTransfer {
    #[serde(flatten)]
    pub transfer: XdccTransfer,
    pub priority: TransferPriority,
    pub retry_count: u32,
    pub max_retries: u32,
    pub queue_position: Option<usize>,
}

impl EnhancedTransfer {
    pub fn new(transfer: XdccTransfer) -> Self {
        Self {
            transfer,
            priority: TransferPriority::Normal,
            retry_count: 0,
            max_retries: 3,
            queue_position: None,
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }
}

/// Download analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DownloadAnalytics {
    pub total_downloads: u64,
    pub successful_downloads: u64,
    pub failed_downloads: u64,
    pub total_bytes_downloaded: u64,
    pub average_download_speed: f64,
    pub total_download_time_seconds: u64,
    pub most_active_network: Option<String>,
    pub most_reliable_bot: Option<String>,
}

impl Default for DownloadAnalytics {
    fn default() -> Self {
        Self {
            total_downloads: 0,
            successful_downloads: 0,
            failed_downloads: 0,
            total_bytes_downloaded: 0,
            average_download_speed: 0.0,
            total_download_time_seconds: 0,
            most_active_network: None,
            most_reliable_bot: None,
        }
    }
}
