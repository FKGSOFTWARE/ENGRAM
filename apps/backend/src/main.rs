mod db;
mod llm;
mod models;
mod routes;
mod services;

use axum::Router;
use sqlx::sqlite::SqlitePoolOptions;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use llm::ProviderManager;

pub struct AppState {
    pub db: sqlx::SqlitePool,
    pub llm: ProviderManager,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env file if present
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "engram_backend=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Database setup
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:engram.db?mode=rwc".to_string());

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    // Run migrations
    db::migrate(&pool).await?;

    // Initialize LLM providers
    let llm = ProviderManager::from_env();
    if llm.has_available_provider() {
        tracing::info!("LLM providers available: {:?}", llm.available_providers());
    } else {
        tracing::warn!("No LLM providers configured - evaluation will be unavailable");
    }

    let state = Arc::new(AppState { db: pool, llm });

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .nest("/api", routes::api_router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await?;
    tracing::info!("Server running on http://localhost:3001");
    axum::serve(listener, app).await?;

    Ok(())
}
