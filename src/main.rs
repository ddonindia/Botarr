mod api;
mod config;
mod db;
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

    let state = AppState {
        search_aggregator: Arc::new(SearchAggregator::with_default_providers(None)),
        transfer_manager: Arc::new(RwLock::new(TransferManager::new(download_dir.clone()))),
        download_dir,
        database: Arc::new(database),
        config: Arc::new(RwLock::new(app_config)),
    };

    // Build router
    let app = Router::new()
        .merge(api::routes())
        .fallback(static_handler)
        .with_state(state);

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
