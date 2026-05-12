pub mod handlers;
pub mod models;

use crate::AppState;
use axum::{
    routing::{delete, get, post, put},
    Router,
};

pub use handlers::downloads::spawn_download_task;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Downloads & Queue
        .route("/api/search", get(handlers::downloads::xdcc_search))
        .route("/api/parse", post(handlers::downloads::xdcc_parse_url))
        .route("/api/download", post(handlers::downloads::xdcc_download))
        .route(
            "/api/transfers",
            get(handlers::downloads::xdcc_list_transfers),
        )
        .route(
            "/api/transfers/{id}",
            get(handlers::downloads::xdcc_get_transfer)
                .delete(handlers::downloads::xdcc_cancel_transfer),
        )
        .route(
            "/api/transfers/{id}/retry",
            post(handlers::downloads::xdcc_retry_transfer),
        )
        .route(
            "/api/transfers/{id}/resume",
            post(handlers::downloads::xdcc_resume_transfer),
        )
        .route(
            "/api/transfers/{id}/priority",
            post(handlers::downloads::xdcc_set_priority),
        )
        .route(
            "/api/transfers/{id}/logs",
            get(handlers::downloads::xdcc_get_transfer_logs),
        )
        .route("/api/bots/stats", get(handlers::downloads::xdcc_bot_stats))
        .route("/api/analytics", get(handlers::downloads::xdcc_analytics))
        .route("/api/queue", get(handlers::downloads::xdcc_queue_status))
        // History
        .route(
            "/api/history",
            get(handlers::history::xdcc_history).delete(handlers::history::xdcc_clear_history),
        )
        .route(
            "/api/history/{id}",
            delete(handlers::history::xdcc_delete_history),
        )
        .route(
            "/api/history/bulk",
            post(handlers::history::xdcc_bulk_delete_history),
        )
        .route(
            "/api/search-history",
            get(handlers::history::xdcc_search_history)
                .delete(handlers::history::xdcc_clear_search_history),
        )
        .route(
            "/api/search-history/{id}",
            delete(handlers::history::xdcc_delete_search_history),
        )
        .route(
            "/api/search-history/bulk",
            post(handlers::history::xdcc_bulk_delete_search_history),
        )
        // Settings & Networks
        .route(
            "/api/settings",
            get(handlers::settings::get_settings).put(handlers::settings::update_settings),
        )
        .route(
            "/api/settings/networks",
            get(handlers::settings::get_networks),
        )
        .route(
            "/api/settings/networks/{name}",
            put(handlers::settings::update_network).delete(handlers::settings::delete_network),
        )
        // Plugins & System
        .route(
            "/api/plugins/status",
            get(handlers::system::get_plugin_status),
        )
        .route(
            "/api/plugins/autodl/filters",
            get(handlers::system::get_autodl_filters).put(handlers::system::update_autodl_filters),
        )
        .route("/api/irc/ws", get(handlers::system::irc_ws_handler))
}
