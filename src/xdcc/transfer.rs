//! Enhanced Transfer Manager with Queue Management and Retry Logic
//!
//! Manages XDCC transfers with:
//! - Priority queue management
//! - Auto-retry failed downloads
//! - Bot reliability tracking
//! - Download history and analytics

use super::{TransferStatus, XdccTransfer, XdccUrl};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

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

/// Enhanced Transfer Manager with queue and retry support
pub struct EnhancedTransferManager {
    /// Active transfers indexed by ID
    transfers: Arc<RwLock<HashMap<String, EnhancedTransfer>>>,
    /// Cancellation tokens for each active transfer
    cancel_tokens: Arc<RwLock<HashMap<String, CancellationToken>>>,
    /// Download queue (pending transfers)
    queue: Arc<RwLock<VecDeque<String>>>,
    /// Bot reliability statistics
    bot_stats: Arc<RwLock<HashMap<String, BotStats>>>,
    /// Download history (completed/failed transfers)
    history: Arc<RwLock<Vec<XdccTransfer>>>,
    /// Analytics
    analytics: Arc<RwLock<DownloadAnalytics>>,
    /// Maximum history size
    max_history: usize,
    /// Download directory for deletion support
    download_dir: String,
}

impl EnhancedTransferManager {
    pub fn new(download_dir: String) -> Self {
        Self {
            transfers: Arc::new(RwLock::new(HashMap::new())),
            cancel_tokens: Arc::new(RwLock::new(HashMap::new())),
            queue: Arc::new(RwLock::new(VecDeque::new())),
            bot_stats: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
            analytics: Arc::new(RwLock::new(DownloadAnalytics::default())),
            max_history: 50,
            download_dir,
        }
    }

    /// Create a new transfer with priority
    pub async fn create_transfer(
        &self,
        url: XdccUrl,
        priority: TransferPriority,
    ) -> (String, CancellationToken) {
        let id = Uuid::new_v4().to_string();
        let transfer = XdccTransfer::new(id.clone(), url);
        let mut enhanced = EnhancedTransfer::new(transfer);
        enhanced.priority = priority;

        let token = CancellationToken::new();

        {
            let mut transfers = self.transfers.write().await;
            transfers.insert(id.clone(), enhanced);
        }

        {
            let mut tokens = self.cancel_tokens.write().await;
            tokens.insert(id.clone(), token.clone());
        }

        // Add to queue
        self.add_to_queue(id.clone(), priority).await;

        (id, token)
    }

    /// Add transfer to priority queue
    async fn add_to_queue(&self, id: String, _priority: TransferPriority) {
        let mut queue = self.queue.write().await;

        // Find insertion position based on priority
        let pos = queue
            .iter()
            .position(|_queue_id| {
                // This is simplified - in reality we'd look up the priority
                false // For now, just append
            })
            .unwrap_or(queue.len());

        queue.insert(pos, id.clone());

        // Update queue positions
        let mut transfers = self.transfers.write().await;
        for (idx, queue_id) in queue.iter().enumerate() {
            if let Some(transfer) = transfers.get_mut(queue_id) {
                transfer.queue_position = Some(idx + 1);
            }
        }
    }

    /// Get current queue size (Pending transfers)
    pub async fn queue_size(&self) -> usize {
        let transfers = self.transfers.read().await;
        transfers
            .values()
            .filter(|t| t.transfer.status == TransferStatus::Pending)
            .count()
    }

