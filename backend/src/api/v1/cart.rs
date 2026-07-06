//! Cart endpoints — customer-only. All routes require auth.

use axum::{
    extract::{Path, State},
    routing::{get, patch, post},
    Json, Router,
};
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::domain::cart::{AddCartItemRequest, CartResponse, UpdateCartItemRequest};
use crate::error::AppResult;
use crate::services::cart_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/cart", get(get_cart).delete(clear_cart))
        .route("/cart/items", post(add_item))
        .route("/cart/items/:item_id", patch(update_item).delete(delete_item))
}

async fn get_cart(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Option<CartResponse>>> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| {
        crate::error::AppError::internal("invalid user id in token")
    })?;
    let cart = cart_service::get_cart(&state.db, user_id).await?;
    Ok(Json(cart))
}

async fn clear_cart(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<serde_json::Value>> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| {
        crate::error::AppError::internal("invalid user id in token")
    })?;
    cart_service::clear(&state.db, user_id).await?;
    Ok(Json(serde_json::json!({"cleared": true})))
}

async fn add_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<AddCartItemRequest>,
) -> AppResult<Json<CartResponse>> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| {
        crate::error::AppError::internal("invalid user id in token")
    })?;
    let cart = cart_service::add_item(&state.db, user_id, req).await?;
    Ok(Json(cart))
}

async fn update_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
    Json(req): Json<UpdateCartItemRequest>,
) -> AppResult<Json<CartResponse>> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| {
        crate::error::AppError::internal("invalid user id in token")
    })?;
    let cart =
        cart_service::update_item(&state.db, user_id, item_id, req.quantity, req.notes).await?;
    Ok(Json(cart))
}

async fn delete_item(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(item_id): Path<Uuid>,
) -> AppResult<Json<CartResponse>> {
    let user_id: Uuid = auth.user_id.parse().map_err(|_| {
        crate::error::AppError::internal("invalid user id in token")
    })?;
    let cart = cart_service::delete_item(&state.db, user_id, item_id).await?;
    Ok(Json(cart))
}
