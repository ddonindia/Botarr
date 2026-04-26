use crate::config::NetworkConfig;
use crate::xdcc::{XdccSearchResult, XdccUrl};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub providers: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct SearchResponse {
    pub results: Vec<XdccSearchResult>,
    pub count: usize,
}

#[derive(Debug, Deserialize)]
pub struct ParseUrlRequest {
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ParseUrlResponse {
    pub valid: bool,
    pub url: Option<XdccUrl>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct DownloadRequest {
    pub url: String,
    #[serde(default)]
    pub priority: Option<String>,
    #[serde(default)]
    pub filename: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DownloadResponse {
    pub transfer_id: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct PluginStatusResponse {
    pub loaded_scripts: Vec<String>,
    pub logs: HashMap<String, Vec<String>>,
    pub active_monitors: Vec<crate::xdcc::monitor::MonitorStatus>,
    pub raw_irc_logs: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct SetPriorityRequest {
    pub priority: String,
}

#[derive(Debug, Deserialize)]
pub struct HistoryRequest {
    #[serde(default = "default_history_page")]
    pub page: i64,
    #[serde(default = "default_history_limit")]
    pub limit: usize,
}

fn default_history_page() -> i64 {
    1
}
fn default_history_limit() -> usize {
    100
}

#[derive(Debug, Deserialize)]
pub struct DeleteHistoryParams {
    #[serde(default)]
    pub delete_file: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaginationParams {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_limit")]
    pub limit: i64,
}

fn default_page() -> i64 {
    1
}
fn default_limit() -> i64 {
    20
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteRequest {
    pub ids: Vec<String>,
    #[serde(default)]
    pub delete_files: bool,
}

#[derive(Debug, Deserialize)]
pub struct BulkDeleteSearchRequest {
    pub ids: Vec<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub use_ssl: Option<bool>,
    pub connect_timeout: Option<u64>,
    pub general_timeout: Option<u64>,
    pub proxy_enabled: Option<bool>,
    pub proxy_url: Option<String>,
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub realname: Option<String>,
    pub max_retries: Option<u32>,
    pub retry_delay: Option<u64>,
    pub queue_limit: Option<u32>,
    pub passive_dcc: Option<bool>,
    pub dcc_port_min: Option<u16>,
    pub dcc_port_max: Option<u16>,
    pub resume_enabled: Option<bool>,
    pub enabled_providers: Option<Vec<String>>,
    pub results_per_page: Option<u32>,
    pub search_timeout: Option<u64>,
    pub networks: Option<HashMap<String, NetworkConfig>>,
    pub move_completed: Option<bool>,
    pub move_completed_dir: Option<String>,
    pub postprocess_script_enabled: Option<bool>,
    pub postprocess_script: Option<String>,
    pub postprocess_timeout: Option<u64>,
}
