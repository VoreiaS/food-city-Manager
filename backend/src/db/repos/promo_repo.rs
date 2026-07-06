//! Promo code repository.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
#[allow(dead_code)]
pub struct PromoCode {
    pub id: Uuid,
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String, // percent | flat | free_delivery
    pub discount_value: rust_decimal::Decimal,
    pub min_order_cents: i64,
    pub max_uses: Option<i32>,
    pub used_count: i32,
    pub daily_cap: Option<i32>,
    pub per_user_cap: i32,
    pub valid_from: DateTime<Utc>,
    pub valid_until: Option<DateTime<Utc>>,
    pub active: bool,
    pub applicable_restaurants: Vec<Uuid>,
    pub customer_segment: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_code(db: &PgPool, code: &str) -> SqlxResult<Option<PromoCode>> {
    sqlx::query_as::<_, PromoCode>(
        r#"
        SELECT id, code, description, discount_type, discount_value,
               min_order_cents, max_uses, used_count, daily_cap, per_user_cap,
               valid_from, valid_until, active, applicable_restaurants, customer_segment,
               created_at, updated_at
        FROM promo_codes
        WHERE code = $1 AND active = true
        "#,
    )
    .bind(code)
    .fetch_optional(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<PromoCode>> {
    sqlx::query_as::<_, PromoCode>(
        r#"
        SELECT id, code, description, discount_type, discount_value,
               min_order_cents, max_uses, used_count, daily_cap, per_user_cap,
               valid_from, valid_until, active, applicable_restaurants, customer_segment,
               created_at, updated_at
        FROM promo_codes WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn list_all(db: &PgPool, limit: i64, offset: i64) -> SqlxResult<Vec<PromoCode>> {
    sqlx::query_as::<_, PromoCode>(
        r#"
        SELECT id, code, description, discount_type, discount_value,
               min_order_cents, max_uses, used_count, daily_cap, per_user_cap,
               valid_from, valid_until, active, applicable_restaurants, customer_segment,
               created_at, updated_at
        FROM promo_codes
        ORDER BY created_at DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

#[derive(Debug, Clone)]
pub struct NewPromoCode {
    pub code: String,
    pub description: Option<String>,
    pub discount_type: String,
    pub discount_value: rust_decimal::Decimal,
    pub min_order_cents: i64,
    pub max_uses: Option<i32>,
    pub daily_cap: Option<i32>,
    pub per_user_cap: i32,
    pub valid_until: Option<DateTime<Utc>>,
    pub applicable_restaurants: Vec<Uuid>,
    pub customer_segment: String,
}

pub async fn create(db: &PgPool, p: NewPromoCode) -> SqlxResult<PromoCode> {
    sqlx::query_as::<_, PromoCode>(
        r#"
        INSERT INTO promo_codes (code, description, discount_type, discount_value,
            min_order_cents, max_uses, per_user_cap, daily_cap, valid_from, valid_until,
            active, applicable_restaurants, customer_segment)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW(), $9, true, $10, $11)
        RETURNING id, code, description, discount_type, discount_value,
            min_order_cents, max_uses, used_count, daily_cap, per_user_cap,
            valid_from, valid_until, active, applicable_restaurants, customer_segment,
            created_at, updated_at
        "#,
    )
    .bind(&p.code)
    .bind(p.description.as_deref())
    .bind(&p.discount_type)
    .bind(p.discount_value)
    .bind(p.min_order_cents)
    .bind(p.max_uses)
    .bind(p.per_user_cap)
    .bind(p.daily_cap)
    .bind(p.valid_until)
    .bind(&p.applicable_restaurants)
    .bind(&p.customer_segment)
    .fetch_one(db)
    .await
}

/// Atomically increment used_count if under max_uses. Returns true if redemption allowed.
pub async fn try_increment_used_count(
    db: &PgPool,
    promo_id: Uuid,
) -> SqlxResult<bool> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        r#"
        UPDATE promo_codes
        SET used_count = used_count + 1, updated_at = NOW()
        WHERE id = $1
          AND active = true
          AND (max_uses IS NULL OR used_count < max_uses)
          AND (valid_until IS NULL OR valid_until > NOW())
        RETURNING id
        "#,
    )
    .bind(promo_id)
    .fetch_optional(db)
    .await?;
    Ok(row.is_some())
}

/// Decrement used_count (when an order with a promo is canceled).
pub async fn decrement_used_count(db: &PgPool, promo_id: Uuid) -> SqlxResult<()> {
    sqlx::query("UPDATE promo_codes SET used_count = GREATEST(used_count - 1, 0), updated_at = NOW() WHERE id = $1")
        .bind(promo_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_active(db: &PgPool, id: Uuid, active: bool) -> SqlxResult<()> {
    sqlx::query("UPDATE promo_codes SET active = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(active)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PromoRedemption {
    pub id: Uuid,
    pub promo_code_id: Uuid,
    pub user_id: Uuid,
    pub order_id: Option<Uuid>,
    pub redeemed_at: DateTime<Utc>,
}

/// Count how many times a user has redeemed a promo.
pub async fn count_user_redemptions(
    db: &PgPool,
    promo_id: Uuid,
    user_id: Uuid,
) -> SqlxResult<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM promo_redemptions WHERE promo_code_id = $1 AND user_id = $2",
    )
    .bind(promo_id)
    .bind(user_id)
    .fetch_one(db)
    .await?;
    Ok(row.0)
}

/// Count today's redemptions (for daily_cap enforcement).
pub async fn count_today_redemptions(db: &PgPool, promo_id: Uuid) -> SqlxResult<i64> {
    let row: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM promo_redemptions WHERE promo_code_id = $1 AND redeemed_at::date = NOW()::date",
    )
    .bind(promo_id)
    .fetch_one(db)
    .await?;
    Ok(row.0)
}

/// Record a redemption. UNIQUE(promo_code_id, user_id) constraint enforces per_user_cap=1.
/// For per_user_cap > 1, callers should check count_user_redemptions first.
pub async fn record_redemption(
    db: &PgPool,
    promo_id: Uuid,
    user_id: Uuid,
    order_id: Option<Uuid>,
) -> SqlxResult<()> {
    sqlx::query(
        r#"
        INSERT INTO promo_redemptions (promo_code_id, user_id, order_id)
        VALUES ($1, $2, $3)
        ON CONFLICT (promo_code_id, user_id) DO NOTHING
        "#,
    )
    .bind(promo_id)
    .bind(user_id)
    .bind(order_id)
    .execute(db)
    .await?;
    Ok(())
}

// Re-export for callers that need it
pub use chrono::Utc as _Utc;
