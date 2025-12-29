use crate::config::NetworkConfig;
use crate::xdcc::{
    TransferPriority, TransferStatus, XdccClient, XdccConfig, XdccEvent, XdccSearchResult, XdccUrl,
};
use crate::AppState;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post, put},
    Json, Router,
};
use serde::{Deserialize, Serialize};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/api/search", get(xdcc_search))
        .route("/api/parse", post(xdcc_parse_url))
        .route("/api/download", post(xdcc_download))
        .route("/api/transfers", get(xdcc_list_transfers))
        .route(
            "/api/transfers/{id}",
            get(xdcc_get_transfer).delete(xdcc_cancel_transfer),
        )
        .route("/api/transfers/{id}/retry", post(xdcc_retry_transfer))
        .route("/api/transfers/{id}/priority", post(xdcc_set_priority))
        .route("/api/bots/stats", get(xdcc_bot_stats))
        .route("/api/analytics", get(xdcc_analytics))
        .route("/api/history", get(xdcc_history))
        .route(
            "/api/history/{id}",
            axum::routing::delete(xdcc_delete_history),
        )
        .route("/api/history/bulk", post(xdcc_bulk_delete_history))
        .route("/api/search-history", get(xdcc_search_history))
        .route(
            "/api/search-history/{id}",
            axum::routing::delete(xdcc_delete_search_history),
        )
        .route(
            "/api/search-history/bulk",
            post(xdcc_bulk_delete_search_history),
        )
        .route("/api/queue", get(xdcc_queue_status))
        // Settings API
        .route("/api/settings", get(get_settings).put(update_settings))
        .route("/api/settings/networks", get(get_networks))
        .route(
            "/api/settings/networks/{name}",
            put(update_network).delete(delete_network),
        )
}

// ============= Request/Response Types =============

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
    pub priority: Option<String>, // "low", "normal", "high", "urgent"
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

// ============= Handlers =============

/// Search XDCC providers
/// GET /api/search?query=...
pub async fn xdcc_search(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<SearchRequest>,
) -> impl IntoResponse {
    let providers = params.providers.map(|p| {
        p.split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
    });

    match state
        .search_aggregator
        .search(&params.query, providers.as_deref())
        .await
    {
        Ok(results) => {
            let count = results.len();

            // Save search to history with results
            let results_json = serde_json::to_string(&results).ok();
            if let Err(e) =
                state
                    .database
                    .insert_search(&params.query, count as i64, results_json.as_deref())
            {
                tracing::error!("Failed to save search history: {}", e);
            }

            Json(SearchResponse { results, count }).into_response()
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: e.to_string(),
            }),
        )
            .into_response(),
    }
}

/// Parse and validate an XDCC URL
/// POST /api/parse
pub async fn xdcc_parse_url(Json(req): Json<ParseUrlRequest>) -> impl IntoResponse {
    match XdccUrl::parse(&req.url) {
        Ok(url) => Json(ParseUrlResponse {
            valid: true,
            url: Some(url),
            error: None,
        }),
        Err(e) => Json(ParseUrlResponse {
            valid: false,
            url: None,
            error: Some(e.to_string()),
        }),
    }
}

