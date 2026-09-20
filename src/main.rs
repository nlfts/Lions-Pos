use lions_pos::{
    config::AppConfig,
    database::init_db,
    routes::create_router,
    state::AppState,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "lions_pos=debug,tower_http=debug,axum=trace".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    tracing::info!("Initializing Lions POS backend on port {}", config.port);

    // Initialize database & tables & seed
    let pool = init_db(&config.database_url).await?;
    let state = AppState::new(pool, config.clone());

    let app = create_router(state);

    let bind_addr = format!("{}:{}", config.host, config.port);
    let listener = tokio::net::TcpListener::bind(&bind_addr).await?;

    tracing::info!("🦁 Lion POS API running at http://{}", bind_addr);
    println!("🦁 Lion POS API is live at http://127.0.0.1:{}", config.port);

    axum::serve(listener, app).await?;

    Ok(())
}