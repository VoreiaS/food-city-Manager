//! Admin endpoints — admin role only.

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::{driver_repo, order_repo, restaurant_repo};
use crate::domain::restaurant::RestaurantStatus;
use crate::error::{AppError, AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/admin/live/orders", get(live_orders))
        .route("/admin/live/drivers", get(live_drivers))
        .route("/admin/live/restaurants", get(live_restaurants))
        .route("/admin/orders/:id/reassign", post(reassign_order))
        .route("/admin/restaurants/:id/status", post(set_restaurant_status))
        .route("/admin/analytics/summary", get(analytics_summary))
        .route("/admin/verifications", get(list_verifications))
        .route("/admin/verifications/:id/approve", post(approve_verification))
        .route("/admin/verifications/:id/reject", post(reject_verification))
}

fn require_admin(auth: &AuthUser) -> AppResult<()> {
    if auth.role != crate::domain::user::UserRole::Admin {
        return Err(AppError::forbidden("admin role required"));
    }
    Ok(())
}

#[derive(Serialize, sqlx::FromRow)]
struct LiveOrderRow {
    id: Uuid,
    restaurant_name: String,
    status: String,
    total_cents: i64,
    placed_at: chrono::DateTime<chrono::Utc>,
    driver_id: Option<Uuid>,
}

