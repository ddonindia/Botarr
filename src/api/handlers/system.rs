use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use futures::{sink::SinkExt, stream::StreamExt};
use std::collections::HashMap;

use crate::api::models::{ErrorResponse, PluginStatusResponse};
use crate::AppState;

pub async fn get_plugin_status(State(state): State<AppState>) -> Json<PluginStatusResponse> {
    let loaded_scripts = state.plugin_manager.loaded_scripts.read().unwrap().clone();

    let mut logs = HashMap::new();
    {
        let logs_map = state.plugin_manager.recent_logs.read().unwrap();
        for (k, v) in logs_map.iter() {
            logs.insert(k.clone(), v.iter().cloned().collect());
        }
    }

    let active_monitors_lock = state.irc_monitor.active_monitors.clone();
    let active_monitors = active_monitors_lock.read().await.clone();

    let raw_irc_logs_lock = state.irc_monitor.raw_logs.clone();
    let raw_irc_logs = raw_irc_logs_lock.read().await.iter().cloned().collect();

    Json(PluginStatusResponse {
        loaded_scripts,
        logs,
        active_monitors,
        raw_irc_logs,
    })
}

pub async fn get_autodl_filters() -> impl IntoResponse {
    match std::fs::read_to_string("plugins/autodl.json") {
        Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
            Ok(json) => Json(json).into_response(),
            Err(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: "Failed to parse autodl.json".into(),
                }),
            )
                .into_response(),
        },
        Err(_) => Json(serde_json::json!({ "filters": [] })).into_response(), // Default empty if not exists
    }
}

pub async fn update_autodl_filters(
    State(state): State<AppState>,
    Json(filters): Json<serde_json::Value>,
) -> impl IntoResponse {
    match serde_json::to_string_pretty(&filters) {
        Ok(json_str) => {
            if let Err(e) = std::fs::write("plugins/autodl.json", json_str) {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ErrorResponse {
                        error: format!("Failed to save: {}", e),
                    }),
                )
                    .into_response();
            }

            // Always stop old monitors first to prevent duplicates when updating filters or toggling
            state
                .irc_monitor
                .stop_monitors_for_plugin("autodl.lua")
                .await;

            // Check if plugin is enabled, attempt to load the plugin if not already loaded
            if let Some(enabled) = filters.get("enabled").and_then(|f| f.as_bool()) {
                if enabled {
                    state
                        .plugin_manager
                        .load_script_file(std::path::Path::new("plugins/autodl.lua"));
                    // Tell the plugin to reload its config and restart monitors
                    state.plugin_manager.emit_signal(
                        "config_changed",
                        crate::plugin::EventData::String("autodl.lua".to_string()),
                    );
                }
            } else if let Some(filters_array) = filters.get("filters").and_then(|f| f.as_array()) {
                // Fallback for older formats
                if !filters_array.is_empty() {
                    state
                        .plugin_manager
                        .load_script_file(std::path::Path::new("plugins/autodl.lua"));
                }
            }

            Json(serde_json::json!({ "success": true })).into_response()
        }
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!("Invalid JSON: {}", e),
            }),
        )
            .into_response(),
    }
}

pub async fn irc_ws_handler(
    ws: WebSocketUpgrade,
    State(state): State<AppState>,
) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_irc_socket(socket, state))
}

async fn handle_irc_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.irc_client_manager.ws_tx.subscribe();
    let client_manager = state.irc_client_manager.clone();

    // Spawn a task to receive messages from the IRC manager and send to the WebSocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if let Ok(json) = serde_json::to_string(&msg) {
                if sender.send(Message::Text(json.into())).await.is_err() {
                    break;
                }
            }
        }
    });

    // Spawn a task to receive messages from the WebSocket and send to the IRC manager
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                if let Ok(cmd) = serde_json::from_str::<crate::irc_client::WsCommand>(&text) {
                    client_manager.handle_command(cmd).await;
                }
            }
        }
    });

    // If any one of the tasks exit, abort the other.
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
