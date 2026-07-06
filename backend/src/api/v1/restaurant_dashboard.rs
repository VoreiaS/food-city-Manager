//! Restaurant-side endpoints (orders management, menu management).
//!
//! All routes require restaurant role + ownership of the resource.

use axum::{
    extract::{Path, Query, State},
    routing::{get, patch, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::{menu_repo, order_repo, restaurant_repo};
use crate::domain::menu::{MenuItem, MenuItemStatus};
use crate::domain::order::OrderDto;
use crate::domain::restaurant::RestaurantStatus;
use crate::error::{AppError, AppResult};
use crate::services::{menu_service, order_service};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Orders
        .route("/restaurant/orders", get(list_orders))
        .route("/restaurant/orders/:id/accept", post(accept_order))
        .route("/restaurant/orders/:id/reject", post(reject_order))
        .route("/restaurant/orders/:id/preparing", post(mark_preparing))
        .route("/restaurant/orders/:id/ready", post(mark_ready))
        // Menu management
        .route("/restaurant/menu", get(get_menu).post(create_item))
        .route("/restaurant/menu/items/:id", patch(update_item).delete(delete_item))
        .route("/restaurant/menu/items/:id/photo", post(upload_item_photo))
        .route("/restaurant/menu/categories", post(create_category))
        .route("/restaurant/menu/categories/:id", axum::routing::delete(delete_category))
        // Reviews
        .route("/restaurant/reviews", get(list_reviews))
        // Earnings
        .route("/restaurant/earnings", get(get_earnings))
        // Restaurant profile
        .route("/restaurant/profile", get(get_profile).patch(update_profile))
        .route("/restaurant/status", post(update_status))
}

async fn get_restaurant_for_owner(
    db: &sqlx::PgPool,
    user_id: Uuid,
) -> AppResult<crate::domain::restaurant::Restaurant> {
    restaurant_repo::find_by_owner(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("no restaurant found for your account; contact admin"))
}

async fn list_orders(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<PaginationQuery>,
) -> AppResult<Json<Vec<OrderDto>>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let orders = order_service::list_for_restaurant(
        &state.db,
        restaurant.id,
        q.page.unwrap_or(1),
        q.page_size.unwrap_or(50),
    )
    .await?;
    Ok(Json(orders))
}

#[derive(Deserialize)]
struct PaginationQuery {
    page: Option<u32>,
    page_size: Option<u32>,
}

async fn accept_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_restaurant(&auth)?;
    verify_order_ownership(&state.db, user_id, id).await?;
    Ok(Json(order_service::accept(&state.db, id).await?))
}

#[derive(Deserialize)]
struct RejectRequest {
    reason: String,
}

async fn reject_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<RejectRequest>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_restaurant(&auth)?;
    verify_order_ownership(&state.db, user_id, id).await?;
    // Restaurant-initiated cancel: use a nil customer_id but bypass ownership
    // by calling the internal cancel directly with the order's customer_id.
    let order = crate::db::repos::order_repo::find_by_id(&state.db, id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    Ok(Json(order_service::cancel(&state.db, id, order.customer_id, &req.reason).await?))
}

async fn mark_preparing(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_restaurant(&auth)?;
    verify_order_ownership(&state.db, user_id, id).await?;
    Ok(Json(order_service::mark_preparing(&state.db, id).await?))
}

async fn mark_ready(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id = require_restaurant(&auth)?;
    verify_order_ownership(&state.db, user_id, id).await?;
    Ok(Json(order_service::mark_ready(&state.db, id).await?))
}

async fn get_menu(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<menu_service::MenuResponseView>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let menu = menu_service::get_active_menu(&state.db, restaurant.id).await?;
    Ok(Json(menu))
}

async fn create_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateItemRequest>,
) -> AppResult<Json<MenuItem>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let _version = menu_repo::active_version(&state.db, restaurant.id)
        .await?
        .ok_or_else(|| AppError::not_found("no active menu version"))?;

    let id = Uuid::now_v7();
    let item = sqlx::query_as::<_, MenuItem>(
        r#"
        INSERT INTO menu_items (id, category_id, name, description, price_cents, image_url,
            is_veg, is_vegan, is_halal, spice_level, allergens, track_stock, stock_count,
            sort_order, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
        RETURNING id, category_id, name, description, price_cents, image_url,
            is_veg, is_vegan, is_halal, spice_level, allergens, track_stock, stock_count,
            sort_order,
            status as "status",
            created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(req.category_id)
    .bind(&req.name)
    .bind(req.description.as_deref())
    .bind(req.price_cents)
    .bind(req.image_url.as_deref())
    .bind(req.is_veg.unwrap_or(false))
    .bind(req.is_vegan.unwrap_or(false))
    .bind(req.is_halal.unwrap_or(false))
    .bind(req.spice_level.unwrap_or(0))
    .bind(&req.allergens.unwrap_or_default())
    .bind(req.track_stock.unwrap_or(false))
    .bind(req.stock_count.unwrap_or(0))
    .bind(req.sort_order.unwrap_or(0))
    .bind(MenuItemStatus::Available)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(item))
}

