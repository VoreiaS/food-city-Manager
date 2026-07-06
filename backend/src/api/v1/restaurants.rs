//! Restaurant endpoints (public browsing).

use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::domain::restaurant::{Restaurant, RestaurantCard, RestaurantQuery};
use crate::error::AppResult;
use crate::services::restaurant_service;

pub fn public_routes() -> Router<AppState> {
    Router::new()
        .route("/restaurants", get(list_restaurants))
        .route("/restaurants/cuisines", get(list_cuisines))
        .route("/restaurants/:id", get(get_restaurant))
        .route("/restaurants/by-slug/:slug", get(get_restaurant_by_slug))
}

#[derive(Debug, Serialize)]
pub struct RestaurantListResponse {
    pub data: Vec<RestaurantCard>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

#[derive(Debug, Serialize)]
pub struct RestaurantDetailResponse {
    #[serde(flatten)]
    pub restaurant: Restaurant,
    pub is_open: bool,
}

async fn list_restaurants(
    State(state): State<AppState>,
    Query(q): Query<RestaurantQuery>,
) -> AppResult<Json<RestaurantListResponse>> {
    let result = restaurant_service::search(&state.db, q).await?;
    Ok(Json(RestaurantListResponse {
        data: result.data,
        page: result.page,
        page_size: result.page_size,
        total: result.total,
    }))
}

async fn list_cuisines(State(state): State<AppState>) -> AppResult<Json<Vec<String>>> {
    let cuisines = restaurant_service::list_cuisines(&state.db).await?;
    Ok(Json(cuisines))
}

async fn get_restaurant(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<RestaurantDetailResponse>> {
    let r = restaurant_service::get_by_id(&state.db, id).await?;
    let is_open = r.is_open(chrono::Utc::now());
    Ok(Json(RestaurantDetailResponse { restaurant: r, is_open }))
}

async fn get_restaurant_by_slug(
    State(state): State<AppState>,
    Path(slug): Path<String>,
) -> AppResult<Json<RestaurantDetailResponse>> {
    let r = restaurant_service::get_by_slug(&state.db, &slug).await?;
    let is_open = r.is_open(chrono::Utc::now());
    Ok(Json(RestaurantDetailResponse { restaurant: r, is_open }))
}
