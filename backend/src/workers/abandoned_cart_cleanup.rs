//! Abandoned cart cleanup — marks stale active carts as abandoned.
//!
//! Carts that haven't been updated in 24 hours are marked `abandoned`.
//! This frees the user to start a new cart without conflict, and keeps
//! the DB clean.

use sqlx::PgPool;
use tracing::info;

const STALE_HOURS: i64 = 24;
const POLL_INTERVAL_SECS: u64 = 3600; // 1 hour

pub async fn run(db: PgPool) {
    info!("abandoned_cart_cleanup worker started ({}h threshold)", STALE_HOURS);
    loop {
        if let Err(e) = tick(&db).await {
            tracing::warn!(error = ?e, "abandoned_cart_cleanup tick failed");
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn tick(db: &PgPool) -> anyhow::Result<()> {
    let result = sqlx::query(
        r#"
        UPDATE carts
        SET status = 'abandoned', updated_at = NOW()
        WHERE status = 'active'
          AND updated_at < NOW() - ($1 || ' hours')::interval
        "#,
    )
    .bind(STALE_HOURS)
    .execute(db)
    .await?;

    let affected = result.rows_affected();
    if affected > 0 {
        info!(count = affected, "marked stale carts as abandoned");
    }
    Ok(())
}
