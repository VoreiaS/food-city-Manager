//! Driver matching loop — assigns available drivers to pending orders.
//!
//! Strategy (simplified for v1):
//! 1. Find orders in status `ready` (food prepared, waiting for driver) with no driver assigned.
//! 2. For each, find nearby available drivers (using Redis GEO set).
//! 3. Send "offer" via Redis pub/sub on the driver's user channel.
//! 4. Drivers accept via `POST /drivers/orders/:id/accept` (atomic).
//! 5. If no driver accepts within 60s, expand radius and retry.
//!
//! For v1 we auto-assign the nearest available driver to the oldest ready order
//! to keep the demo flowing. Real broadcast model comes in v2.

use deadpool_redis::Pool as RedisPool;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::db::repos::{driver_repo, order_repo, restaurant_repo};
use crate::services::realtime_service;

pub async fn run(db: PgPool, redis: RedisPool) {
    info!("driver_match_loop worker started");
    loop {
        if let Err(e) = tick(&db, &redis).await {
            warn!(error = ?e, "driver_match_loop tick failed");
        }
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}

async fn tick(db: &PgPool, redis: &RedisPool) -> anyhow::Result<()> {
    // Find orders in `ready` state with no driver assigned.
    let pending: Vec<crate::domain::order::Order> = sqlx::query_as::<_, crate::domain::order::Order>(
        r#"
        SELECT id, customer_id, restaurant_id, driver_id,
               status as "status",
               payment_status as "payment_status",
               snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
               total_cents, currency, delivery_address, notes,
               placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
               cancellation_reason, estimated_delivery_at, created_at, updated_at
        FROM orders
        WHERE status = 'ready' AND driver_id IS NULL
        ORDER BY ready_at ASC
        LIMIT 10
        "#,
    )
    .fetch_all(db)
    .await?;

    if pending.is_empty() {
        return Ok(());
    }

    for order in pending {
        // Look up restaurant location
        let restaurant = match restaurant_repo::find_by_id(db, order.restaurant_id).await? {
            Some(r) => r,
            None => continue,
        };

        // Find nearby drivers within 5km
        let candidates =
            match realtime_service::find_nearby_drivers(redis, restaurant.lat, restaurant.lng, 5000.0, 5)
                .await
            {
                Ok(c) => c,
                Err(e) => {
                    warn!(error = ?e, "find_nearby_drivers failed");
                    continue;
                }
            };

        if candidates.is_empty() {
            continue;
        }

        // Try to assign each candidate (first one wins atomically).
        for (driver_id, _dist) in &candidates {
            // Verify driver is still available
            let driver = match driver_repo::find_by_id(db, *driver_id).await? {
                Some(d) if d.status == crate::domain::driver::DriverStatus::Available => d,
                _ => continue,
            };

            // Atomic transition: available → assigned
            let updated = driver_repo::transition_status(
                db,
                driver.id,
                crate::domain::driver::DriverStatus::Available,
                crate::domain::driver::DriverStatus::Assigned,
            )
            .await?;

            if updated.is_some() {
                // Assign driver to order
                if let Some(_order) = order_repo::assign_driver(db, order.id, driver.id).await? {
                    driver_repo::set_current_order(db, driver.id, Some(order.id)).await?;
                    realtime_service::set_driver_unavailable(redis, driver.id).await?;

                    // Append event
                    let seq = order_repo::next_event_sequence(db, order.id).await?;
                    order_repo::append_event(
                        db,
                        order.id,
                        seq,
                        "driver.assigned",
                        &serde_json::json!({"driver_id": driver.id, "auto_assigned": true}),
                    )
                    .await?;

                    info!(
                        order_id = %order.id,
                        driver_id = %driver.id,
                        "auto-assigned driver to order"
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}