/// Start an XDCC download
/// POST /api/download
pub async fn xdcc_download(
    State(state): State<AppState>,
    Json(req): Json<DownloadRequest>,
) -> impl IntoResponse {
    // Parse the URL
    let url = match XdccUrl::parse(&req.url) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response();
        }
    };

    // Parse priority
    let priority = match req.priority.as_deref() {
        Some("low") => TransferPriority::Low,
        Some("high") => TransferPriority::High,
        Some("urgent") => TransferPriority::Urgent,
        _ => TransferPriority::Normal,
    };

    // Create transfer tracking with cancellation token
    let (transfer_id, cancel_token) = {
        let tm = state.transfer_manager.write().await;
        tm.create_transfer(url.clone(), priority).await
    };

    // Clone what we need for the background task
    let download_dir = state.download_dir.clone();
    let transfer_manager = state.transfer_manager.clone();
    let config = state.config.clone();
    let tid = transfer_id.clone();

    // Start the download in a background task
    let _handle = tokio::spawn(async move {
        tracing::info!("Starting XDCC download task for {}", tid);

        // Build XdccConfig from AppConfig
        let app_config = config.read().await;
        let client_config = XdccConfig {
            nickname: app_config.nickname.clone(),
            username: app_config.username.clone(),
            realname: app_config.realname.clone(),
            use_ssl: app_config.use_ssl,
            connect_timeout_secs: app_config.connect_timeout,
            timeout_secs: app_config.general_timeout,
            download_dir,
            networks: app_config
                .networks
                .iter()
                .map(|(k, v)| (k.clone(), (v.host.clone(), v.port, v.ssl)))
                .collect(),
            proxy_enabled: app_config.proxy_enabled,
            proxy_url: app_config.proxy_url.clone(),
        };
        drop(app_config); // Release lock before async operations

        let client = XdccClient::new(client_config);

        // Update status
        {
            let tm = transfer_manager.write().await;
            tm.update_status(&tid, TransferStatus::Connecting).await;
        }

        match client.start_download(url).await {
            Ok(mut rx) => {
                tracing::info!("Download channel open for {}", tid);
                loop {
                    tokio::select! {
                        // Check for cancellation
                        _ = cancel_token.cancelled() => {
                            tracing::info!("Download cancelled for {}", tid);
                            break;
                        }
                        // Process events
                        event = rx.recv() => {
                            match event {
                                Some(XdccEvent::Connecting) => {
                                    let tm = transfer_manager.write().await;
                                    tm.update_status(&tid, TransferStatus::Connecting).await;
                                }
                                Some(XdccEvent::Joining(channel)) => {
                                    tracing::info!("Joining channel: {}", channel);
                                    let tm = transfer_manager.write().await;
                                    tm.update_status(&tid, TransferStatus::Joining).await;
                                }
                                Some(XdccEvent::Joined(channel)) => {
                                    tracing::info!("Joined channel: {}", channel);
                                }
                                Some(XdccEvent::Requesting(bot, slot)) => {
                                    tracing::info!("Requesting pack #{} from {}", slot, bot);
                                    let tm = transfer_manager.write().await;
                                    tm.update_status(&tid, TransferStatus::Requesting).await;
                                }
                                Some(XdccEvent::DccSend { filename, size, ip, port }) => {
                                    tracing::info!("DCC SEND from {}:{} - {} ({} bytes)", ip, port, filename, size);
                                    let tm = transfer_manager.write().await;
                                    tm.set_file_info(&tid, filename, size).await;
                                    tm.update_status(&tid, TransferStatus::Downloading).await;
                                }
                                Some(XdccEvent::Progress { downloaded, total, speed }) => {
                                    let tm = transfer_manager.write().await;
                                    tm.update_progress(&tid, downloaded, speed).await;
                                    // Log progress periodically
                                    if downloaded % (10 * 1024 * 1024) < 65536 {
                                        let pct = if total > 0 { (downloaded as f64 / total as f64) * 100.0 } else { 0.0 };
                                        tracing::debug!("Download progress: {:.1}% ({}/{} bytes)", pct, downloaded, total);
                                    }
                                }
                                Some(XdccEvent::Completed) => {
                                    tracing::info!("Download completed for {}", tid);
                                    let tm = transfer_manager.write().await;
                                    tm.set_completed(&tid).await;
                                    break;
                                }
                                Some(XdccEvent::Error(e)) => {
                                    tracing::error!("Download error for {}: {}", tid, e);
                                    let tm = transfer_manager.write().await;
                                    tm.set_failed(&tid, e).await;
                                    break;
                                }
                                None => break, // Channel closed
                                _ => {}
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to start download {}: {}", tid, e);
                let tm = transfer_manager.write().await;
                tm.set_failed(&tid, e.to_string()).await;
            }
        }
        tracing::info!("Download task finished for {}", tid);
    });

    Json(DownloadResponse {
        transfer_id,
        status: "started".to_string(),
    })
    .into_response()
}

/// List all transfers
/// GET /api/transfers
pub async fn xdcc_list_transfers(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let transfers = tm.list_transfers().await;
    // Serialize enhanced transfers (includes priority, retry_count, queue_position)
    Json(serde_json::json!({ "transfers": transfers }))
}

/// Get a specific transfer
/// GET /api/transfers/:id
pub async fn xdcc_get_transfer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    match tm.get_transfer(&id).await {
        Some(transfer) => Json(transfer).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Transfer not found".to_string(),
            }),
        )
            .into_response(),
    }
}

