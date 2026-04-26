use crate::api::models::*;
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

/// Get download history
pub async fn xdcc_history(
    State(state): State<AppState>,
    Query(params): Query<HistoryRequest>,
) -> impl IntoResponse {
    match state
        .database
        .list_downloads(params.page, params.limit as i64)
    {
        Ok(history) => Json(history).into_response(),
        Err(e) => {
            tracing::error!("Failed to fetch download history: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponse {
                    error: format!("Database error: {}", e),
                }),
            )
                .into_response()
        }
    }
}

/// Delete history item
pub async fn xdcc_delete_history(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Query(params): Query<DeleteHistoryParams>,
) -> impl IntoResponse {
    tracing::info!(
        "DELETE /api/history/{} called, delete_file={}",
        id,
        params.delete_file
    );

    let tm = state.transfer_manager.write().await;

    if tm.delete_history_item(&id, params.delete_file).await {
        Json(serde_json::json!({"status": "deleted"})).into_response()
    } else {
        tracing::warn!("History item {} not found", id);
        (
            StatusCode::NOT_FOUND,
            Json(ErrorResponse {
                error: "History item not found".to_string(),
            }),
        )
            .into_response()
    }
}

/// Clear all download history
pub async fn xdcc_clear_history(State(state): State<AppState>) -> impl IntoResponse {
    let tm = state.transfer_manager.write().await;

    // Clear from TransferManager memory
    tm.clear_history().await;

    // Clear from database
    match state.database.clear_download_history() {
        Ok(deleted) => Json(serde_json::json!({
            "status": "cleared",
            "deleted": deleted
        }))
        .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: format!("Failed to clear history from database: {}", e),
            }),
        )
            .into_response(),
    }
}

/// Bulk delete download history
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

/// Get search history with pagination
pub async fn xdcc_search_history(
    State(state): State<AppState>,
    Query(params): Query<PaginationParams>,
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

/// Clear all search history
pub async fn xdcc_clear_search_history(State(state): State<AppState>) -> impl IntoResponse {
    match state.database.clear_search_history() {
        Ok(deleted) => Json(serde_json::json!({
            "status": "cleared",
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

/// Delete a search history item
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
