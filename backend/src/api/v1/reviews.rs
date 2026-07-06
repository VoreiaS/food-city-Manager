//! Review endpoints.

use axum::{
    extract::{Path, Query, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::api::mw::auth::AuthUser;
use crate::api::AppState;
use crate::db::repos::{order_repo, review_repo, restaurant_repo};
use crate::domain::order::OrderStatus;
use crate::error::{AppError, AppResult};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/reviews", post(create_review))
        .route("/restaurants/:id/reviews", get(list_reviews))
        .route("/reviews/:id/reply", post(reply_review))
}

#[derive(Deserialize)]
struct CreateReviewRequest {
    order_id: Uuid,
    rating_food: i16,
    rating_delivery: i16,
    rating_packaging: i16,
    rating_overall: i16,
    body: Option<String>,
    photo_urls: Option<Vec<String>>,
}

async fn create_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Json(req): Json<CreateReviewRequest>,
) -> AppResult<Json<review_repo::Review>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;

    // Validate ratings
    for r in [
        req.rating_food,
        req.rating_delivery,
        req.rating_packaging,
        req.rating_overall,
    ] {
        if !(1..=5).contains(&r) {
            return Err(AppError::validation("ratings must be between 1 and 5"));
        }
    }

    // Validate order belongs to user + is delivered
    let order = order_repo::find_by_id(&state.db, req.order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    if order.customer_id != user_id {
        return Err(AppError::forbidden("you can only review your own orders"));
    }
    if order.status != OrderStatus::Delivered {
        return Err(AppError::business_rule("can only review delivered orders"));
    }

    // Time window: reviews must be submitted within 30 days of delivery
    if let Some(delivered_at) = order.delivered_at {
        let age_days = (chrono::Utc::now() - delivered_at).num_days();
        if age_days > 30 {
            return Err(AppError::business_rule(
                "reviews must be submitted within 30 days of delivery",
            ));
        }
    }

    // Check no existing review for this order
    if let Some(_existing) = review_repo::find_by_order(&state.db, req.order_id).await? {
        return Err(AppError::conflict("order already reviewed"));
    }

    let id = Uuid::now_v7();
    let photos = req.photo_urls.unwrap_or_default();
    let review = review_repo::insert(
        &state.db,
        id,
        req.order_id,
        user_id,
        order.restaurant_id,
        req.rating_food,
        req.rating_delivery,
        req.rating_packaging,
        req.rating_overall,
        req.body.as_deref(),
        &photos,
    )
    .await?;

    // Recompute restaurant rating
    review_repo::recompute_restaurant_rating(&state.db, order.restaurant_id).await?;

    Ok(Json(review))
}

#[derive(Deserialize)]
struct ListReviewsQuery {
    page: Option<u32>,
    page_size: Option<u32>,
}

async fn list_reviews(
    State(state): State<AppState>,
    Path(restaurant_id): Path<Uuid>,
    Query(q): Query<ListReviewsQuery>,
) -> AppResult<Json<Vec<review_repo::Review>>> {
    let page = q.page.unwrap_or(1).max(1);
    let page_size = q.page_size.unwrap_or(20).clamp(1, 100);
    let offset = ((page - 1) * page_size) as i64;
    let reviews =
        review_repo::list_for_restaurant(&state.db, restaurant_id, page_size as i64, offset)
            .await?;
    Ok(Json(reviews))
}

#[derive(Deserialize)]
struct ReplyRequest {
    reply: String,
}

async fn reply_review(
    State(state): State<AppState>,
    auth: AuthUser,
    Path(review_id): Path<Uuid>,
    Json(req): Json<ReplyRequest>,
) -> AppResult<Json<review_repo::Review>> {
    let user_id: Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id"))?;
    if auth.role != crate::domain::user::UserRole::Restaurant {
        return Err(AppError::forbidden("restaurant role required"));
    }

    let review = review_repo::find_by_id(&state.db, review_id)
        .await?
        .ok_or_else(|| AppError::not_found("review"))?;

    // Verify the restaurant owns this review
    let restaurant = restaurant_repo::find_by_owner(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))?;
    if review.restaurant_id != restaurant.id {
        return Err(AppError::forbidden("review belongs to a different restaurant"));
    }

    let updated = review_repo::set_reply(&state.db, review_id, &req.reply).await?;
    Ok(Json(updated))
}