    /// Update transfer priority
    pub async fn set_priority(&self, id: &str, priority: TransferPriority) -> bool {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            transfer.priority = priority;
            drop(transfers);

            // Re-queue if pending
            if let Some(t) = self.get_transfer(id).await {
                if t.transfer.status == TransferStatus::Pending {
                    self.requeue_transfer(id.to_string(), priority).await;
                }
            }
            return true;
        }
        false
    }

    /// Requeue a transfer with new priority
    async fn requeue_transfer(&self, id: String, _priority: TransferPriority) {
        let mut queue = self.queue.write().await;

        // Remove from current position
        queue.retain(|queue_id| queue_id != &id);

        // Re-insert based on priority
        let pos = queue.iter().position(|_| false).unwrap_or(queue.len());
        queue.insert(pos, id);
    }

    /// Retry a failed transfer
    pub async fn retry_transfer(&self, id: &str) -> bool {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            if !transfer.can_retry() {
                return false;
            }

            transfer.retry_count += 1;
            transfer.transfer.status = TransferStatus::Pending;
            transfer.transfer.error = None;
            // Don't reset downloaded/progress - the XDCC client will use DCC RESUME
            // to continue from where the partial file left off
            transfer.transfer.speed = 0.0;
            transfer.transfer.updated_at = Utc::now();

            let priority = transfer.priority;
            let id = id.to_string();
            drop(transfers);

            // Add back to queue
            self.add_to_queue(id, priority).await;
            return true;
        }
        false
    }

    /// Record bot statistics
    pub async fn record_bot_success(&self, bot: &str, network: &str, bytes: u64, speed: f64) {
        let key = format!("{}@{}", bot, network);
        let mut stats = self.bot_stats.write().await;

        let bot_stat = stats
            .entry(key)
            .or_insert_with(|| BotStats::new(bot.to_string(), network.to_string()));

        bot_stat.record_success(bytes, speed);
    }

    pub async fn record_bot_failure(&self, bot: &str, network: &str) {
        let key = format!("{}@{}", bot, network);
        let mut stats = self.bot_stats.write().await;

        let bot_stat = stats
            .entry(key)
            .or_insert_with(|| BotStats::new(bot.to_string(), network.to_string()));

        bot_stat.record_failure();
    }

    /// Get all bot statistics sorted by reliability
    pub async fn get_all_bot_stats(&self) -> Vec<BotStats> {
        let stats = self.bot_stats.read().await;
        let mut all_stats: Vec<_> = stats.values().cloned().collect();
        all_stats.sort_by(|a, b| {
            b.reliability_score
                .partial_cmp(&a.reliability_score)
                .unwrap()
        });
        all_stats
    }

    /// Get transfer by ID
    pub async fn get_transfer(&self, id: &str) -> Option<EnhancedTransfer> {
        let transfers = self.transfers.read().await;
        transfers.get(id).cloned()
    }

    /// List all active transfers
    pub async fn list_transfers(&self) -> Vec<EnhancedTransfer> {
        let transfers = self.transfers.read().await;
        let mut list: Vec<_> = transfers.values().cloned().collect();
        list.sort_by(|a, b| b.transfer.created_at.cmp(&a.transfer.created_at));
        list
    }

    /// Get download history
    pub async fn get_history(&self, limit: usize) -> Vec<XdccTransfer> {
        let history = self.history.read().await;
        history.iter().rev().take(limit).cloned().collect()
    }

    /// Get analytics
    pub async fn get_analytics(&self) -> DownloadAnalytics {
        self.analytics.read().await.clone()
    }

    /// Update analytics on transfer completion
    async fn update_analytics(&self, transfer: &XdccTransfer, success: bool) {
        let mut analytics = self.analytics.write().await;

        analytics.total_downloads += 1;
        if success {
            analytics.successful_downloads += 1;
            if let Some(size) = transfer.size {
                analytics.total_bytes_downloaded += size;
            }

            // Update average speed
            if analytics.average_download_speed == 0.0 {
                analytics.average_download_speed = transfer.speed;
            } else {
                analytics.average_download_speed =
                    analytics.average_download_speed * 0.9 + transfer.speed * 0.1;
            }
        } else {
            analytics.failed_downloads += 1;
        }

        // Update most active network
        let bot_stats = self.bot_stats.read().await;
        if let Some(most_active) = bot_stats.values().max_by_key(|s| s.total_downloads) {
            analytics.most_active_network = Some(most_active.network.clone());
            analytics.most_reliable_bot = Some(most_active.bot_name.clone());
        }
    }

    /// Update transfer status
    pub async fn update_status(&self, id: &str, status: TransferStatus) {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            transfer.transfer.status = status.clone();
            transfer.transfer.updated_at = Utc::now();

            // Move to history if completed/failed
            if matches!(status, TransferStatus::Completed | TransferStatus::Failed) {
                let t = transfer.transfer.clone();
                drop(transfers);

                let mut history = self.history.write().await;
                history.push(t.clone());

                // Trim history
                let history_len = history.len();
                if history_len > self.max_history {
                    history.drain(0..history_len - self.max_history);
                }

                // Update analytics
                self.update_analytics(&t, status == TransferStatus::Completed)
                    .await;
            }
        }
    }

    /// Update transfer progress
    pub async fn update_progress(&self, id: &str, downloaded: u64, speed: f64) {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            transfer.transfer.downloaded = downloaded;
            transfer.transfer.speed = speed;
            if let Some(size) = transfer.transfer.size {
                if size > 0 {
                    transfer.transfer.progress = (downloaded as f64 / size as f64) * 100.0;
                }
            }
            transfer.transfer.updated_at = Utc::now();
        }
    }

    /// Set transfer file info
    pub async fn set_file_info(&self, id: &str, filename: String, size: u64) {
        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            transfer.transfer.filename = Some(filename);
            transfer.transfer.size = Some(size);
            transfer.transfer.updated_at = Utc::now();
        }
    }

    /// Mark transfer as failed with auto-retry
    /// Returns Some((url, token)) if retry should happen, so caller can spawn new download task
    pub async fn set_failed(&self, id: &str, error: String, fatal: bool) -> Option<(XdccUrl, CancellationToken)> {
        let retry_info = {
            let mut transfers = self.transfers.write().await;
            if let Some(transfer) = transfers.get_mut(id) {
                if !fatal && transfer.can_retry() {
                    // Mark for retry
                    transfer.retry_count += 1;
                    transfer.transfer.status = TransferStatus::Pending;
                    transfer.transfer.error = None;
                    transfer.transfer.speed = 0.0;
                    transfer.transfer.updated_at = Utc::now();
                    
                    // Create new cancellation token for retry
                    let new_token = CancellationToken::new();
                    let url = transfer.transfer.url.clone();
                    
                    tracing::info!("Transfer {} failed (retryable), will retry (attempt {}/{})", 
                        id, transfer.retry_count, transfer.max_retries);
                    
                    // Store new token
                    drop(transfers);
                    {
                        let mut tokens = self.cancel_tokens.write().await;
                        tokens.insert(id.to_string(), new_token.clone());
                    }
                    
                    Some((url, new_token))
                } else {
                    None
                }
            } else {
                None
            }
        };

        if retry_info.is_some() {
            return retry_info;
        }

        // Permanently failed - move to history
        let mut transfers = self.transfers.write().await;
        if let Some(mut transfer) = transfers.remove(id) {
            transfer.transfer.status = TransferStatus::Failed;
            transfer.transfer.error = Some(error);
            transfer.transfer.updated_at = Utc::now();

            // Record bot failure
            let bot = transfer.transfer.url.bot.clone();
            let network = transfer.transfer.url.network.clone();

            // Add to history
            drop(transfers);
            let mut history = self.history.write().await;
            history.push(transfer.transfer.clone());

            // Trim history
            let history_len = history.len();
            if history_len > self.max_history {
                history.drain(0..history_len - self.max_history);
            }

            self.record_bot_failure(&bot, &network).await;
            self.update_analytics(&transfer.transfer, false).await;
        }

        let mut tokens = self.cancel_tokens.write().await;
        tokens.remove(id);

        let mut queue = self.queue.write().await;
        queue.retain(|queue_id| queue_id != id);

        None
    }

    /// Mark transfer as completed
    /// Mark transfer as completed and move to history
    pub async fn set_completed(&self, id: &str) {
        let (bot, network, bytes, speed, transfer_copy) = {
            let mut transfers = self.transfers.write().await;
            if let Some(transfer) = transfers.get_mut(id) {
                transfer.transfer.status = TransferStatus::Completed;
                transfer.transfer.updated_at = Utc::now();
                transfer.transfer.progress = 100.0;

                let info = (
                    transfer.transfer.url.bot.clone(),
                    transfer.transfer.url.network.clone(),
                    transfer.transfer.size.unwrap_or(0),
                    transfer.transfer.speed,
                    transfer.transfer.clone(),
                );

                // Remove from active transfers
                transfers.remove(id);
                info
            } else {
                return;
            }
        };

        // Record bot success
        self.record_bot_success(&bot, &network, bytes, speed).await;

        // Add to history
        let mut history = self.history.write().await;
        history.push(transfer_copy.clone());

        // Trim history
        let history_len = history.len();
        if history_len > self.max_history {
            history.drain(0..history_len - self.max_history);
        }

        // Update analytics
        self.update_analytics(&transfer_copy, true).await;

        let mut tokens = self.cancel_tokens.write().await;
        tokens.remove(id);

        // Remove from queue just in case
        let mut queue = self.queue.write().await;
        queue.retain(|queue_id| queue_id != id);
    }

    /// Cancel a transfer
    pub async fn cancel_transfer(&self, id: &str) -> bool {
        // Check if transfer is finished (completed, failed, or cancelled)
        let is_finished = {
            let transfers = self.transfers.read().await;
            if let Some(transfer) = transfers.get(id) {
                matches!(
                    transfer.transfer.status,
                    TransferStatus::Completed | TransferStatus::Failed | TransferStatus::Cancelled
                )
            } else {
                false
            }
        };

        // If finished, just remove it
        if is_finished {
            return self.remove_transfer(id).await;
        }

        // Otherwise, cancel active transfer
        {
            let tokens = self.cancel_tokens.read().await;
            if let Some(token) = tokens.get(id) {
                token.cancel();
                tracing::info!("Cancelled download task for {}", id);
            }
        }

        let mut transfers = self.transfers.write().await;
        if let Some(transfer) = transfers.get_mut(id) {
            transfer.transfer.status = TransferStatus::Cancelled;
            transfer.transfer.updated_at = Utc::now();

            let mut tokens = self.cancel_tokens.write().await;
            tokens.remove(id);

            // Remove from queue if present
            let mut queue = self.queue.write().await;
            queue.retain(|queue_id| queue_id != id);

            return true;
        }
        false
    }

    /// Remove a transfer completely from the manager
    pub async fn remove_transfer(&self, id: &str) -> bool {
        let mut transfers = self.transfers.write().await;
        let removed = transfers.remove(id).is_some();

        if removed {
            // Also remove from cancel tokens and queue
            let mut tokens = self.cancel_tokens.write().await;
            tokens.remove(id);

            let mut queue = self.queue.write().await;
            queue.retain(|queue_id| queue_id != id);

            tracing::info!("Removed transfer {}", id);
        }

        removed
    }

    /// Delete history item
    pub async fn delete_history_item(&self, id: &str, delete_file: bool) -> bool {
        tracing::info!(
            "Attempting to delete history item: {}, delete_file: {}",
            id,
            delete_file
        );

        let mut history = self.history.write().await;
        if let Some(pos) = history.iter().position(|t| t.id == id) {
            let item = history.remove(pos);

            if delete_file {
                if let Some(filename) = item.filename {
                    let safe_filename =
                        filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                    let path = std::path::Path::new(&self.download_dir).join(&safe_filename);

                    tracing::info!("Attempting to delete file at path: {:?}", path);

                    if path.exists() {
                        match tokio::fs::remove_file(&path).await {
                            Ok(_) => tracing::info!("Successfully deleted file: {:?}", path),
                            Err(e) => tracing::error!("Failed to delete file {:?}: {}", path, e),
                        }
                    } else {
                        tracing::warn!("File not found for deletion: {:?}", path);
                    }
                } else {
                    tracing::warn!("No filename present for history item {}", id);
                }
            }
            return true;
        }

        tracing::warn!("History item {} not found", id);
        false
    }
}

impl Default for EnhancedTransferManager {
    fn default() -> Self {
        Self::new("./downloads".to_string())
    }
}
