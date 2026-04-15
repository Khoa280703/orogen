mod account;
mod api;
mod auth;
mod cli_chat;
mod config;
mod conversation;
mod db;
mod error;
mod grok;
mod middleware;
mod providers;
mod services;

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

use axum::http::HeaderValue;
use clap::{Parser, Subcommand};
use std::sync::LazyLock;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};
use tower_http::set_header::SetResponseHeaderLayer;

// Define header names as static constants
static CONTENT_TYPE: LazyLock<axum::http::HeaderName> =
    LazyLock::new(|| axum::http::HeaderName::from_static("content-type"));
static X_CONTENT_TYPE_OPTIONS: LazyLock<axum::http::HeaderName> =
    LazyLock::new(|| axum::http::HeaderName::from_static("x-content-type-options"));
static X_FRAME_OPTIONS: LazyLock<axum::http::HeaderName> =
    LazyLock::new(|| axum::http::HeaderName::from_static("x-frame-options"));

use crate::account::pool::AccountPool;
use crate::cli_chat::CliChat;
use crate::config::{AppConfig, load_config};
use crate::grok::client::GrokClient;
use crate::providers::ProviderRegistry;

pub struct AppState {
    pub config: AppConfig,
    pub pool: AccountPool,
    pub grok: GrokClient,
    /// Per-API-key request counter
    pub key_request_counts: Arc<RwLock<HashMap<String, u64>>>,
    /// PostgreSQL connection pool
    pub db: sqlx::PgPool,
    /// All valid API keys (from config + database)
    pub api_keys: Arc<RwLock<std::collections::HashSet<String>>>,
    /// Provider registry for future consumer APIs
    pub providers: ProviderRegistry,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            pool: self.pool.clone(),
            grok: self.grok.clone(),
            key_request_counts: self.key_request_counts.clone(),
            db: self.db.clone(),
            api_keys: self.api_keys.clone(),
            providers: self.providers.clone(),
        }
    }
}

#[derive(Parser)]
#[command(name = "grok-local", version = "1.0.0")]
#[command(about = "Local Grok chat proxy with OpenAI-compatible API")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start interactive chat with Grok (default)
    Chat {
        /// Resume a conversation by ID
        #[arg(short, long)]
        resume: Option<String>,
    },
    /// Start the API server
    Serve {
        #[arg(short, long)]
        port: Option<u16>,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _ = dotenvy::dotenv();
    let cli = Cli::parse();

    // Only show logs for serve mode; chat mode uses stderr for daemon only
    let log_level = match &cli.command {
        Some(Commands::Serve { .. }) => tracing::Level::INFO,
        _ => tracing::Level::WARN,
    };
    tracing_subscriber::fmt().with_max_level(log_level).init();

    match cli.command {
        Some(Commands::Serve { port }) => {
            start_server(load_config(), port).await?;
        }
        Some(Commands::Chat { resume }) => {
            let mut chat = CliChat::new(resume.as_deref()).await;
            chat.start().await;
        }
        None => {
            let mut chat = CliChat::new(None).await;
            chat.start().await;
        }
    }

    Ok(())
}

async fn start_server(
    config: AppConfig,
    port_override: Option<u16>,
) -> Result<(), Box<dyn std::error::Error>> {
    let port = port_override.unwrap_or(config.api_port);

    // DATABASE_URL is required
    let url = config
        .database_url
        .as_ref()
        .ok_or("DATABASE_URL is required. Set it in .env or environment.")?;

    let db = db::init_pool(url).await?;
    tracing::info!(
        "Connected to PostgreSQL: {}",
        url.split('@').last().unwrap_or(url)
    );

    // Run migrations
    if let Err(e) = db::migrate::run_migrations(url).await {
        tracing::warn!("Migration warning: {}", e);
    }

    let pool = AccountPool::new(db.clone());
    let grok = GrokClient::new()
        .await
        .expect("Failed to create Grok client");
    let providers = ProviderRegistry::with_grok(grok.clone());

    // Load API keys from config + database
    let mut api_keys = config.all_keys();
    match crate::db::api_keys::list_api_keys(&db).await {
        Ok(db_keys) => {
            let count = db_keys.len();
            for key in db_keys {
                if key.active.unwrap_or(true) {
                    api_keys.insert(key.key);
                }
            }
            tracing::info!("Loaded {} API keys from database", count);
        }
        Err(e) => {
            tracing::warn!("Failed to load API keys from database: {}", e);
        }
    }
    let configured_key_count = api_keys.len();

    // CORS - permissive for development (no credentials)
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers(Any)
        .expose_headers([]);

    // Security headers
    let content_type = SetResponseHeaderLayer::if_not_present(
        CONTENT_TYPE.clone(),
        HeaderValue::from_static("application/json"),
    );
    let x_content_type_options = SetResponseHeaderLayer::overriding(
        X_CONTENT_TYPE_OPTIONS.clone(),
        HeaderValue::from_static("nosniff"),
    );
    let x_frame_options = SetResponseHeaderLayer::overriding(
        X_FRAME_OPTIONS.clone(),
        HeaderValue::from_static("DENY"),
    );

    let state = AppState {
        config: config.clone(),
        pool,
        grok,
        key_request_counts: Arc::new(RwLock::new(HashMap::new())),
        db,
        api_keys: Arc::new(RwLock::new(api_keys)),
        providers,
    };

    let app = api::router(state.clone())
        .layer(cors)
        .layer(content_type)
        .layer(x_content_type_options)
        .layer(x_frame_options);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Grok Local API server running on http://{addr}");
    tracing::info!("Models: Available from database (GET /v1/models)");
    tracing::info!("API keys configured: {}", configured_key_count);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
