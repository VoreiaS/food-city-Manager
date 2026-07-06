//! Address repository.

use serde::Serialize;
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Address {
    pub id: Uuid,
    pub user_id: Uuid,
    pub label: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub postal_code: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub formatted_address: String,
    pub is_default: bool,
}

pub async fn list_for_user(db: &PgPool, user_id: Uuid) -> SqlxResult<Vec<Address>> {
    sqlx::query_as::<_, Address>(
        r#"
        SELECT id, user_id, label, line1, line2, city, postal_code,
               lat, lng, formatted_address, is_default
        FROM addresses
        WHERE user_id = $1 AND deleted_at IS NULL
        ORDER BY is_default DESC, created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid, user_id: Uuid) -> SqlxResult<Option<Address>> {
    sqlx::query_as::<_, Address>(
        r#"
        SELECT id, user_id, label, line1, line2, city, postal_code,
               lat, lng, formatted_address, is_default
        FROM addresses
        WHERE id = $1 AND user_id = $2 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await
}

#[derive(Debug, Clone)]
pub struct NewAddress {
    pub user_id: Uuid,
    pub label: String,
    pub line1: String,
    pub line2: Option<String>,
    pub city: String,
    pub postal_code: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub formatted_address: String,
    pub is_default: bool,
}

pub async fn create(db: &PgPool, input: NewAddress) -> SqlxResult<Address> {
    // If is_default, clear other defaults first.
    if input.is_default {
        sqlx::query("UPDATE addresses SET is_default = false WHERE user_id = $1")
            .bind(input.user_id)
            .execute(db)
            .await?;
    }
    sqlx::query_as::<_, Address>(
        r#"
        INSERT INTO addresses (user_id, label, line1, line2, city, postal_code, lat, lng, formatted_address, is_default)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        RETURNING id, user_id, label, line1, line2, city, postal_code, lat, lng, formatted_address, is_default
        "#,
    )
    .bind(input.user_id)
    .bind(input.label)
    .bind(input.line1)
    .bind(input.line2)
    .bind(input.city)
    .bind(input.postal_code)
    .bind(input.lat)
    .bind(input.lng)
    .bind(input.formatted_address)
    .bind(input.is_default)
    .fetch_one(db)
    .await
}

pub async fn delete(db: &PgPool, id: Uuid, user_id: Uuid) -> SqlxResult<()> {
    sqlx::query("UPDATE addresses SET deleted_at = NOW() WHERE id = $1 AND user_id = $2")
        .bind(id)
        .bind(user_id)
        .execute(db)
        .await?;
    Ok(())
}
