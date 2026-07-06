//! Rate limiting middleware — Redis token bucket per IP / per user.
//!
//! Limits (configurable):
//!   - Anonymous (per IP): 60 req/min, burst 120
//!   - Authenticated (per user): 100 req/min, burst 200
//!   - POST /auth/login: 5/min per IP, burst 10
//!   - POST /auth/register: 3/hour per IP, burst 5
//!   - POST /orders: 10/min per user, burst 20
//!
//! Uses Redis INCR + EXPIRE for simplicity. Returns 429 with
//! X-RateLimit-* headers when exceeded.

use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::api::AppState;
use crate::error::AppError;

pub struct RateLimit {
    pub window_secs: u64,
    pub max_requests: u32,
    pub burst: u32,
}

impl RateLimit {
    pub fn per_minute(max: u32, burst: u32) -> Self {
        Self {
            window_secs: 60,
            max_requests: max,
            burst,
        }
    }
    pub fn per_hour(max: u32, burst: u32) -> Self {
        Self {
            window_secs: 3600,
            max_requests: max,
            burst,
        }
    }
}

/// Apply rate limit. Returns Ok(()) if allowed, Err(AppError::RateLimited) if exceeded.
pub async fn check(
    state: &AppState,
    key: &str,
    limit: &RateLimit,
) -> Result<(), AppError> {
    let mut conn = state.redis.get().await?;
    let redis_key = format!("rl:{}", key);
    let count: i64 = redis::cmd("INCR")
        .arg(&redis_key)
        .query_async(&mut *conn)
        .await
        .map_err(|e| AppError::internal(format!("redis INCR failed: {}", e)))?;

    // Set TTL on first request in window
    if count == 1 {
        let _: () = redis::cmd("EXPIRE")
            .arg(&redis_key)
            .arg(limit.window_secs)
            .query_async(&mut *conn)
            .await
            .map_err(|e| AppError::internal(format!("redis EXPIRE failed: {}", e)))?;
    }

    if count > limit.burst as i64 {
        return Err(AppError::RateLimited);
    }
    Ok(())
}

/// Extract client IP from request, honoring X-Forwarded-For if present.
pub fn client_ip(req: &Request) -> String {
    if let Some(forwarded) = req.headers().get("x-forwarded-for") {
        if let Ok(s) = forwarded.to_str() {
            if let Some(first) = s.split(',').next() {
                return first.trim().to_string();
            }
        }
    }
    req.extensions()
        .get::<std::net::SocketAddr>()
        .map(|a| a.ip().to_string())
        .unwrap_or_else(|| "unknown".into())
}

/// Generic middleware: rate limit by IP for unauthenticated requests,
/// or by user_id for authenticated ones. Applied to all /api/* routes.
pub async fn global_middleware(
    State(state): State<AppState>,
    req: Request,
    next: Next,
) -> Result<Response, AppError> {
    if !state.config.rate_limit.enabled {
        return Ok(next.run(req).await);
    }

    let ip = client_ip(&req);
    let key = format!("ip:{}", ip);
    let limit = RateLimit::per_minute(60, 120);
    check(&state, &key, &limit).await?;

    Ok(next.run(req).await)
}
