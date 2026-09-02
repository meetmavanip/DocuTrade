mod config;
mod errors;
mod state;
pub mod handlers;
pub mod models;
pub mod services;
pub mod middleware;
use axum::{
    routing::get,
    Router,
    Json,
};
use serde_json::json;
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::state::AppState;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,backend=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = Config::from_env();
    tracing::info!("Starting DocuTrade backend on port {}", config.server_port);

    // Note: Database connection setup is mocked to not block execution if DB isn't running yet.
    // In production, we'd use:
    // let pool = PgPoolOptions::new()
    //     .max_connections(5)
    //     .connect(&config.database_url)
    //     .await
    //     .expect("Failed to connect to Postgres");
    // 
    // For scaffolding, we'll create a dummy pool for compilation
    let pool = PgPoolOptions::new().connect_lazy(&config.database_url)?;
    
    // Start background tracking worker
    crate::services::ais_client::start_tracking_worker(pool.clone()).await;

    let state = AppState::new(pool, config.clone());

    // CORS configuration
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build our application with a route
    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", crate::handlers::api_routes(state.clone()))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::extract::DefaultBodyLimit::max(50 * 1024 * 1024))
        .with_state(state);

    // Run our app with hyper, listening globally on the configured port
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server_port));
    tracing::info!("Listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "version": "0.1.0", "service": "DocuTrade API" }))
}
