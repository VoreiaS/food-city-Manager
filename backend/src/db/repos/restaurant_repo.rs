//! Restaurant repository.

use chrono::Utc;
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::restaurant::{Restaurant, RestaurantStatus};

/// Find a single restaurant by ID (not soft-deleted).
pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<Restaurant>> {
    sqlx::query_as::<_, Restaurant>(
        r#"
        SELECT id, owner_user_id, group_id, name, slug, description,
               cuisine_types, price_range, logo_url, cover_url,
               lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents,
               status as "status",
               hours_json, rating_avg, rating_count,
               stripe_account_id, commission_percent,
               created_at, updated_at, deleted_at
        FROM restaurants
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_slug(db: &PgPool, slug: &str) -> SqlxResult<Option<Restaurant>> {
    sqlx::query_as::<_, Restaurant>(
        r#"
        SELECT id, owner_user_id, group_id, name, slug, description,
               cuisine_types, price_range, logo_url, cover_url,
               lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents,
               status as "status",
               hours_json, rating_avg, rating_count,
               stripe_account_id, commission_percent,
               created_at, updated_at, deleted_at
        FROM restaurants
        WHERE slug = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(slug)
    .fetch_optional(db)
    .await
}

/// Search restaurants by location + filters. Returns the full row plus
/// a computed distance_m column from the user's (lat,lng).
pub async fn search(
    db: &PgPool,
    lat: Option<f64>,
    lng: Option<f64>,
    radius_m: i32,
    cuisine: Option<&str>,
    price_range: Option<i16>,
    rating_min: Option<f64>,
    q: Option<&str>,
    limit: i64,
    offset: i64,
) -> SqlxResult<Vec<RestaurantSearchRow>> {
    // Use a dynamic query because of optional filters.
    // sqlx 0.8 doesn't have a great query-builder, so we build SQL with
    // bind guards via numbered placeholders and conditional WHERE clauses.
    let radius_m = radius_m as f64;

    // Approx degrees for radius filter — using haversine post-filter for accuracy.
    // For a tighter filter we'd use PostGIS; this is a coarse pre-filter.
    let deg_lat = if lat.is_some() { radius_m / 111_000.0 } else { 0.0 };
    let deg_lng = if lat.is_some() {
        radius_m / (111_320.0 * (lat.unwrap().to_radians().cos()))
    } else {
        0.0
    };

    let q_pattern = q.map(|s| format!("%{}%", s.to_lowercase()));

    let rows = sqlx::query_as::<_, RestaurantSearchRow>(
        r#"
        SELECT
            r.id, r.owner_user_id, r.group_id, r.name, r.slug, r.description,
            r.cuisine_types, r.price_range, r.logo_url, r.cover_url,
            r.lat, r.lng, r.delivery_radius_m, r.delivery_fee_cents, r.min_order_cents,
            r.status as "status",
            r.hours_json, r.rating_avg, r.rating_count,
            r.stripe_account_id, r.commission_percent,
            r.created_at, r.updated_at, r.deleted_at,
            CASE WHEN $1::float8 IS NULL THEN NULL
                 ELSE (
                   6371000 * 2 * asin(sqrt(
                     power(sin((radians($1 - r.lat)) / 2), 2) +
                     cos(radians($1)) * cos(radians(r.lat)) *
                     power(sin((radians($2 - r.lng)) / 2), 2)
                   ))
                 )
            END as distance_m
        FROM restaurants r
        WHERE r.deleted_at IS NULL
          AND r.status = 'active'
          AND ($1::float8 IS NULL OR
               (r.lat BETWEEN $1 - $3 AND $1 + $3 AND
                r.lng BETWEEN $2 - $4 AND $2 + $4))
          AND ($5::text IS NULL OR $5 = ANY(r.cuisine_types))
          AND ($6::int2 IS NULL OR r.price_range = $6)
          AND ($7::float8 IS NULL OR r.rating_avg >= $7)
          AND ($8::text IS NULL OR
               lower(r.name) LIKE $8 OR
               array_to_string(r.cuisine_types, ',') LIKE $8)
        ORDER BY
          CASE WHEN $1::float8 IS NULL THEN r.rating_avg END DESC NULLS LAST,
          distance_m ASC NULLS LAST,
          r.rating_avg DESC NULLS LAST
        LIMIT $9 OFFSET $10
        "#,
    )
    .bind(lat)
    .bind(lng)
    .bind(deg_lat)
    .bind(deg_lng)
    .bind(cuisine)
    .bind(price_range)
    .bind(rating_min)
    .bind(q_pattern.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await?;

    Ok(rows)
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RestaurantSearchRow {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub group_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub cuisine_types: Vec<String>,
    pub price_range: i16,
    pub logo_url: Option<String>,
    pub cover_url: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub delivery_radius_m: i32,
    pub delivery_fee_cents: i64,
    pub min_order_cents: i64,
    pub status: RestaurantStatus,
    pub hours_json: serde_json::Value,
    pub rating_avg: Option<f64>,
    pub rating_count: i64,
    pub stripe_account_id: Option<String>,
    pub commission_percent: rust_decimal::Decimal,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub deleted_at: Option<chrono::DateTime<Utc>>,
    pub distance_m: Option<f64>,
}

impl From<RestaurantSearchRow> for Restaurant {
    fn from(r: RestaurantSearchRow) -> Self {
        Self {
            id: r.id,
            owner_user_id: r.owner_user_id,
            group_id: r.group_id,
            name: r.name,
            slug: r.slug,
            description: r.description,
            cuisine_types: r.cuisine_types,
            price_range: r.price_range,
            logo_url: r.logo_url,
            cover_url: r.cover_url,
            lat: r.lat,
            lng: r.lng,
            delivery_radius_m: r.delivery_radius_m,
            delivery_fee_cents: r.delivery_fee_cents,
            min_order_cents: r.min_order_cents,
            status: r.status,
            hours_json: r.hours_json,
            rating_avg: r.rating_avg,
            rating_count: r.rating_count,
            stripe_account_id: r.stripe_account_id,
            commission_percent: r.commission_percent,
            created_at: r.created_at,
            updated_at: r.updated_at,
            deleted_at: r.deleted_at,
        }
    }
}

/// Count total restaurants matching the same filters (for pagination).
pub async fn count_search(
    db: &PgPool,
    lat: Option<f64>,
    lng: Option<f64>,
    radius_m: i32,
    cuisine: Option<&str>,
    price_range: Option<i16>,
    rating_min: Option<f64>,
    q: Option<&str>,
) -> SqlxResult<i64> {
    let radius_m = radius_m as f64;
    let deg_lat = if lat.is_some() { radius_m / 111_000.0 } else { 0.0 };
    let deg_lng = if lat.is_some() {
        radius_m / (111_320.0 * (lat.unwrap().to_radians().cos()))
    } else {
        0.0
    };
    let q_pattern = q.map(|s| format!("%{}%", s.to_lowercase()));

    let row: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)
        FROM restaurants r
        WHERE r.deleted_at IS NULL
          AND r.status = 'active'
          AND ($1::float8 IS NULL OR
               (r.lat BETWEEN $1 - $3 AND $1 + $3 AND
                r.lng BETWEEN $2 - $4 AND $2 + $4))
          AND ($5::text IS NULL OR $5 = ANY(r.cuisine_types))
          AND ($6::int2 IS NULL OR r.price_range = $6)
          AND ($7::float8 IS NULL OR r.rating_avg >= $7)
          AND ($8::text IS NULL OR
               lower(r.name) LIKE $8 OR
               array_to_string(r.cuisine_types, ',') LIKE $8)
        "#,
    )
    .bind(lat)
    .bind(lng)
    .bind(deg_lat)
    .bind(deg_lng)
    .bind(cuisine)
    .bind(price_range)
    .bind(rating_min)
    .bind(q_pattern.as_deref())
    .fetch_one(db)
    .await?;

    Ok(row.0)
}

/// Get a restaurant owned by a specific user (for restaurant dashboard).
pub async fn find_by_owner(db: &PgPool, owner_user_id: Uuid) -> SqlxResult<Option<Restaurant>> {
    sqlx::query_as::<_, Restaurant>(
        r#"
        SELECT id, owner_user_id, group_id, name, slug, description,
               cuisine_types, price_range, logo_url, cover_url,
               lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents,
               status as "status",
               hours_json, rating_avg, rating_count,
               stripe_account_id, commission_percent,
               created_at, updated_at, deleted_at
        FROM restaurants
        WHERE owner_user_id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(owner_user_id)
    .fetch_optional(db)
    .await
}

/// Update restaurant status (active / paused / closing / closed).
pub async fn update_status(
    db: &PgPool,
    id: Uuid,
    status: RestaurantStatus,
) -> SqlxResult<Restaurant> {
    sqlx::query_as::<_, Restaurant>(
        r#"
        UPDATE restaurants
        SET status = $2, updated_at = NOW()
        WHERE id = $1 AND deleted_at IS NULL
        RETURNING
            id, owner_user_id, group_id, name, slug, description,
            cuisine_types, price_range, logo_url, cover_url,
            lat, lng, delivery_radius_m, delivery_fee_cents, min_order_cents,
            status as "status",
            hours_json, rating_avg, rating_count,
            stripe_account_id, commission_percent,
            created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(status)
    .fetch_one(db)
    .await
}

/// Find all cuisines currently used by active restaurants (for filter UI).
pub async fn list_cuisines(db: &PgPool) -> SqlxResult<Vec<String>> {
    let rows: Vec<(Option<String>,)> = sqlx::query_as(
        r#"
        SELECT DISTINCT unnest(cuisine_types) as cuisine
        FROM restaurants
        WHERE deleted_at IS NULL AND status = 'active'
        ORDER BY cuisine
        "#,
    )
    .fetch_all(db)
    .await?;
    Ok(rows.into_iter().filter_map(|r| r.0).collect())
}

/// Compute the current UTC time (small helper for tests).
pub fn now_utc() -> chrono::DateTime<Utc> {
    Utc::now()
}
