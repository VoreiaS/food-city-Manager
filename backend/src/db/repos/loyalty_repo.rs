//! Loyalty repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::loyalty::LoyaltyTier;

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct LoyaltyAccount {
    pub id: Uuid,
    pub user_id: Uuid,
    pub points_balance: i64,
    pub tier: LoyaltyTier,
    pub lifetime_points: i64,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, serde::Serialize, sqlx::FromRow)]
pub struct LoyaltyTransaction {
    pub id: Uuid,
    pub account_id: Uuid,
    pub points_delta: i64,
    pub reason: String,
    pub order_id: Option<Uuid>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

pub async fn find_by_user(db: &PgPool, user_id: Uuid) -> SqlxResult<Option<LoyaltyAccount>> {
    sqlx::query_as::<_, LoyaltyAccount>(
        r#"
        SELECT id, user_id, points_balance,
               tier as "tier",
               lifetime_points, created_at, updated_at
        FROM loyalty_accounts WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn create_for_user(db: &PgPool, user_id: Uuid) -> SqlxResult<LoyaltyAccount> {
    sqlx::query_as::<_, LoyaltyAccount>(
        r#"
        INSERT INTO loyalty_accounts (user_id) VALUES ($1)
        ON CONFLICT (user_id) DO NOTHING
        RETURNING id, user_id, points_balance,
            tier as "tier",
            lifetime_points, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(db)
    .await
}

pub async fn list_transactions(
    db: &PgPool,
    account_id: Uuid,
    limit: i64,
) -> SqlxResult<Vec<LoyaltyTransaction>> {
    sqlx::query_as::<_, LoyaltyTransaction>(
        r#"
        SELECT id, account_id, points_delta, reason, order_id, created_at
        FROM loyalty_transactions
        WHERE account_id = $1
        ORDER BY created_at DESC
        LIMIT $2
        "#,
    )
    .bind(account_id)
    .bind(limit)
    .fetch_all(db)
    .await
}

/// Add points (positive delta) or redeem (negative delta) atomically.
/// Also updates tier if lifetime_points crosses thresholds.
pub async fn adjust_points(
    db: &PgPool,
    user_id: Uuid,
    points_delta: i64,
    reason: &str,
    order_id: Option<Uuid>,
) -> SqlxResult<LoyaltyAccount> {
    let mut tx = db.begin().await?;

    // Ensure account exists
    let account: LoyaltyAccount = sqlx::query_as::<_, LoyaltyAccount>(
        r#"
        INSERT INTO loyalty_accounts (user_id) VALUES ($1)
        ON CONFLICT (user_id) DO UPDATE SET updated_at = NOW()
        RETURNING id, user_id, points_balance,
            tier as "tier",
            lifetime_points, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await?;

    // Adjust balance (only positive deltas add to lifetime)
    let new_balance = account.points_balance + points_delta;
    let new_lifetime = if points_delta > 0 {
        account.lifetime_points + points_delta
    } else {
        account.lifetime_points
    };
    let new_tier = if new_lifetime >= 10_000 {
        LoyaltyTier::Platinum
    } else if new_lifetime >= 5_000 {
        LoyaltyTier::Gold
    } else {
        LoyaltyTier::Silver
    };

    let updated: LoyaltyAccount = sqlx::query_as::<_, LoyaltyAccount>(
        r#"
        UPDATE loyalty_accounts
        SET points_balance = $2, lifetime_points = $3,
            tier = $4, updated_at = NOW()
        WHERE user_id = $1
        RETURNING id, user_id, points_balance,
            tier as "tier",
            lifetime_points, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(new_balance)
    .bind(new_lifetime)
    .bind(new_tier)
    .fetch_one(&mut *tx)
    .await?;

    // Insert transaction record
    sqlx::query(
        r#"
        INSERT INTO loyalty_transactions (account_id, points_delta, reason, order_id)
        VALUES ($1, $2, $3, $4)
        "#,
    )
    .bind(account.id)
    .bind(points_delta)
    .bind(reason)
    .bind(order_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(updated)
}
