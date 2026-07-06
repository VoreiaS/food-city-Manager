//! Review repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Review {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub restaurant_id: Uuid,
    pub rating_food: i16,
    pub rating_delivery: i16,
    pub rating_packaging: i16,
    pub rating_overall: i16,
    pub body: Option<String>,
    pub photo_urls: Vec<String>,
    pub reply_body: Option<String>,
    pub reply_at: Option<chrono::DateTime<chrono::Utc>>,
    pub is_hidden: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<Review>> {
    sqlx::query_as::<_, Review>(
        r#"
        SELECT id, order_id, customer_id, restaurant_id,
               rating_food, rating_delivery, rating_packaging, rating_overall,
               body, photo_urls, reply_body, reply_at, is_hidden,
               created_at, updated_at
        FROM reviews WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_order(db: &PgPool, order_id: Uuid) -> SqlxResult<Option<Review>> {
    sqlx::query_as::<_, Review>(
        r#"
        SELECT id, order_id, customer_id, restaurant_id,
               rating_food, rating_delivery, rating_packaging, rating_overall,
               body, photo_urls, reply_body, reply_at, is_hidden,
               created_at, updated_at
        FROM reviews WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_optional(db)
    .await
}

pub async fn list_for_restaurant(
    db: &PgPool,
    restaurant_id: Uuid,
    limit: i64,
    offset: i64,
) -> SqlxResult<Vec<Review>> {
    sqlx::query_as::<_, Review>(
        r#"
        SELECT id, order_id, customer_id, restaurant_id,
               rating_food, rating_delivery, rating_packaging, rating_overall,
               body, photo_urls, reply_body, reply_at, is_hidden,
               created_at, updated_at
        FROM reviews
        WHERE restaurant_id = $1 AND is_hidden = false
        ORDER BY created_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(restaurant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn insert(
    db: &PgPool,
    id: Uuid,
    order_id: Uuid,
    customer_id: Uuid,
    restaurant_id: Uuid,
    rating_food: i16,
    rating_delivery: i16,
    rating_packaging: i16,
    rating_overall: i16,
    body: Option<&str>,
    photo_urls: &[String],
) -> SqlxResult<Review> {
    sqlx::query_as::<_, Review>(
        r#"
        INSERT INTO reviews (id, order_id, customer_id, restaurant_id,
            rating_food, rating_delivery, rating_packaging, rating_overall,
            body, photo_urls)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, order_id, customer_id, restaurant_id,
            rating_food, rating_delivery, rating_packaging, rating_overall,
            body, photo_urls, reply_body, reply_at, is_hidden,
            created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(order_id)
    .bind(customer_id)
    .bind(restaurant_id)
    .bind(rating_food)
    .bind(rating_delivery)
    .bind(rating_packaging)
    .bind(rating_overall)
    .bind(body)
    .bind(photo_urls)
    .fetch_one(db)
    .await
}

pub async fn set_reply(
    db: &PgPool,
    id: Uuid,
    reply: &str,
) -> SqlxResult<Review> {
    sqlx::query_as::<_, Review>(
        r#"
        UPDATE reviews
        SET reply_body = $2, reply_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING id, order_id, customer_id, restaurant_id,
            rating_food, rating_delivery, rating_packaging, rating_overall,
            body, photo_urls, reply_body, reply_at, is_hidden,
            created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(reply)
    .fetch_one(db)
    .await
}

/// Recompute and store rating_avg + rating_count for a restaurant.
pub async fn recompute_restaurant_rating(
    db: &PgPool,
    restaurant_id: Uuid,
) -> SqlxResult<()> {
    sqlx::query(
        r#"
        UPDATE restaurants r
        SET rating_avg = sub.avg_rating,
            rating_count = sub.review_count,
            updated_at = NOW()
        FROM (
            SELECT
                AVG(rating_overall)::float8 AS avg_rating,
                COUNT(*)::bigint AS review_count
            FROM reviews
            WHERE restaurant_id = $1 AND is_hidden = false
        ) sub
        WHERE r.id = $1
        "#,
    )
    .bind(restaurant_id)
    .execute(db)
    .await?;
    Ok(())
}
