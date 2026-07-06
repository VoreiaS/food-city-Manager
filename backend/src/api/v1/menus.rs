//! Menu endpoints.

use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use uuid::Uuid;

use crate::api::AppState;
use crate::domain::menu::MenuResponse;
use crate::error::AppResult;
use crate::services::menu_service;

pub fn public_routes() -> Router<AppState> {
    Router::new().route("/restaurants/:id/menu", get(get_menu))
}

async fn get_menu(
    State(state): State<AppState>,
    Path(restaurant_id): Path<Uuid>,
) -> AppResult<Json<MenuResponse>> {
    let menu = menu_service::get_active_menu(&state.db, restaurant_id).await?;
    Ok(Json(menu))
}