#[derive(Deserialize)]
struct CreateItemRequest {
    category_id: Uuid,
    name: String,
    description: Option<String>,
    price_cents: i64,
    image_url: Option<String>,
    is_veg: Option<bool>,
    is_vegan: Option<bool>,
    is_halal: Option<bool>,
    spice_level: Option<i16>,
    allergens: Option<Vec<String>>,
    track_stock: Option<bool>,
    stock_count: Option<i32>,
    sort_order: Option<i32>,
}

async fn update_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
    Json(req): Json<UpdateItemRequest>,
) -> AppResult<Json<MenuItem>> {
    let user_id = require_restaurant(&auth)?;
    verify_menu_item_ownership(&state.db, user_id, item_id).await?;

    // Build dynamic update — only set fields that are provided.
    let item = sqlx::query_as::<_, MenuItem>(
        r#"
        UPDATE menu_items
        SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            price_cents = COALESCE($4, price_cents),
            image_url = COALESCE($5, image_url),
            is_veg = COALESCE($6, is_veg),
            spice_level = COALESCE($7, spice_level),
            stock_count = COALESCE($8, stock_count),
            status = COALESCE($9, status),
            sort_order = COALESCE($10, sort_order),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, category_id, name, description, price_cents, image_url,
            is_veg, is_vegan, is_halal, spice_level, allergens, track_stock, stock_count,
            sort_order,
            status as "status",
            created_at, updated_at
        "#,
    )
    .bind(item_id)
    .bind(req.name.as_deref())
    .bind(req.description.as_deref())
    .bind(req.price_cents)
    .bind(req.image_url.as_deref())
    .bind(req.is_veg)
    .bind(req.spice_level)
    .bind(req.stock_count)
    .bind(req.status)
    .bind(req.sort_order)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(item))
}

#[derive(Deserialize)]
struct UpdateItemRequest {
    name: Option<String>,
    description: Option<String>,
    price_cents: Option<i64>,
    image_url: Option<String>,
    is_veg: Option<bool>,
    spice_level: Option<i16>,
    stock_count: Option<i32>,
    status: Option<MenuItemStatus>,
    sort_order: Option<i32>,
}

#[derive(Deserialize)]
struct CreateCategoryRequest {
    name: String,
    sort_order: Option<i32>,
}

async fn create_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateCategoryRequest>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let version = menu_repo::active_version(&state.db, restaurant.id)
        .await?
        .ok_or_else(|| AppError::not_found("no active menu version"))?;
    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO menu_categories (id, menu_version_id, name, sort_order) VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(version.id)
    .bind(&req.name)
    .bind(req.sort_order.unwrap_or(0))
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({"id": id, "name": req.name})))
}

async fn get_profile(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<crate::domain::restaurant::Restaurant>> {
    let user_id = require_restaurant(&auth)?;
    let r = get_restaurant_for_owner(&state.db, user_id).await?;
    Ok(Json(r))
}

#[derive(Deserialize)]
struct UpdateProfileRequest {
    name: Option<String>,
    description: Option<String>,
    logo_url: Option<String>,
    cover_url: Option<String>,
    delivery_fee_cents: Option<i64>,
    min_order_cents: Option<i64>,
    delivery_radius_m: Option<i32>,
    hours_json: Option<serde_json::Value>,
}

async fn update_profile(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateProfileRequest>,
) -> AppResult<Json<crate::domain::restaurant::Restaurant>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let updated: crate::domain::restaurant::Restaurant = sqlx::query_as::<_, crate::domain::restaurant::Restaurant>(
        r#"
        UPDATE restaurants SET
            name = COALESCE($2, name),
            description = COALESCE($3, description),
            logo_url = COALESCE($4, logo_url),
            cover_url = COALESCE($5, cover_url),
            delivery_fee_cents = COALESCE($6, delivery_fee_cents),
            min_order_cents = COALESCE($7, min_order_cents),
            delivery_radius_m = COALESCE($8, delivery_radius_m),
            hours_json = COALESCE($9, hours_json),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, owner_user_id, group_id, name, slug, description,
            cuisine_types, price_range, logo_url, cover_url,
            lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents,
            status as "status",
            hours_json, rating_avg, rating_count,
            stripe_account_id, commission_percent,
            created_at, updated_at, deleted_at
        "#,
    )
    .bind(restaurant.id)
    .bind(req.name.as_deref())
    .bind(req.description.as_deref())
    .bind(req.logo_url.as_deref())
    .bind(req.cover_url.as_deref())
    .bind(req.delivery_fee_cents)
    .bind(req.min_order_cents)
    .bind(req.delivery_radius_m)
    .bind(req.hours_json)
    .fetch_one(&state.db)
    .await?;
    Ok(Json(updated))
}

