use axum::{
    extract::{Path, State},
    routing::{delete, get},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::address_repo::{self, Address, NewAddress};
use crate::error::{AppError, AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/addresses", get(list_addresses).post(create_address))
        .route("/addresses/:id", delete(delete_address))
}

#[derive(Debug, Deserialize)]
pub struct CreateAddressRequest {
    pub label: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub postal_code: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub formatted_address: String,
    #[serde(default)]
    pub is_default: bool,
}

async fn list_addresses(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<Address>>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let addresses = address_repo::list_for_user(&state.db, user_id).await?;
    Ok(Json(addresses))
}

async fn create_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateAddressRequest>,
) -> AppResult<Json<Address>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    // Validate lat/lng ranges
    if !(-90.0..=90.0).contains(&req.lat) {
        return Err(AppError::validation("latitude must be between -90 and 90"));
    }
    if !(-180.0..=180.0).contains(&req.lng) {
        return Err(AppError::validation("longitude must be between -180 and 180"));
    }
    if req.label.trim().is_empty() {
        return Err(AppError::validation("label is required"));
    }
    if req.line1.trim().is_empty() {
        return Err(AppError::validation("address line 1 is required"));
    }
    if req.city.trim().is_empty() {
        return Err(AppError::validation("city is required"));
    }

    let addr = address_repo::create(
        &state.db,
        NewAddress {
            user_id,
            label: req.label,
            line1: req.line1,
            line2: req.line2,
            city: req.city,
            postal_code: req.postal_code,
            lat: req.lat,
            lng: req.lng,
            formatted_address: req.formatted_address,
            is_default: req.is_default,
        },
    )
    .await?;
    Ok(Json(addr))
}

async fn delete_address(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(id): Path<Uuid>,
) -> AppResult<Json<serde_json::Value>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    address_repo::delete(&state.db, id, user_id).await?;
    Ok(Json(serde_json::json!({"deleted": true})))
}
