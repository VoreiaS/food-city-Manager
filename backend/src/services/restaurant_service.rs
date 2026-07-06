//! Restaurant service — search, get by id/slug.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::restaurant_repo;
use crate::domain::restaurant::{
    haversine_m, Restaurant, RestaurantCard, RestaurantQuery,
};
use crate::error::{AppError, AppResult};

pub struct SearchResponse {
    pub data: Vec<RestaurantCard>,
    pub page: u32,
    pub page_size: u32,
    pub total: i64,
}

pub async fn search(db: &PgPool, query: RestaurantQuery) -> AppResult<SearchResponse> {
    let radius_m = query.radius_m.unwrap_or(5000).min(50_000);
    let limit = query.limit();
    let offset = query.offset();
    let now = Utc::now();

    let cuisine = query.cuisine.as_deref();
    let q = query.q.as_deref();

    let rows = restaurant_repo::search(
        db,
        query.lat,
        query.lng,
        radius_m,
        cuisine,
        query.price_range,
        query.rating_min,
        q,
        limit,
        offset,
    )
    .await?;

    // Convert rows → cards, applying distance filter (within delivery_radius_m) and veg filter.
    let cards: Vec<RestaurantCard> = rows
        .into_iter()
        .filter_map(|r| {
            // Skip if outside delivery_radius_m
            if let (Some(lat), Some(lng)) = (query.lat, query.lng) {
                let dist = haversine_m(lat, lng, r.lat, r.lng);
                if dist > r.delivery_radius_m as i64 {
                    return None;
                }
            }
            let restaurant: Restaurant = r.into();
            Some(restaurant)
        })
        .map(|r| {
            let is_open = r.is_open(now);
            let distance_m = if let (Some(lat), Some(lng)) = (query.lat, query.lng) {
                Some(haversine_m(lat, lng, r.lat, r.lng))
            } else {
                None
            };
            RestaurantCard {
                id: r.id,
                name: r.name,
                slug: r.slug,
                description: r.description,
                cuisine_types: r.cuisine_types,
                price_range: r.price_range,
                logo_url: r.logo_url,
                cover_url: r.cover_url,
                delivery_fee_cents: r.delivery_fee_cents,
                min_order_cents: r.min_order_cents,
                rating_avg: r.rating_avg,
                rating_count: r.rating_count,
                status: r.status,
                is_open,
                distance_m,
                delivery_eta_min: Some(estimate_eta_min(distance_m)),
            }
        })
        .collect();

    let total = restaurant_repo::count_search(
        db,
        query.lat,
        query.lng,
        radius_m,
        cuisine,
        query.price_range,
        query.rating_min,
        q,
    )
    .await?;

    Ok(SearchResponse {
        data: cards,
        page: query.page_num(),
        page_size: query.page_size_num(),
        total,
    })
}

fn estimate_eta_min(distance_m: Option<i64>) -> i32 {
    match distance_m {
        Some(d) => {
            // Rough: 5 min prep + 1 min per 250m travel
            let travel = (d / 250).max(5) as i32;
            5 + travel
        }
        None => 35, // default
    }
}

pub async fn get_by_id(db: &PgPool, id: Uuid) -> AppResult<Restaurant> {
    restaurant_repo::find_by_id(db, id)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))
}

pub async fn get_by_slug(db: &PgPool, slug: &str) -> AppResult<Restaurant> {
    restaurant_repo::find_by_slug(db, slug)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))
}

pub async fn list_cuisines(db: &PgPool) -> AppResult<Vec<String>> {
    Ok(restaurant_repo::list_cuisines(db).await?)
}
