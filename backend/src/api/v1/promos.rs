//! Promo code endpoints — customer validate + admin CRUD.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::promo_repo;
use crate::error::{AppError, AppResult};
use crate::services::promo_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        // Customer: validate a promo code before checkout
        .route("/promos/validate", post(validate_promo))
        // Admin: CRUD
        .route("/admin/promos", get(list_promos).post(create_promo))
        .route("/admin/promos/:id", get(get_promo).patch(update_promo))
        .route("/admin/promos/:id/deactivate", post(deactivate_promo))
}

#[derive(Deserialize)]
struct ValidatePromoRequest {
    code: String,
    order_subtotal_cents: i64,
    restaurant_id: Uuid,
}

#[derive(Serialize)]
struct ValidatePromoResponse {
    valid: bool,
    code: String,
    discount_type: String,
    discount_cents: i64,
    description: Option<String>,
}

async fn validate_promo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<ValidatePromoRequest>,
) -> AppResult<Json<ValidatePromoResponse>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let v = promo_service::validate(
        &state.db,
        &req.code,
        user_id,
        req.order_subtotal_cents,
        req.restaurant_id,
    )
    .await?;
    Ok(Json(ValidatePromoResponse {
        valid: true,
        code: v.code,
        discount_type: v.discount_type,
        discount_cents: v.discount_cents,
        description: v.description,
    }))
}

async fn list_promos(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<Vec<promo_repo::PromoCode>>> {
    require_admin(&auth)?;
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(50).clamp(1, 200);
    let offset = ((page - 1) * page_size) as i64;
    let promos = promo_repo::list_all(&state.db, page_size as i64, offset).await?;
    Ok(Json(promos))
}

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    page_size: Option<u32>,
}

async fn get_promo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<promo_repo::PromoCode>> {
    require_admin(&auth)?;
    let promo = promo_repo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::not_found("promo code"))?;
    Ok(Json(promo))
}

#[derive(Deserialize)]
struct CreatePromoRequest {
    code: String,
    description: Option<String>,
    discount_type: String, // percent | flat | free_delivery
    discount_value: rust_decimal::Decimal,
    min_order_cents: Option<i64>,
    max_uses: Option<i32>,
    daily_cap: Option<i32>,
    per_user_cap: Option<i32>,
    valid_until: Option<chrono::DateTime<chrono::Utc>>,
    applicable_restaurants: Option<Vec<Uuid>>,
    customer_segment: Option<String>,
}

async fn create_promo(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreatePromoRequest>,
) -> AppResult<Json<promo_repo::PromoCode>> {
    require_admin(&auth)?;

    let code = req.code.trim().to_uppercase();
    if code.is_empty() {
        return Err(AppError::validation("promo code is required"));
    }
    if !["percent", "flat", "free_delivery"].contains(&req.discount_type.as_str()) {
        return Err(AppError::validation("discount_type must be percent, flat, or free_delivery"));
    }

    let promo = promo_repo::create(
        &state.db,
        promo_repo::NewPromoCode {
            code,
            description: req.description,
            discount_type: req.discount_type,
            discount_value: req.discount_value,
            min_order_cents: req.min_order_cents.unwrap_or(0),
            max_uses: req.max_uses,
            daily_cap: req.daily_cap,
            per_user_cap: req.per_user_cap.unwrap_or(1),
            valid_until: req.valid_until,
            applicable_restaurants: req.applicable_restaurants.unwrap_or_default(),
            customer_segment: req.customer_segment.unwrap_or_else(|| "all".to_string()),
        },
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            AppError::conflict("promo code already exists")
        }
        _ => AppError::Database(e),
    })?;
    Ok(Json(promo))
}

#[derive(Deserialize)]
struct UpdatePromoRequest {
    description: Option<String>,
    max_uses: Option<i32>,
    daily_cap: Option<i32>,
    valid_until: Option<chrono::DateTime<chrono::Utc>>,
    active: Option<bool>,
}

async fn update_promo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<UpdatePromoRequest>,
) -> AppResult<Json<serde_json::Value>> {
    require_admin(&auth)?;

    if let Some(active) = req.active {
        promo_repo::set_active(&state.db, id, active).await?;
    }

    // Apply field updates (description, max_uses, daily_cap, valid_until)
    // We use a single UPDATE with COALESCE for atomicity.
    sqlx::query(
        r#"
        UPDATE promo_codes
        SET description = COALESCE($2, description),
            max_uses = COALESCE($3, max_uses),
            daily_cap = COALESCE($4, daily_cap),
            valid_until = COALESCE($5, valid_until),
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(req.description.as_deref())
    .bind(req.max_uses)
    .bind(req.daily_cap)
    .bind(req.valid_until)
    .execute(&state.db)
    .await?;

    let promo = promo_repo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::not_found("promo code"))?;
    Ok(Json(serde_json::json!({
        "id": promo.id,
        "code": promo.code,
        "active": promo.active,
        "used_count": promo.used_count,
        "max_uses": promo.max_uses,
        "daily_cap": promo.daily_cap,
    })))
}

async fn deactivate_promo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    require_admin(&auth)?;
    promo_repo::set_active(&state.db, id, false).await?;
    Ok(Json(serde_json::json!({"deactivated": true})))
}

fn require_admin(auth: &AuthUser) -> AppResult<()> {
    if auth.role != crate::domain::user::UserRole::Admin {
        return Err(AppError::forbidden("admin role required"));
    }
    Ok(())
}
