use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio_util::sync::CancellationToken;

use crate::api::models::*;
use crate::config::AppConfig;
use crate::postprocess::{run_postprocess, PostprocessConfig};
use crate::xdcc::transfer::EnhancedTransferManager;
use crate::xdcc::{TransferPriority, TransferStatus, XdccClient, XdccConfig, XdccEvent, XdccUrl};
use crate::AppState;

pub fn spawn_download_task(
    tid: String,
    url: XdccUrl,
    cancel_token: CancellationToken,
    download_dir: String,
    transfer_manager: Arc<RwLock<EnhancedTransferManager>>,
    config: Arc<RwLock<AppConfig>>,
    plugin_manager: Arc<crate::plugin::PluginManager>,
) {
    tokio::spawn(async move {
        tracing::info!("Starting XDCC download task for {}", tid);

        let app_config = config.read().await;
        let client_config = XdccConfig {
            nickname: app_config.nickname.clone(),
            username: app_config.username.clone(),
            realname: app_config.realname.clone(),
            use_ssl: app_config.use_ssl,
            connect_timeout_secs: app_config.connect_timeout,
            timeout_secs: app_config.general_timeout,
            download_dir: download_dir.clone(),
            networks: app_config
                .networks
                .iter()
                .map(|(k, v)| {
                    (
                        k.clone(),
                        (
                            v.host.clone(),
                            v.port,
                            v.ssl,
                            v.autojoin_channels.clone(),
                            v.join_delay_secs,
                            v.nickserv_password.clone(),
                        ),
                    )
                })
                .collect(),
            proxy_enabled: app_config.proxy_enabled,
            proxy_url: app_config.proxy_url.clone(),
            resume_enabled: app_config.resume_enabled,
        };
        drop(app_config);

        let client = XdccClient::new(client_config);

        {
            let tm = transfer_manager.write().await;
            tm.update_status(&tid, TransferStatus::Connecting).await;
        }

        let mut retry_info: Option<(XdccUrl, CancellationToken)> = None;

        match client.start_download(url).await {
            Ok(mut rx) => {
                tracing::info!("Download channel open for {}", tid);
                loop {
                    tokio::select! {
                        _ = cancel_token.cancelled() => {
                            tracing::info!("Download cancelled for {}", tid);
                            break;
                        }
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
                                    tm.add_log(&tid, format!("DCC SEND from {}:{} - {} ({} bytes)", ip, port, filename, size)).await;
                                    tm.set_file_info(&tid, filename.clone(), size).await;
                                    tm.update_status(&tid, TransferStatus::Downloading).await;
                                    plugin_manager.emit_signal("download_started", crate::plugin::EventData::String(filename));
                                }
                                Some(XdccEvent::Progress { downloaded, total, speed }) => {
                                    let tm = transfer_manager.write().await;
                                    tm.update_progress(&tid, downloaded, speed).await;
                                    if downloaded % (10 * 1024 * 1024) < 65536 {
                                        let pct = if total > 0 { (downloaded as f64 / total as f64) * 100.0 } else { 0.0 };
                                        tracing::debug!("Download progress: {:.1}% ({}/{} bytes)", pct, downloaded, total);
                                    }
                                }
                                Some(XdccEvent::Completed) => {
                                    tracing::info!("Download completed for {}", tid);
                                    {
                                        let tm = transfer_manager.write().await;
                                        tm.add_log(&tid, "Download completed successfully".to_string()).await;
                                    }

                                    let completed_filename = {
                                        let tm = transfer_manager.read().await;
                                        if let Some(t) = tm.get_transfer(&tid).await {
                                            t.transfer.filename.clone()
                                        } else {
                                            None
                                        }
                                    };

                                    {
                                        let tm = transfer_manager.write().await;
                                        tm.set_completed(&tid).await;
                                    }
                                    if let Some(filename) = completed_filename.clone() {
                                        plugin_manager.emit_signal("download_completed", crate::plugin::EventData::String(filename));
                                    }

                                    if let Some(filename) = completed_filename {
                                        let app_config = config.read().await;
                                        if app_config.move_completed || app_config.postprocess_script_enabled {
                                            let pp_config = PostprocessConfig {
                                                move_completed_dir: if app_config.move_completed && !app_config.move_completed_dir.is_empty() {
                                                    Some(app_config.move_completed_dir.clone())
                                                } else {
                                                    None
                                                },
                                                script_path: if app_config.postprocess_script_enabled && !app_config.postprocess_script.is_empty() {
                                                    Some(app_config.postprocess_script.clone())
                                                } else {
                                                    None
                                                },
                                                script_timeout_secs: app_config.postprocess_timeout,
                                            };
                                            drop(app_config);

                                            let safe_filename = filename.replace(['/', '\\', ':', '*', '?', '"', '<', '>', '|'], "_");
                                            let file_path = std::path::Path::new(&download_dir)
                                                .join(&safe_filename)
                                                .to_string_lossy()
                                                .to_string();

                                            tracing::info!("Running postprocessing on: {}", file_path);
                                            let result = run_postprocess(&file_path, &pp_config).await;

                                            if !result.errors.is_empty() {
                                                for err in &result.errors {
                                                    tracing::warn!("Postprocessing warning: {}", err);
                                                }
                                            }
                                            if let Some(moved_to) = result.moved_to {
                                                tracing::info!("File moved to: {}", moved_to);
                                            }
                                            if let Some(exit_code) = result.script_exit_code {
                                                tracing::info!("Postprocess script exited with code: {}", exit_code);
                                            }
                                        }
                                    }
                                    break;
                                }
                                Some(XdccEvent::Error(e)) => {
                                    tracing::error!("Download error for {}: {}", tid, e);
                                    plugin_manager.emit_signal("download_failed", crate::plugin::EventData::String(format!("{}", e)));
                                    let tm = transfer_manager.write().await;
                                    tm.add_log(&tid, format!("Error: {}", e)).await;
                                    retry_info = tm.set_failed(&tid, e.to_string(), e.is_fatal()).await;
                                    break;
                                }
                                Some(XdccEvent::IrcMessage(network, channel, nick, message)) => {
                                    plugin_manager.emit_signal("irc_message", crate::plugin::EventData::Tuple4(network, channel, nick, message));
                                }
                                Some(XdccEvent::IrcNotice(nick, message)) => {
                                    plugin_manager.emit_signal("irc_notice", crate::plugin::EventData::Tuple2(nick, message));
                                }
                                Some(XdccEvent::Log(msg)) => {
                                    let tm = transfer_manager.write().await;
                                    tm.add_log(&tid, msg).await;
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
                retry_info = tm.set_failed(&tid, e.to_string(), e.is_fatal()).await;
            }
        }

        if let Some((retry_url, new_token)) = retry_info {
            tracing::info!("Spawning retry download for {}", tid);
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;
            spawn_download_task(
                tid.clone(),
                retry_url,
                new_token,
                download_dir,
                transfer_manager,
                config,
                plugin_manager.clone(),
            );
        } else {
            tracing::info!("Download task finished for {}", tid);
        }
    });
}

pub async fn xdcc_search(
    State(state): State<AppState>,
    Query(params): Query<SearchRequest>,
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

pub async fn xdcc_download(
    State(state): State<AppState>,
    Json(req): Json<DownloadRequest>,
) -> impl IntoResponse {
    let url = match XdccUrl::parse(&req.url) {
        Ok(u) => u,
        Err(e) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(ErrorResponse {
                    error: e.to_string(),
                }),
            )
                .into_response()
        }
    };

    let priority = match req.priority.as_deref() {
        Some("low") => TransferPriority::Low,
        Some("high") => TransferPriority::High,
        Some("urgent") => TransferPriority::Urgent,
        _ => TransferPriority::Normal,
    };

    let result = {
        let tm = state.transfer_manager.write().await;
        tm.create_transfer(url.clone(), priority, true, req.filename.clone())
            .await
    };

    let (transfer_id, _cancel_token) = match result {
        Ok(res) => res,
        Err(e) => {
            return (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e })).into_response()
        }
    };

    Json(DownloadResponse {
        transfer_id,
        status: "paused".to_string(),
    })
    .into_response()
}

pub async fn xdcc_list_transfers(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let transfers = tm.list_transfers().await;
    Json(serde_json::json!({ "transfers": transfers }))
}

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
                error: "Cannot retry transfer".to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn xdcc_resume_transfer(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.write().await;
    if tm.resume_transfer(&id).await {
        Json(serde_json::json!({"status": "resumed", "transfer_id": id})).into_response()
    } else {
        (
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "Cannot resume transfer".to_string(),
            }),
        )
            .into_response()
    }
}

pub async fn xdcc_get_transfer_logs(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let logs = tm.get_logs(&id).await;
    Json(serde_json::json!({ "logs": logs })).into_response()
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

pub async fn xdcc_bot_stats(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let stats = tm.get_all_bot_stats().await;
    Json(serde_json::json!({ "bots": stats }))
}

pub async fn xdcc_analytics(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let analytics = tm.get_analytics().await;
    Json(analytics)
}

pub async fn xdcc_queue_status(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.read().await;
    let queue_size = tm.queue_size().await;
    Json(serde_json::json!({
        "queue_size": queue_size,
        "status": "ok"
    }))
}
