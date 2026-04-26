use crate::api::models::UpdateSettingsRequest;
use crate::config::NetworkConfig;
use crate::AppState;
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Json,
};

/// Get current settings
pub async fn get_settings(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.clone())
}

/// Update settings
pub async fn update_settings(
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
    if let Some(v) = req.networks {
        config.networks = v;
    }
    // Postprocessing settings
    if let Some(v) = req.move_completed {
        config.move_completed = v;
    }
    if let Some(v) = req.move_completed_dir {
        config.move_completed_dir = v;
    }
    if let Some(v) = req.postprocess_script_enabled {
        config.postprocess_script_enabled = v;
    }
    if let Some(v) = req.postprocess_script {
        config.postprocess_script = v;
    }
    if let Some(v) = req.postprocess_timeout {
        config.postprocess_timeout = v.clamp(10, 3600);
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
pub async fn get_networks(State(state): State<AppState>) -> impl IntoResponse {
    let config = state.config.read().await;
    Json(config.networks.clone())
}

/// Add or update a network
pub async fn update_network(
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
pub async fn delete_network(
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
