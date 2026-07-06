//! Order endpoints — customer-side. Restaurant-side transitions live in
//! `restaurant_orders` (Phase 4). Driver transitions in `drivers` (Phase 5).

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::domain::order::{CreateOrderRequest, OrderDto};
use crate::error::{AppError, AppResult};
use crate::services::order_service;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/orders", post(create_order).get(list_orders))
        .route("/orders/:id", get(get_order))
        .route("/orders/:id/cancel", post(cancel_order))
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub order: OrderDto,
    pub payment: PaymentDto,
}

#[derive(Debug, Serialize)]
pub struct PaymentDto {
    pub intent_id: Uuid,
    pub provider_intent_id: Option<String>,
    pub client_secret: Option<String>,
    pub status: String,
    pub amount_cents: i64,
    pub currency: String,
    pub mock_mode: bool,
}

#[derive(Debug, Deserialize)]
pub struct CancelOrderRequest {
    pub reason: String,
}

#[derive(Debug, Deserialize)]
pub struct ListOrdersQuery {
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

async fn create_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateOrderRequest>,
) -> AppResult<Json<CreateOrderResponse>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let result = order_service::place_order(
        &state.db,
        &state.config.stripe.secret_key,
        user_id,
        req,
    )
    .await?;
    Ok(Json(CreateOrderResponse {
        order: result.order,
        payment: PaymentDto {
            intent_id: result.payment.intent_id,
            provider_intent_id: result.payment.provider_intent_id,
            client_secret: result.payment.client_secret,
            status: format!("{:?}", result.payment.status).to_lowercase(),
            amount_cents: result.payment.amount_cents,
            currency: result.payment.currency,
            mock_mode: result.payment.mock_mode,
        },
    }))
}

async fn list_orders(
    State(state): State<AppState>,
    auth: AuthUser,
    Query(q): Query<ListOrdersQuery>,
) -> AppResult<Json<Vec<OrderDto>>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let orders =
        order_service::list_for_customer(&state.db, user_id, q.page.unwrap_or(1), q.page_size.unwrap_or(20))
            .await?;
    Ok(Json(orders))
}

async fn get_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<OrderDto>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let order = order_service::get_order(&state.db, id).await?;
    // Ownership check: only the customer, the assigned driver, the restaurant owner, or admin can view
    let is_owner = order.order.customer_id == user_id;
    let is_driver = order.order.driver_id == Some(user_id);
    let is_admin = auth.role == crate::domain::user::UserRole::Admin;
    let is_restaurant = if auth.role == crate::domain::user::UserRole::Restaurant {
        match crate::db::repos::restaurant_repo::find_by_owner(&state.db, user_id).await? {
            Some(r) => r.id == order.order.restaurant_id,
            None => false,
        }
    } else {
        false
    };
    if !is_owner && !is_driver && !is_admin && !is_restaurant {
        return Err(AppError::forbidden("you don't have access to this order"));
    }
    Ok(Json(order))
}

async fn cancel_order(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
    Json(req): Json<CancelOrderRequest>,
) -> AppResult<Json<OrderDto>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let order = order_service::cancel(&state.db, id, user_id, &req.reason).await?;
    Ok(Json(order))
}