/// Cancel a transfer
/// DELETE /api/transfers/:id
pub async fn xdcc_cancel_transfer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.write().await;
    if tm.cancel_transfer(&id).await {
        Json(serde_json::json!({"status": "cancelled"})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Transfer not found".to_string(),
            }),
        )
            .into_response()
    }
}

// ============= New Enhanced Feature Handlers =============

/// Retry a failed transfer
/// POST /api/transfers/:id/retry
pub async fn xdcc_retry_transfer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.write().await;
    if tm.retry_transfer(&id).await {
        Json(serde_json::json!({"status": "retrying", "transfer_id": id})).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Cannot retry transfer (not found or max retries reached)".to_string(),
            }),
        )
            .into_response()
    }
}

/// Set transfer priority
/// POST /api/transfers/:id/priority
#[derive(Debug, Deserialize)]
pub struct SetPriorityRequest {
    pub priority: String, // "low", "normal", "high", "urgent"
}

pub async fn xdcc_set_priority(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<SetPriorityRequest>,
) -> impl IntoResponse {
    let priority = match req.priority.as_str() {
        "low" => TransferPriority::Low,
        "high" => TransferPriority::High,
        "urgent" => TransferPriority::Urgent,
        _ => TransferPriority::Normal,
    };

    let tm = state.transfer_manager.write().await;
    if tm.set_priority(&id, priority).await {
        Json(serde_json::json!({"status": "updated", "priority": req.priority})).into_response()
    } else {
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Transfer not found".to_string(),
            }),
        )
            .into_response()
    }
}

/// Get bot statistics
/// GET /api/bots/stats
pub async fn xdcc_bot_stats(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let stats = tm.get_all_bot_stats().await;
    Json(serde_json::json!({ "bots": stats }))
}

/// Get analytics
/// GET /api/analytics
pub async fn xdcc_analytics(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let analytics = tm.get_analytics().await;
    Json(analytics)
}

/// Get download history
/// GET /api/history?limit=100
#[derive(Debug, Deserialize)]
pub struct HistoryRequest {
    #[serde(default = "default_history_limit")]
    pub limit: usize,
}

fn default_history_limit() -> usize {
    100
}

pub async fn xdcc_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<HistoryRequest>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let history = tm.get_history(params.limit).await;
    Json(serde_json::json!({ "history": history, "count": history.len() }))
}

/// Get queue status
/// GET /api/queue
pub async fn xdcc_queue_status(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let queue_size = tm.queue_size().await;
    Json(serde_json::json!({
        "queue_size": queue_size,
        "status": "ok"
    }))
}

#[derive(Debug, Deserialize)]
pub struct DeleteHistoryParams {
    #[serde(default)]
    pub delete_file: bool,
}

/// Delete history item
/// DELETE /api/history/:id?delete_file=true
pub async fn xdcc_delete_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    axum::extract::Query(params): axum::extract::Query<DeleteHistoryParams>,
) -> impl IntoResponse {
    tracing::info!(
        "DELETE /api/history/{} called, delete_file={}",
        id,
        params.delete_file
    );

    let tm = state.transfer_manager.write().await;

    // Log current history state for debugging
    let history_count = tm.get_history(100).await.len();
    tracing::info!("Current history count: {}", history_count);

    if tm.delete_history_item(&id, params.delete_file).await {
        // Also delete from database
        let _ = state.database.delete_download(&id);
        Json(serde_json::json!({"status": "deleted"})).into_response()
    } else {
        tracing::warn!(
            "History item {} not found in {} history items",
            id,
            history_count
        );
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "History item not found".to_string(),
            }),
        )
            .into_response()
    }
}

// ============= Pagination Params =============

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

// ============= Bulk Delete History =============

/// Bulk delete download history
/// POST /api/history/bulk
pub async fn xdcc_bulk_delete_history(
    State(state): State<AppState>,
    Json(req): Json<BulkDeleteRequest>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.write().await;
    let mut deleted = 0;

    for id in &req.ids {
        if tm.delete_history_item(id, req.delete_files).await {
            let _ = state.database.delete_download(id);
            deleted += 1;
        }
    }

    Json(serde_json::json!({
        "status": "ok",
        "deleted": deleted
    }))
}

// ============= Search History Endpoints =============

