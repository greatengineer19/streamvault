mod config;
mod db;
mod error;
mod models;
mod r2;
mod routes;

use axum::{Router, http::{Method, HeaderValue}};
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env in development
    dotenvy::dotenv().ok();

    // Init tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "streamvault_api=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load config
    let config = config::Config::from_env()?;

    // Connect to Postgres
    let pool = db::connect(&config.database_url).await?;

    // Run migrations
    db::migrate(&pool).await?;

    // Build R2 client
    let r2_client = r2::build_client(&config).await;

    // Build app state
    let state = routes::AppState {
        pool,
        r2_client,
        config: config.clone(),
    };

    // CORS — only allow explicitly listed origins
    // Set ALLOWED_ORIGINS in Railway as comma-separated URLs:
    // e.g. "http://localhost:5173,https://streamvault-lilac.vercel.app"
    let allowed_origins = std::env::var("ALLOWED_ORIGINS")
        .unwrap_or_else(|_| "http://localhost:5173".into());

    let origins: Vec<HeaderValue> = allowed_origins
        .split(',')
        .map(|o| o.trim().parse::<HeaderValue>().expect("Invalid origin"))
        .collect();

    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new()
        .nest("/api", routes::api_router())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = format!("0.0.0.0:{}", config.port);
    tracing::info!("StreamVault API listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}