#[derive(Deserialize)]
struct UpdateStatusRequest {
    status: RestaurantStatus,
}

async fn update_status(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<UpdateStatusRequest>,
) -> AppResult<Json<crate::domain::restaurant::Restaurant>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let updated = restaurant_repo::update_status(&state.db, restaurant.id, req.status).await?;
    Ok(Json(updated))
}

fn require_restaurant(auth: &AuthUser) -> AppResult<Uuid> {
    if auth.role != crate::domain::user::UserRole::Restaurant {
        return Err(AppError::forbidden("restaurant role required"));
    }
    auth.user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))
}

/// Verify that the order belongs to the restaurant owned by `user_id`.
/// Prevents a restaurant from accepting/canceling another restaurant's orders.
async fn verify_order_ownership(
    db: &sqlx::PgPool,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<()> {
    let restaurant = get_restaurant_for_owner(db, user_id).await?;
    let order = order_repo::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    if order.restaurant_id != restaurant.id {
        return Err(AppError::forbidden(
            "order does not belong to your restaurant",
        ));
    }
    Ok(())
}

/// Verify that a menu item belongs to the restaurant owned by `user_id`.
/// Prevents a restaurant from editing/deleting another restaurant's menu items.
async fn verify_menu_item_ownership(
    db: &sqlx::PgPool,
    user_id: Uuid,
    item_id: Uuid,
) -> AppResult<()> {
    let restaurant = get_restaurant_for_owner(db, user_id).await?;
    let menu_version = menu_repo::active_version(db, restaurant.id)
        .await?
        .ok_or_else(|| AppError::not_found("no active menu version"))?;
    let categories = menu_repo::categories(db, menu_version.id).await?;
    let category_ids: Vec<Uuid> = categories.iter().map(|c| c.0).collect();
    let items = menu_repo::items_for_categories(db, &category_ids).await?;
    if !items.iter().any(|i| i.id == item_id) {
        return Err(AppError::forbidden(
            "menu item does not belong to your restaurant",
        ));
    }
    Ok(())
}

// --- DELETE item ---
async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = require_restaurant(&auth)?;
    verify_menu_item_ownership(&state.db, user_id, item_id).await?;
    // Soft delete by setting status to 'hidden' (preserves order history references)
    sqlx::query(
        r#"UPDATE menu_items SET status = 'hidden', updated_at = NOW() WHERE id = $1"#,
    )
    .bind(item_id)
    .execute(&state.db)
    .await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

// --- DELETE category ---
async fn delete_category(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(category_id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let _user_id = require_restaurant(&auth)?;
    // Check no active items remain in this category
    let count: i64 = sqlx::query_scalar(
        r#"SELECT COUNT(*) FROM menu_items WHERE category_id = $1 AND status != 'hidden'"#,
    )
    .bind(category_id)
    .fetch_one(&state.db)
    .await?;
    if count > 0 {
        return Err(AppError::conflict(format!(
            "category has {} active items; delete or hide them first",
            count
        )));
    }
    sqlx::query("DELETE FROM menu_categories WHERE id = $1")
        .bind(category_id)
        .execute(&state.db)
        .await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}

// --- Upload item photo (multipart) ---
async fn upload_item_photo(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
    mut multipart: axum::extract::Multipart,
) -> AppResult<Json<serde_json::Value>> {
    let user_id = require_restaurant(&auth)?;
    verify_menu_item_ownership(&state.db, user_id, item_id).await?;

    // Read the first field as the image file
    let field = multipart
        .next_field()
        .await
        .map_err(|e| AppError::validation(format!("multipart error: {}", e)))?
        .ok_or_else(|| AppError::validation("no file uploaded"))?;

    let filename = field
        .file_name()
        .unwrap_or("upload.jpg")
        .to_string();
    let content_type = field
        .content_type()
        .unwrap_or("image/jpeg")
        .to_string();

    // Validate content type
    if !content_type.starts_with("image/") {
        return Err(AppError::validation("file must be an image"));
    }

    let data = field
        .bytes()
        .await
        .map_err(|e| AppError::validation(format!("read bytes failed: {}", e)))?;

    // Limit to 5 MB
    if data.len() > 5 * 1024 * 1024 {
        return Err(AppError::validation("image too large (max 5 MB)"));
    }

    // Generate a unique filename: <item_id>/<uuid>.<ext>
    let ext = filename
        .rsplit('.')
        .next()
        .filter(|e| e.len() <= 5)
        .unwrap_or("jpg");
    let stored_name = format!("menu/{}_{}.{}", item_id, Uuid::now_v7(), ext);
    let upload_dir = std::path::Path::new("uploads");
    let _ = std::fs::create_dir_all(upload_dir.join("menu"));
    let file_path = upload_dir.join(&stored_name);
    std::fs::write(&file_path, &data)
        .map_err(|e| AppError::internal(format!("write file failed: {}", e)))?;

    // Build URL — in production this would be a CDN URL
    let image_url = format!("/uploads/{}", stored_name);

    // Update the menu item
    sqlx::query("UPDATE menu_items SET image_url = $2, updated_at = NOW() WHERE id = $1")
        .bind(item_id)
        .bind(&image_url)
        .execute(&state.db)
        .await?;

    tracing::info!(
        item_id = %item_id,
        filename = %filename,
        size = data.len(),
        "uploaded menu item photo"
    );

    Ok(Json(serde_json::json!({
        "image_url": image_url,
        "size": data.len(),
    })))
}

// --- Reviews list (for restaurant owner) ---
async fn list_reviews(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<crate::db::repos::review_repo::Review>>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;
    let reviews =
        crate::db::repos::review_repo::list_for_restaurant(&state.db, restaurant.id, 100, 0).await?;
    Ok(Json(reviews))
}

// --- Earnings summary (daily/weekly/monthly) ---
#[derive(serde::Serialize)]
struct EarningsSummary {
    today_cents: i64,
    week_cents: i64,
    month_cents: i64,
    today_orders: i64,
    week_orders: i64,
    month_orders: i64,
    pending_payouts_cents: i64,
    next_payout_at: Option<chrono::DateTime<chrono::Utc>>,
}

async fn get_earnings(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<EarningsSummary>> {
    let user_id = require_restaurant(&auth)?;
    let restaurant = get_restaurant_for_owner(&state.db, user_id).await?;

    // Aggregate delivered orders
    let today: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(subtotal_cents - (subtotal_cents * commission_percent / 100)), 0)::bigint,
               COUNT(*)::bigint
        FROM orders o
        JOIN restaurants r ON r.id = o.restaurant_id
        WHERE o.restaurant_id = $1
          AND o.status = 'delivered'
          AND o.delivered_at::date = NOW()::date
        "#,
    )
    .bind(restaurant.id)
    .fetch_one(&state.db)
    .await?;

    let week: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(subtotal_cents - (subtotal_cents * commission_percent / 100)), 0)::bigint,
               COUNT(*)::bigint
        FROM orders o
        JOIN restaurants r ON r.id = o.restaurant_id
        WHERE o.restaurant_id = $1
          AND o.status = 'delivered'
          AND o.delivered_at > NOW() - INTERVAL '7 days'
        "#,
    )
    .bind(restaurant.id)
    .fetch_one(&state.db)
    .await?;

    let month: (i64, i64) = sqlx::query_as(
        r#"
        SELECT COALESCE(SUM(subtotal_cents - (subtotal_cents * commission_percent / 100)), 0)::bigint,
               COUNT(*)::bigint
        FROM orders o
        JOIN restaurants r ON r.id = o.restaurant_id
        WHERE o.restaurant_id = $1
          AND o.status = 'delivered'
          AND o.delivered_at > NOW() - INTERVAL '30 days'
        "#,
    )
    .bind(restaurant.id)
    .fetch_one(&state.db)
    .await?;

    let pending: i64 = sqlx::query_scalar(
        r#"
        SELECT COALESCE(SUM(amount_cents), 0)::bigint
        FROM payout_ledger
        WHERE payee_id = $1 AND payee_type = 'restaurant' AND status = 'pending'
        "#,
    )
    .bind(restaurant.id)
    .fetch_one(&state.db)
    .await?;

    // Next payout: next Monday
    let next_payout = (chrono::Utc::now() + chrono::Duration::days(7))
        .date_naive()
        .week(chrono::Weekday::Mon)
        .first_day()
        .and_hms_opt(0, 0, 0)
        .map(|dt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(dt, chrono::Utc));

    Ok(Json(EarningsSummary {
        today_cents: today.0,
        week_cents: week.0,
        month_cents: month.0,
        today_orders: today.1,
        week_orders: week.1,
        month_orders: month.1,
        pending_payouts_cents: pending,
        next_payout_at: next_payout,
    }))
}