/// Get search history with pagination
/// GET /api/search-history?page=1&limit=20
pub async fn xdcc_search_history(
    State(state): State<AppState>,
    axum::extract::Query(params): axum::extract::Query<PaginationParams>,
) -> impl IntoResponse {
    match state.database.list_searches(params.page, params.limit) {
        Ok(response) => Json(response).into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Delete a search history item
/// DELETE /api/search-history/:id
pub async fn xdcc_delete_search_history(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> impl IntoResponse {
    match state.database.delete_search(id) {
        Ok(true) => Json(serde_json::json!({"status": "deleted"})).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "Search history item not found".to_string(),
            }),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Bulk delete search history
/// POST /api/search-history/bulk
pub async fn xdcc_bulk_delete_search_history(
    State(state): State<AppState>,
    Json(req): Json<BulkDeleteSearchRequest>,
) -> impl IntoResponse {
    match state.database.bulk_delete_searches(&req.ids) {
        Ok(deleted) => Json(serde_json::json!({
            "status": "ok",
            "deleted": deleted
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Database error: {}", e),
            }),
        )
            .into_response(),
    }
}

// ============= Settings API Handlers =============

/// Get current settings
async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.clone())
}

/// Update settings request (partial update)
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
}

/// Update settings
async fn update_settings(
    State(state): State<AppState>,
    Json(req): Json<UpdateSettingsRequest>,
) -> impl IntoResponse {
    let mut config = state.config.write().await;

    // Apply partial updates
    if let Some(v) = req.use_ssl {
        config.use_ssl = v;
    }
    if let Some(v) = req.connect_timeout {
        config.connect_timeout = v.clamp(5, 60);
    }
    if let Some(v) = req.general_timeout {
        config.general_timeout = v.clamp(30, 300);
    }
    if let Some(v) = req.proxy_enabled {
        config.proxy_enabled = v;
    }
    if let Some(v) = req.proxy_url {
        config.proxy_url = v;
    }
    if let Some(v) = req.nickname {
        config.nickname = v;
    }
    if let Some(v) = req.username {
        config.username = v;
    }
    if let Some(v) = req.realname {
        config.realname = v;
    }
    if let Some(v) = req.max_retries {
        config.max_retries = v.clamp(0, 10);
    }
    if let Some(v) = req.retry_delay {
        config.retry_delay = v.clamp(5, 300);
    }
    if let Some(v) = req.queue_limit {
        config.queue_limit = v.clamp(1, 10);
    }
    if let Some(v) = req.passive_dcc {
        config.passive_dcc = v;
    }
    if let Some(v) = req.dcc_port_min {
        config.dcc_port_min = v.max(1024);
    }
    if let Some(v) = req.dcc_port_max {
        config.dcc_port_max = v;
    }
    if let Some(v) = req.resume_enabled {
        config.resume_enabled = v;
    }
    if let Some(v) = req.enabled_providers {
        config.enabled_providers = v;
    }
    if let Some(v) = req.results_per_page {
        config.results_per_page = v.clamp(10, 200);
    }
    if let Some(v) = req.search_timeout {
        config.search_timeout = v.clamp(10, 120);
    }

    // Save to file
    let config_path =
        std::env::var("BOTARR_CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
    if let Err(e) = config.save(&config_path) {
        tracing::warn!("Failed to save config: {}", e);
    }

    Json(serde_json::json!({ "status": "ok" }))
}

/// Get all networks
async fn get_networks(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.networks.clone())
}

/// Add or update a network
async fn update_network(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(network): Json<NetworkConfig>,
) -> impl IntoResponse {
    let mut config = state.config.write().await;
    config.networks.insert(name.clone(), network);

    // Save to file
    let config_path =
        std::env::var("BOTARR_CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
    if let Err(e) = config.save(&config_path) {
        tracing::warn!("Failed to save config: {}", e);
    }

    Json(serde_json::json!({ "status": "ok", "network": name }))
}

/// Delete a network
async fn delete_network(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> impl IntoResponse {
    let mut config = state.config.write().await;

    if config.networks.remove(&name).is_some() {
        // Save to file
        let config_path =
            std::env::var("BOTARR_CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
        if let Err(e) = config.save(&config_path) {
            tracing::warn!("Failed to save config: {}", e);
        }
        Json(serde_json::json!({ "status": "ok", "deleted": name }))
    } else {
        Json(serde_json::json!({ "status": "error", "message": "Network not found" }))
    }
}
