//! Dispute repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize, sqlx::Type)]
#[sqlx(type_name = "dispute_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DisputeStatus {
    Open,
    Resolved,
    Rejected,
    Escalated,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct Dispute {
    pub id: Uuid,
    pub order_id: Uuid,
    pub customer_id: Uuid,
    pub issue_type: String,
    pub description: String,
    pub evidence_urls: Vec<String>,
    pub status: DisputeStatus,
    pub resolution: Option<String>,
    pub refund_amount_cents: Option<i64>,
    pub resolved_by: Option<Uuid>,
    pub resolved_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

pub async fn create(
    db: &PgPool,
    id: Uuid,
    order_id: Uuid,
    customer_id: Uuid,
    issue_type: &str,
    description: &str,
    evidence_urls: &[String],
) -> SqlxResult<Dispute> {
    sqlx::query_as::<_, Dispute>(
        r#"
        INSERT INTO disputes (id, order_id, customer_id, issue_type, description, evidence_urls)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(order_id)
    .bind(customer_id)
    .bind(issue_type)
    .bind(description)
    .bind(evidence_urls)
    .fetch_one(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<Dispute>> {
    sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        FROM disputes WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_order(db: &PgPool, order_id: Uuid) -> SqlxResult<Option<Dispute>> {
    sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        FROM disputes WHERE order_id = $1
        "#,
    )
    .bind(order_id)
    .fetch_optional(db)
    .await
}

pub async fn list_open(db: &PgPool, limit: i64, offset: i64) -> SqlxResult<Vec<Dispute>> {
    sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        FROM disputes
        WHERE status = 'open'
        ORDER BY created_at ASC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn list_for_customer(
    db: &PgPool,
    customer_id: Uuid,
) -> SqlxResult<Vec<Dispute>> {
    sqlx::query_as::<_, Dispute>(
        r#"
        SELECT id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        FROM disputes
        WHERE customer_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(customer_id)
    .fetch_all(db)
    .await
}

pub async fn resolve(
    db: &PgPool,
    id: Uuid,
    status: DisputeStatus,
    resolution: Option<&str>,
    refund_amount_cents: Option<i64>,
    resolved_by: Uuid,
) -> SqlxResult<Dispute> {
    sqlx::query_as::<_, Dispute>(
        r#"
        UPDATE disputes
        SET status = $2, resolution = $3, refund_amount_cents = $4,
            resolved_by = $5, resolved_at = NOW(), updated_at = NOW()
        WHERE id = $1
        RETURNING id, order_id, customer_id, issue_type, description, evidence_urls,
            status as "status",
            resolution, refund_amount_cents, resolved_by, resolved_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(status)
    .bind(resolution)
    .bind(refund_amount_cents)
    .bind(resolved_by)
    .fetch_one(db)
    .await
}
