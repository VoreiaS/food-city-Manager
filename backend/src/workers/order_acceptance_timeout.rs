//! Order acceptance timeout — auto-rejects orders restaurants don't
//! accept within 5 minutes.
//!
//! Flow:
//! 1. Poll `orders WHERE status = 'pending_accept' AND placed_at < NOW() - 5 min`
//! 2. Transition each to `auto_rejected` + `canceled`
//! 3. Issue full refund via payment_service
//! 4. Append `order.canceled` event for WS fan-out
//! 5. Notify customer (event will trigger frontend toast)

use chrono::Utc;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::db::repos::order_repo;
use crate::domain::order::OrderStatus;
use crate::domain::payment::PaymentStatus;
use crate::services::payment_service;

const TIMEOUT_SECS: i64 = 300; // 5 minutes
const POLL_INTERVAL_SECS: u64 = 60;

pub async fn run(db: PgPool) {
    info!("order_acceptance_timeout worker started ({}s timeout)", TIMEOUT_SECS);
    loop {
        if let Err(e) = tick(&db).await {
            warn!(error = ?e, "order_acceptance_timeout tick failed");
        }
        tokio::time::sleep(std::time::Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn tick(db: &PgPool) -> anyhow::Result<()> {
    // Find timed-out orders
    let timed_out: Vec<crate::domain::order::Order> = sqlx::query_as::<_, crate::domain::order::Order>(
        r#"
        SELECT id, customer_id, restaurant_id, driver_id,
               status as "status",
               payment_status as "payment_status",
               snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
               total_cents, currency, delivery_address, notes,
               placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
               cancellation_reason, estimated_delivery_at, created_at, updated_at
        FROM orders
        WHERE status = 'pending_accept'
          AND placed_at < NOW() - ($1 || ' seconds')::interval
        ORDER BY placed_at ASC
        LIMIT 50
        "#,
    )
    .bind(TIMEOUT_SECS)
    .fetch_all(db)
    .await?;

    if timed_out.is_empty() {
        return Ok(());
    }

    for order in timed_out {
        let age_secs = (Utc::now() - order.placed_at).num_seconds();
        info!(
            order_id = %order.id,
            age_secs = age_secs,
            "auto-rejecting order (restaurant did not accept in time)"
        );

        // Transition to auto_rejected → canceled
        let now = Utc::now();
        let updated = order_repo::transition_status(
            db,
            order.id,
            OrderStatus::AutoRejected,
            now,
        )
        .await?;

        if updated.is_some() {
            order_repo::set_cancellation(db, order.id, "restaurant_acceptance_timeout").await?;

            // Full refund if payment had succeeded
            if order.payment_status == PaymentStatus::Succeeded {
                if let Err(e) = payment_service::refund(
                    db,
                    order.id,
                    order.total_cents,
                    "restaurant_acceptance_timeout",
                )
                .await
                {
                    warn!(error = ?e, order_id = %order.id, "auto-reject refund failed");
                }
            }

            // Append event for WS fan-out
            let seq = order_repo::next_event_sequence(db, order.id).await?;
            order_repo::append_event(
                db,
                order.id,
                seq,
                "order.auto_rejected",
                &serde_json::json!({
                    "reason": "restaurant_acceptance_timeout",
                    "age_secs": age_secs,
                    "refunded": order.payment_status == PaymentStatus::Succeeded,
                }),
            )
            .await?;

            info!(order_id = %order.id, "order auto-rejected and refunded");
        }
    }

    Ok(())
}
