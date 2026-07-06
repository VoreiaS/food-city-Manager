//! Payment repository.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::payment::PaymentStatus;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub order_id: Uuid,
    pub provider: String,
    pub provider_intent_id: Option<String>,
    pub idempotency_key: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: PaymentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub async fn find_by_idempotency_key(
    db: &PgPool,
    key: &str,
) -> SqlxResult<Option<PaymentIntent>> {
    sqlx::query_as::<_, PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE idempotency_key = $1
        "#,
    )
    .bind(key)
    .fetch_optional(db)
    .await
}

pub async fn find_by_order(db: &PgPool, order_id: Uuid) -> SqlxResult<Option<PaymentIntent>> {
    sqlx::query_as::<_, PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE order_id = $1
        ORDER BY created_at DESC
        LIMIT 1
        "#,
    )
    .bind(order_id)
    .fetch_optional(db)
    .await
}

pub async fn insert(
    db: &PgPool,
    id: Uuid,
    order_id: Uuid,
    provider: &str,
    idempotency_key: &str,
    amount_cents: i64,
    currency: &str,
    status: PaymentStatus,
    provider_intent_id: Option<&str>,
) -> SqlxResult<PaymentIntent> {
    sqlx::query_as::<_, PaymentIntent>(
        r#"
        INSERT INTO payment_intents (id, order_id, provider, provider_intent_id, idempotency_key, amount_cents, currency, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, order_id, provider, provider_intent_id, idempotency_key, amount_cents, currency,
                  status as "status", created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(order_id)
    .bind(provider)
    .bind(provider_intent_id)
    .bind(idempotency_key)
    .bind(amount_cents)
    .bind(currency)
    .bind(status)
    .fetch_one(db)
    .await
}

pub async fn set_status(
    db: &PgPool,
    id: Uuid,
    status: PaymentStatus,
    provider_intent_id: Option<&str>,
) -> SqlxResult<PaymentIntent> {
    sqlx::query_as::<_, PaymentIntent>(
        r#"
        UPDATE payment_intents
        SET status = $2,
            provider_intent_id = COALESCE($3, provider_intent_id),
            updated_at = NOW()
        WHERE id = $1
        RETURNING id, order_id, provider, provider_intent_id, idempotency_key, amount_cents, currency,
                  status as "status", created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(provider_intent_id)
    .fetch_one(db)
    .await
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PayoutLedgerEntry {
    pub id: Uuid,
    pub order_id: Uuid,
    pub payee_type: String,
    pub payee_id: Uuid,
    pub amount_cents: i64,
    pub currency: String,
    pub stripe_transfer_id: Option<String>,
    pub status: String,
    pub created_at: DateTime<Utc>,
    pub paid_at: Option<DateTime<Utc>>,
}

pub async fn insert_payout(
    db: &PgPool,
    id: Uuid,
    order_id: Uuid,
    payee_type: &str,
    payee_id: Uuid,
    amount_cents: i64,
    currency: &str,
) -> SqlxResult<()> {
    sqlx::query(
        r#"
        INSERT INTO payout_ledger (id, order_id, payee_type, payee_id, amount_cents, currency, status)
        VALUES ($1, $2, $3, $4, $5, $6, 'pending')
        "#,
    )
    .bind(id)
    .bind(order_id)
    .bind(payee_type)
    .bind(payee_id)
    .bind(amount_cents)
    .bind(currency)
    .execute(db)
    .await?;
    Ok(())
}
