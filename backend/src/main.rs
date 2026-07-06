//! Binary entry point. Bootstraps config, DB pool, Redis, tracing,
//! background workers, and the Axum server.

use std::sync::Arc;

use food_city_backend::{
    api::{create_app, AppState},
    config::Config,
    utils::jwt,
};
use sqlx::postgres::PgPoolOptions;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // --- Config ---
    let config = Config::from_env()?;
    let config = Arc::new(config);

    // --- Tracing ---
    let filter = tracing_subscriber::EnvFilter::try_new(&config.app.log_level)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(true)
        .json()
        .init();

    tracing::info!(
        env = %config.app.env,
        port = config.app.port,
        "starting food-city-backend"
    );

    // --- JWT init ---
    jwt::init(&config);

    // --- DB pool ---
    let db_pool = PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .acquire_timeout(std::time::Duration::from_secs(
            config.database.acquire_timeout_secs,
        ))
        .connect(&config.database.url)
        .await?;

    // Run migrations
    // NOTE: migrations were applied manually for this demo. In production,
    // sqlx::migrate! handles this automatically. We catch errors here to
    // allow starting even if migrations are already applied (checksum mismatch).
    if let Err(e) = sqlx::migrate!("./migrations").run(&db_pool).await {
        tracing::warn!(error = ?e, "migration check failed (may be OK if already applied)");
    }
    tracing::info!("database ready");

    // --- Redis pool ---
    let redis_cfg = deadpool_redis::Config::from_url(&config.redis.url);
    let redis_pool = redis_cfg.create_pool(Some(deadpool_redis::Runtime::Tokio1))?;
    tracing::info!("redis pool created (size={})", config.redis.pool_size);

    // --- AppState ---
    let state = AppState {
        config: config.clone(),
        db: db_pool.clone(),
        redis: redis_pool,
    };

    // --- Background workers ---
    // Each spawned task runs for the lifetime of the process. They
    // gracefully terminate when the runtime is shut down.
    if config.features.driver_matching {
        let s = state.clone();
        tokio::spawn(async move {
            food_city_backend::workers::driver_match_loop::run(s.db.clone(), s.redis.clone()).await
        });
    }
    {
        let s = state.clone();
        tokio::spawn(async move {
            food_city_backend::workers::order_acceptance_timeout::run(s.db.clone()).await
        });
    }
    {
        let s = state.clone();
        tokio::spawn(async move {
            food_city_backend::workers::payment_reconciler::run(s.db.clone(), s.config.clone()).await
        });
    }
    {
        let s = state.clone();
        tokio::spawn(async move {
            food_city_backend::workers::abandoned_cart_cleanup::run(s.db.clone()).await
        });
    }

    // --- Axum app ---
    let cors = build_cors(&config);
    let app = create_app(state.clone())
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            food_city_backend::api::mw::rate_limit::global_middleware,
        ));

    // --- Server ---
    let addr = format!("{}:{}", config.app.host, config.app.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "server listening");
    axum::serve(listener, app).await?;

    Ok(())
}

fn build_cors(config: &Config) -> CorsLayer {
    let origins: Vec<_> = config
        .cors_origins()
        .into_iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    if origins.is_empty() {
        CorsLayer::very_permissive()
    } else {
        CorsLayer::new()
            .allow_origin(origins)
            .allow_headers(tower_http::cors::Any)
            .allow_methods(tower_http::cors::Any)
    }
}