async fn live_orders(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<LiveOrderRow>>> {
    require_admin(&auth)?;
    let rows: Vec<LiveOrderRow> = sqlx::query_as::<_, LiveOrderRow>(
        r#"
        SELECT o.id, r.name as restaurant_name,
               o.status::text as status,
               o.total_cents, o.placed_at, o.driver_id
        FROM orders o
        JOIN restaurants r ON r.id = o.restaurant_id
        WHERE o.status IN ('pending_accept', 'accepted', 'preparing', 'ready', 'picked_up', 'delivering')
        ORDER BY o.placed_at DESC
        LIMIT 200
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Serialize)]
struct LiveDriverRow {
    id: Uuid,
    user_id: Uuid,
    status: String,
    current_lat: Option<f64>,
    current_lng: Option<f64>,
    current_order_id: Option<Uuid>,
    rating_avg: Option<f64>,
    total_deliveries: i64,
}

async fn live_drivers(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<LiveDriverRow>>> {
    require_admin(&auth)?;
    let drivers = driver_repo::list_online(&state.db).await?;
    let rows: Vec<LiveDriverRow> = drivers
        .into_iter()
        .map(|d| LiveDriverRow {
            id: d.id,
            user_id: d.user_id,
            status: format!("{:?}", d.status).to_lowercase(),
            current_lat: d.current_lat,
            current_lng: d.current_lng,
            current_order_id: d.current_order_id,
            rating_avg: d.rating_avg,
            total_deliveries: d.total_deliveries,
        })
        .collect();
    Ok(Json(rows))
}

#[derive(Serialize, sqlx::FromRow)]
struct LiveRestaurantRow {
    id: Uuid,
    name: String,
    status: String,
    rating_avg: Option<f64>,
    rating_count: i64,
}

async fn live_restaurants(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<LiveRestaurantRow>>> {
    require_admin(&auth)?;
    let rows: Vec<LiveRestaurantRow> = sqlx::query_as::<_, LiveRestaurantRow>(
        r#"
        SELECT id, name, status::text as status, rating_avg, rating_count
        FROM restaurants
        WHERE deleted_at IS NULL AND status != 'pending_verification'
        ORDER BY name
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

#[derive(Deserialize)]
struct ReassignRequest {
    driver_id: Uuid,
}

async fn reassign_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(order_id): Path<Uuid>,
    Json(req): Json<ReassignRequest>,
) -> AppResult<Json<serde_json::Value>> {
    require_admin(&auth)?;
    let driver = driver_repo::find_by_id(&state.db, req.driver_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver"))?;
    if driver.status != crate::domain::driver::DriverStatus::Available {
        return Err(AppError::conflict("driver is not available"));
    }
    let _ = order_repo::assign_driver(&state.db, order_id, req.driver_id).await?;
    driver_repo::set_current_order(&state.db, req.driver_id, Some(order_id)).await?;
    let _ = driver_repo::transition_status(
        &state.db,
        req.driver_id,
        crate::domain::driver::DriverStatus::Available,
        crate::domain::driver::DriverStatus::Assigned,
    )
    .await;
    Ok(Json(serde_json::json!({"reassigned": true, "driver_id": req.driver_id})))
}

#[derive(Deserialize)]
struct SetStatusRequest {
    status: RestaurantStatus,
}

async fn set_restaurant_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(restaurant_id): Path<Uuid>,
    Json(req): Json<SetStatusRequest>,
) -> AppResult<Json<crate::domain::restaurant::Restaurant>> {
    require_admin(&auth)?;
    let updated = restaurant_repo::update_status(&state.db, restaurant_id, req.status).await?;
    Ok(Json(updated))
}

#[derive(Serialize)]
struct AnalyticsSummary {
    total_orders: i64,
    active_orders: i64,
    delivered_orders: i64,
    canceled_orders: i64,
    total_customers: i64,
    total_restaurants: i64,
    total_drivers: i64,
    gmv_cents: i64,
    avg_order_value_cents: i64,
}

async fn analytics_summary(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<AnalyticsSummary>> {
    require_admin(&auth)?;
    let db = &state.db;
    let total_orders: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM orders")
        .fetch_one(db)
        .await?;
    let active_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM orders WHERE status IN ('pending_accept','accepted','preparing','ready','picked_up','delivering')",
    )
    .fetch_one(db)
    .await?;
    let delivered_orders: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM orders WHERE status = 'delivered'")
            .fetch_one(db)
            .await?;
    let canceled_orders: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM orders WHERE status IN ('canceled', 'auto_rejected')",
    )
    .fetch_one(db)
    .await?;
    let total_customers: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM users WHERE role = 'customer' AND deleted_at IS NULL")
            .fetch_one(db)
            .await?;
    let total_restaurants: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM restaurants WHERE deleted_at IS NULL AND status != 'pending_verification'",
    )
    .fetch_one(db)
    .await?;
    let total_drivers: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM drivers").fetch_one(db).await?;
    let gmv_cents: i64 = sqlx::query_scalar(
        "SELECT COALESCE(SUM(total_cents), 0) FROM orders WHERE status = 'delivered'",
    )
    .fetch_one(db)
    .await?;
    let avg_order_value_cents: i64 = if delivered_orders > 0 {
        gmv_cents / delivered_orders
    } else {
        0
    };

    Ok(Json(AnalyticsSummary {
        total_orders,
        active_orders,
        delivered_orders,
        canceled_orders,
        total_customers,
        total_restaurants,
        total_drivers,
        gmv_cents,
        avg_order_value_cents,
    }))
}

// --- Restaurant verifications ---

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
struct VerificationRow {
    id: Uuid,
    restaurant_id: Uuid,
    restaurant_name: String,
    status: String,
    documents: serde_json::Value,
    reviewed_by: Option<Uuid>,
    reviewed_at: Option<chrono::DateTime<chrono::Utc>>,
    notes: Option<String>,
    created_at: chrono::DateTime<chrono::Utc>,
}

async fn list_verifications(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<VerificationRow>>> {
    require_admin(&auth)?;
    let rows = sqlx::query_as::<_, VerificationRow>(
        r#"
        SELECT v.id, v.restaurant_id, r.name as restaurant_name,
               v.status::text as status, v.documents,
               v.reviewed_by, v.reviewed_at, v.notes, v.created_at
        FROM restaurant_verifications v
        JOIN restaurants r ON r.id = v.restaurant_id
        WHERE v.status = 'pending'
        ORDER BY v.created_at ASC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.db)
    .await?;
    Ok(Json(rows))
}

async fn approve_verification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(verification_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    require_admin(&auth)?;
    let admin_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    // Update verification + restaurant status
    let restaurant_id: Uuid = sqlx::query_scalar(
        r#"
        UPDATE restaurant_verifications
        SET status = 'approved', reviewed_by = $2, reviewed_at = NOW(), updated_at = NOW()
        WHERE id = $1 AND status = 'pending'
        RETURNING restaurant_id
        "#,
    )
    .bind(verification_id)
    .bind(admin_id)
    .fetch_one(&state.db)
    .await?;

    sqlx::query("UPDATE restaurants SET status = 'active', updated_at = NOW() WHERE id = $1")
        .bind(restaurant_id)
        .execute(&state.db)
        .await?;

    tracing::info!(
        verification_id = %verification_id,
        restaurant_id = %restaurant_id,
        admin_id = %admin_id,
        "restaurant verification approved"
    );

    Ok(Json(serde_json::json!({
        "approved": true,
        "restaurant_id": restaurant_id,
    })))
}

#[derive(Deserialize)]
struct RejectVerificationRequest {
    notes: String,
}

async fn reject_verification(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(verification_id): Path<Uuid>,
    Json(req): Json<RejectVerificationRequest>,
) -> AppResult<Json<serde_json::Value>> {
    require_admin(&auth)?;
    let admin_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    let restaurant_id: Uuid = sqlx::query_scalar(
        r#"
        UPDATE restaurant_verifications
        SET status = 'rejected', reviewed_by = $2, reviewed_at = NOW(), notes = $3, updated_at = NOW()
        WHERE id = $1 AND status = 'pending'
        RETURNING restaurant_id
        "#,
    )
    .bind(verification_id)
    .bind(admin_id)
    .bind(&req.notes)
    .fetch_one(&state.db)
    .await?;

    tracing::info!(
        verification_id = %verification_id,
        restaurant_id = %restaurant_id,
        admin_id = %admin_id,
        "restaurant verification rejected"
    );

    Ok(Json(serde_json::json!({
        "rejected": true,
        "restaurant_id": restaurant_id,
        "notes": req.notes,
    })))
}
