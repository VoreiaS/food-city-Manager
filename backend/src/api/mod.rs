//! HTTP API layer.

pub mod v1;
pub mod mw;

use axum::{
    routing::get,
    Router,
};
use std::sync::Arc;

use crate::config::Config;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: sqlx::PgPool,
    pub redis: deadpool_redis::Pool,
}

pub fn create_app(state: AppState) -> Router {
    let v1 = v1::routes();

    Router::new()
        .route("/health", get(health))
        .route("/ready", get(ready))
        .route("/metrics", get(metrics))
        .nest_service("/uploads", tower_http::services::ServeDir::new("uploads"))
        .nest("/api/v1", v1)
        .with_state(state)
}

async fn health() -> &'static str {
    "ok"
}

async fn ready(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<String, String> {
    sqlx::query("SELECT 1")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
    let mut conn = state.redis.get().await.map_err(|e| e.to_string())?;
    let _: String = redis::cmd("PING")
        .query_async(&mut *conn)
        .await
        .map_err(|e| e.to_string())?;
    Ok("ok".into())
}

/// Prometheus-compatible metrics endpoint.
/// Format: `metric_name{label="value"} value\n`
async fn metrics(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> String {
    let mut out = String::new();

    // DB pool stats
    let pool_size = state.db.size();
    let idle = state.db.num_idle();
    out.push_str(&format!("# TYPE db_pool_size gauge\n"));
    out.push_str(&format!("db_pool_size {}\n", pool_size));
    out.push_str(&format!("# TYPE db_pool_idle gauge\n"));
    out.push_str(&format!("db_pool_idle {}\n", idle));

    // Redis pool stats (deadpool-redis doesn't expose direct metrics; report config)
    out.push_str(&format!("# TYPE redis_pool_configured gauge\n"));
    out.push_str(&format!("redis_pool_configured {}\n", state.config.redis.pool_size));

    // Order state counts
    let rows_result = sqlx::query("SELECT status::text as status, COUNT(*)::bigint as count FROM orders GROUP BY status")
        .fetch_all(&state.db)
        .await;
    if let Ok(rows) = rows_result {
        use sqlx::Row;
        out.push_str("# TYPE order_state_count gauge\n");
        for row in rows {
            let status: String = row.try_get("status").unwrap_or_default();
            let count: i64 = row.try_get("count").unwrap_or(0);
            out.push_str(&format!("order_state_count{{status=\"{}\"}} {}\n", status, count));
        }
    }

    out
}
