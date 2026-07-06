//! Loyalty endpoints.

use axum::{
    extract::State,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::loyalty_repo;
use crate::error::{AppError, AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/loyalty/me", get(get_account))
        .route("/loyalty/me/transactions", get(list_transactions))
}

#[derive(Serialize)]
struct LoyaltyAccountDto {
    points_balance: i64,
    tier: String,
    lifetime_points: i64,
    next_tier_points: i64,
    tier_benefits: Vec<String>,
}

async fn get_account(State(state): State<AppState>, auth: AuthUser) -> AppResult<Json<LoyaltyAccountDto>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    // Auto-create account if missing
    let account = match loyalty_repo::find_by_user(&state.db, user_id).await? {
        Some(a) => a,
        None => {
            loyalty_repo::create_for_user(&state.db, user_id).await?;
            loyalty_repo::find_by_user(&state.db, user_id)
                .await?
                .ok_or_else(|| AppError::internal("loyalty account creation failed"))?
        }
    };

    let (next_tier_points, benefits) = match account.tier {
        crate::domain::loyalty::LoyaltyTier::Silver => (5_000, vec!["Earn 1 point per $1".into()]),
        crate::domain::loyalty::LoyaltyTier::Gold => {
            (10_000, vec!["Earn 1 point per $1".into(), "Free delivery".into()])
        }
        crate::domain::loyalty::LoyaltyTier::Platinum => (
            10_000,
            vec![
                "Earn 1.5 points per $1".into(),
                "Free delivery".into(),
                "Priority support".into(),
                "Exclusive promos".into(),
            ],
        ),
    };

    Ok(Json(LoyaltyAccountDto {
        points_balance: account.points_balance,
        tier: format!("{:?}", account.tier).to_lowercase(),
        lifetime_points: account.lifetime_points,
        next_tier_points,
        tier_benefits: benefits,
    }))
}

async fn list_transactions(
    State(state): State<AppState>,
    auth: AuthUser,
) -> AppResult<Json<Vec<loyalty_repo::LoyaltyTransaction>>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    let account = match loyalty_repo::find_by_user(&state.db, user_id).await? {
        Some(a) => a,
        None => loyalty_repo::create_for_user(&state.db, user_id).await?,
    };
    let txns = loyalty_repo::list_transactions(&state.db, account.id, 50).await?;
    Ok(Json(txns))
}
