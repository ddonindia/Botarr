mod api;
mod config;
mod db;
mod irc_client;
mod plugin;
mod postprocess;
mod xdcc;

use crate::config::AppConfig;
use crate::xdcc::{SearchAggregator, TransferManager};
use axum::{
    http::{header, StatusCode, Uri},
    response::IntoResponse,
    Router,
};
use rust_embed::RustEmbed;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(RustEmbed)]
#[folder = "web/dist"]
struct Asset;

#[derive(Clone)]
pub struct AppState {
    pub search_aggregator: Arc<SearchAggregator>,
    pub transfer_manager: Arc<RwLock<TransferManager>>,
    pub download_dir: String,
    pub database: Arc<db::Database>,
    pub config: Arc<RwLock<AppConfig>>,
    pub plugin_manager: Arc<plugin::PluginManager>,
    pub irc_monitor: Arc<xdcc::monitor::IrcMonitor>,
    pub irc_client_manager: Arc<irc_client::InteractiveClientManager>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "botarr=debug,api=debug,xdcc=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting Botarr...");

    // Initialize download directory
    let download_dir =
        std::env::var("BOTARR_DOWNLOAD_DIR").unwrap_or_else(|_| "downloads".to_string());
    tokio::fs::create_dir_all(&download_dir).await?;

    // Initialize database
    let db_path = std::env::var("BOTARR_DB_PATH").unwrap_or_else(|_| "botarr.db".to_string());
    let database = db::Database::new(&db_path)
        .map_err(|e| anyhow::anyhow!("Failed to initialize database: {}", e))?;
    tracing::info!("Database initialized at: {}", db_path);

    // Load application config
    let config_path =
        std::env::var("BOTARR_CONFIG_FILE").unwrap_or_else(|_| "config.json".to_string());
    let mut app_config = AppConfig::load(&config_path);
    app_config.download_dir = download_dir.clone();
    tracing::info!(
        "Config loaded with {} networks configured",
        app_config.networks.len()
    );

    let database = Arc::new(database);
    let mut tm = TransferManager::new(download_dir.clone());
    tm.set_database(database.clone());
    let _restored_transfers = tm.restore_incomplete_transfers().await;
    tm.restore_recent_finished_transfers(20).await;

    // Initialize Plugin Manager
    let (plugin_manager, mut plugin_rx) = match plugin::PluginManager::new() {
        Ok(p) => p,
        Err(e) => {
            tracing::error!("Failed to init plugin manager: {:?}", e);
            std::process::exit(1);
        }
    };
    let plugin_manager = Arc::new(plugin_manager);

    let plugins_dir = std::path::Path::new("plugins");
    std::fs::create_dir_all(plugins_dir).unwrap_or_default();
    plugin_manager.load_scripts(plugins_dir);

    let irc_monitor = Arc::new(xdcc::monitor::IrcMonitor::new(
        Arc::new(RwLock::new(app_config.clone())),
        plugin_manager.clone(),
    ));

    let irc_client_manager = Arc::new(irc_client::InteractiveClientManager::new());

    let state = AppState {
        search_aggregator: Arc::new(SearchAggregator::with_default_providers(None)),
        transfer_manager: Arc::new(RwLock::new(tm)),
        download_dir: download_dir.clone(),
        database: database.clone(),
        config: Arc::new(RwLock::new(app_config)),
        plugin_manager: plugin_manager.clone(),
        irc_monitor: irc_monitor.clone(),
        irc_client_manager: irc_client_manager.clone(),
    };

    let monitor_clone = irc_monitor.clone();
    let tm_clone = state.transfer_manager.clone();
    tokio::spawn(async move {
        while let Some(action) = plugin_rx.recv().await {
            match action {
                plugin::PluginAction::MonitorChannel(plugin_name, network, channel) => {
                    monitor_clone.start_monitoring(plugin_name, network, channel);
                }
                plugin::PluginAction::Download(url, filename) => {
                    let lock = tm_clone.read().await;
                    if let Ok(xdcc_url) = crate::xdcc::XdccUrl::parse(&url) {
                        let _ = lock
                            .create_transfer(
                                xdcc_url,
                                crate::xdcc::transfer::TransferPriority::Normal,
                                false,
                                filename,
                            )
                            .await;
                    }
                }
                plugin::PluginAction::Queue(url, filename) => {
                    let lock = tm_clone.read().await;
                    if let Ok(xdcc_url) = crate::xdcc::XdccUrl::parse(&url) {
                        let _ = lock
                            .create_transfer(
                                xdcc_url,
                                crate::xdcc::transfer::TransferPriority::Normal,
                                true,
                                filename,
                            )
                            .await;
                    }
                }
            }
        }
    });

    // Build router
    let app = Router::new()
        .merge(api::routes())
        .fallback(static_handler)
        .with_state(state.clone()); // state must be cloned here because we need it below

    // Start Queue Processor
    let queue_state = state.clone();
    tokio::spawn(async move {
        tracing::info!("Queue processor started");
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(5)).await;

            let active_count = {
                let tm = queue_state.transfer_manager.read().await;
                let transfers = tm.list_transfers().await;
                transfers
                    .iter()
                    .filter(|t| {
                        let status = format!("{:?}", t.transfer.status).to_lowercase();
                        matches!(
                            status.as_str(),
                            "connecting" | "joining" | "requesting" | "downloading"
                        )
                    })
                    .count()
            };

            let limit = {
                let cfg = queue_state.config.read().await;
                cfg.queue_limit as usize
            };

            if active_count < limit {
                let pop_result = {
                    let tm = queue_state.transfer_manager.write().await;
                    tm.pop_queue().await
                };

                if let Some((id, url, token)) = pop_result {
                    tracing::info!("Popped transfer {} from queue, starting download...", id);
                    api::spawn_download_task(
                        id,
                        url,
                        token,
                        queue_state.download_dir.clone(),
                        queue_state.transfer_manager.clone(),
                        queue_state.config.clone(),
                        queue_state.plugin_manager.clone(),
                    );
                }
            }
        }
    });

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3001)); // Default port 3001 for Botarr
    tracing::info!("Listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn static_handler(uri: Uri) -> impl IntoResponse {
    let path = uri.path().trim_start_matches('/').to_string();
    let path = if path.is_empty() {
        "index.html".to_string()
    } else {
        path
    };

    match Asset::get(&path) {
        Some(content) => {
            let mime = mime_guess::from_path(&path).first_or_octet_stream();
            ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
        }
        None => {
            if path.contains('.') {
                return StatusCode::NOT_FOUND.into_response();
            }
            // Fallback to index.html for SPA routing
            match Asset::get("index.html") {
                Some(content) => {
                    let mime = mime_guess::from_path("index.html").first_or_octet_stream();
                    ([(header::CONTENT_TYPE, mime.as_ref())], content.data).into_response()
                }
                None => StatusCode::NOT_FOUND.into_response(),
            }
        }
    }
}
