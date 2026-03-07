mod config;
mod db;
mod error;
mod models;
mod r2;
mod routes;

use axum::{Router, http::Method};
use tower_http::cors::{CrosLayer, Any};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load .env in development
    dotenvy::dotenv().ok();

    // Init tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "streamvault_api=debug,tower_http=debug".into()),
            )
        )
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

    // CORS - allow the SvelteKit frontend origin
    let cors = CorsLayer::new()
                .allow_origin(Any)
                .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
                .allow_headers(Any);

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