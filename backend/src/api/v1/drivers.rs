//! Driver endpoints — driver-role only.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::domain::driver::Driver;
use crate::domain::order::OrderDto;
use crate::error::{AppError, AppResult};
use crate::services::driver_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/drivers/me", get(get_me))
        .route("/drivers/me/online", post(go_online))
        .route("/drivers/me/offline", post(go_offline))
        .route("/drivers/me/location", post(update_location))
        .route("/drivers/me/earnings", get(get_earnings))
        .route("/drivers/orders/:id/accept", post(accept_order))
        .route("/drivers/orders/:id/pickup", post(pickup_order))
        .route("/drivers/orders/:id/deliver", post(deliver_order))
}

fn require_driver(auth: &AuthUser) -> AppResult<Uuid> {
    if auth.role != crate::domain::user::UserRole::Driver {
        return Err(AppError::forbidden("driver role required"));
    }
    auth.user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))
}

async fn get_me(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Driver>> {
    let user_id = require_driver(&auth)?;
    let driver = driver_service::get_profile(&state.db, user_id).await?;
    Ok(Json(driver))
}

#[derive(Deserialize)]
pub struct GoOnlineRequest {
    pub vehicle_type: Option<String>,
}

async fn go_online(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<GoOnlineRequest>,
) -> AppResult<Json<Driver>> {
    let user_id = require_driver(&auth)?;
    let driver =
        driver_service::go_online(&state.db, &state.redis, user_id, req.vehicle_type).await?;
    Ok(Json(driver))
}

async fn go_offline(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<Driver>> {
    let user_id = require_driver(&auth)?;
    let driver = driver_service::go_offline(&state.db, &state.redis, user_id).await?;
    Ok(Json(driver))
}

#[derive(Deserialize)]
pub struct LocationUpdate {
    pub lat: f64,
    pub lng: f64,
    pub heading: Option<f64>,
    pub speed_kph: Option<f64>,
}

async fn update_location(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<LocationUpdate>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = require_driver(&auth)?;

    // Throttle: 1 update per 3 seconds per driver (Redis-based)
    let mut conn = state.redis.get().await
        .map_err(|e| AppError::internal(format!("redis error: {}", e)))?;
    let throttle_key = format!("driver:loc:throttle:{}", user_id);
    let set: Option<String> = redis::cmd("SET")
        .arg(&throttle_key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(3) // 3-second window
        .query_async(&mut *conn)
        .await
        .ok();
    if set.is_none() {
        // Key already exists — throttled. Silently accept but don't process.
        return Ok(Json(serde_json::json!({"updated": false, "reason": "throttled"})));
    }

    driver_service::update_location(
        &state.db,
        &state.redis,
        user_id,
        req.lat,
        req.lng,
        req.heading,
        req.speed_kph,
    )
    .await?;
    Ok(Json(serde_json::json!({"updated": true})))
}

async fn accept_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_driver(&auth)?;
    let order = driver_service::accept_order(&state.db, &state.redis, user_id, order_id).await?;
    Ok(Json(order))
}

async fn pickup_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_driver(&auth)?;
    let order = driver_service::pickup_order(&state.db, &state.redis, user_id, order_id).await?;
    Ok(Json(order))
}

async fn deliver_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_driver(&auth)?;
    let order = driver_service::deliver_order(&state.db, &state.redis, user_id, order_id).await?;
    Ok(Json(order))
}

// --- Driver earnings ---

#[derive(serde::Serialize)]
struct DriverEarnings {
    today_cents: i64,
    week_cents: i64,
    month_cents: i64,
    today_deliveries: i64,
    week_deliveries: i64,
    month_deliveries: i64,
    pending_payouts_cents: i64,
}

async fn get_earnings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<DriverEarnings>> {
    let user_id = require_driver(&auth)?;
    let driver = driver_service::get_profile(&state.db, user_id).await?;

    // Aggregate delivered orders where this driver was assigned
    let today: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(delivery_fee_cents + tip_cents), 0)::bigint,
               COUNT(*)::bigint
        FROM orders
        WHERE driver_id = $1
          AND status = 'delivered'
          AND delivered_at::date = NOW()::date
        "#,
    )
    .bind(driver.id)
    .fetch_one(&state.db)
    .await?;

    let week: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(delivery_fee_cents + tip_cents), 0)::bigint,
               COUNT(*)::bigint
        FROM orders
        WHERE driver_id = $1
          AND status = 'delivered'
          AND delivered_at > NOW() - INTERVAL '7 days'
        "#,
    )
    .bind(driver.id)
    .fetch_one(&state.db)
    .await?;

    let month: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(delivery_fee_cents + tip_cents), 0)::bigint,
               COUNT(*)::bigint
        FROM orders
        WHERE driver_id = $1
          AND status = 'delivered'
          AND delivered_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(driver.id)
    .fetch_one(&state.db)
    .await?;

    let pending: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount_cents), 0)::bigint
        FROM payout_ledger
        WHERE payee_id = $1 AND payee_type = 'driver' AND status = 'pending'
        "#,
    )
    .bind(driver.id)
    .fetch_one(&state.db)
    .await?;

    Ok(Json(DriverEarnings {
        today_cents: today.0,
        week_cents: week.0,
        month_cents: month.0,
        today_deliveries: today.1,
        week_deliveries: week.1,
        month_deliveries: month.1,
        pending_payouts_cents: pending,
    }))
}
