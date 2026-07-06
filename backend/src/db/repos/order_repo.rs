//! Order repository.

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::order::{Order, OrderItem, OrderStatus};
use crate::domain::payment::PaymentStatus;

pub async fn insert(
    db: &PgPool,
    id: Uuid,
    customer_id: Uuid,
    restaurant_id: Uuid,
    snapshot: &serde_json::Value,
    subtotal_cents: i64,
    delivery_fee_cents: i64,
    tax_cents: i64,
    tip_cents: i64,
    discount_cents: i64,
    total_cents: i64,
    currency: &str,
    delivery_address: &serde_json::Value,
    notes: Option<&str>,
    estimated_delivery_at: Option<DateTime<Utc>>,
) -> SqlxResult<Order> {
    sqlx::query_as::<_, Order>(
        r#"
        INSERT INTO orders (
            id, customer_id, restaurant_id, status, payment_status, snapshot,
            subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
            total_cents, currency, delivery_address, notes, estimated_delivery_at
        )
        VALUES ($1, $2, $3, 'pending_accept', 'pending', $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        RETURNING
            id, customer_id, restaurant_id, driver_id,
            status as "status",
            payment_status as "payment_status",
            snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
            total_cents, currency, delivery_address, notes,
            placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
            cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(customer_id)
    .bind(restaurant_id)
    .bind(snapshot)
    .bind(subtotal_cents)
    .bind(delivery_fee_cents)
    .bind(tax_cents)
    .bind(tip_cents)
    .bind(discount_cents)
    .bind(total_cents)
    .bind(currency)
    .bind(delivery_address)
    .bind(notes)
    .bind(estimated_delivery_at)
    .fetch_one(db)
    .await
}

pub async fn insert_item(
    db: &PgPool,
    order_id: Uuid,
    menu_item_id: Option<Uuid>,
    name: &str,
    description: Option<&str>,
    price_cents: i64,
    quantity: i32,
    customizations: &serde_json::Value,
    notes: Option<&str>,
) -> SqlxResult<OrderItem> {
    sqlx::query_as::<_, OrderItem>(
        r#"
        INSERT INTO order_items (order_id, menu_item_id, name, description, price_cents, quantity, customizations, notes, status)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, 'pending')
        RETURNING id, order_id, menu_item_id, name, description, price_cents, quantity, customizations, notes, status
        "#,
    )
    .bind(order_id)
    .bind(menu_item_id)
    .bind(name)
    .bind(description)
    .bind(price_cents)
    .bind(quantity)
    .bind(customizations)
    .bind(notes)
    .fetch_one(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<Order>> {
    sqlx::query_as::<_, Order>(
        r#"
        SELECT id, customer_id, restaurant_id, driver_id,
               status as "status",
               payment_status as "payment_status",
               snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
               total_cents, currency, delivery_address, notes,
               placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
               cancellation_reason, estimated_delivery_at, created_at, updated_at
        FROM orders WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

pub async fn list_for_customer(
    db: &PgPool,
    customer_id: Uuid,
    limit: i64,
    offset: i64,
) -> SqlxResult<Vec<Order>> {
    sqlx::query_as::<_, Order>(
        r#"
        SELECT id, customer_id, restaurant_id, driver_id,
               status as "status",
               payment_status as "payment_status",
               snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
               total_cents, currency, delivery_address, notes,
               placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
               cancellation_reason, estimated_delivery_at, created_at, updated_at
        FROM orders
        WHERE customer_id = $1
        ORDER BY placed_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(customer_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn list_for_restaurant(
    db: &PgPool,
    restaurant_id: Uuid,
    limit: i64,
    offset: i64,
) -> SqlxResult<Vec<Order>> {
    sqlx::query_as::<_, Order>(
        r#"
        SELECT id, customer_id, restaurant_id, driver_id,
               status as "status",
               payment_status as "payment_status",
               snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
               total_cents, currency, delivery_address, notes,
               placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
               cancellation_reason, estimated_delivery_at, created_at, updated_at
        FROM orders
        WHERE restaurant_id = $1
        ORDER BY placed_at DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(restaurant_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(db)
    .await
}

pub async fn items_for_order(db: &PgPool, order_id: Uuid) -> SqlxResult<Vec<OrderItem>> {
    sqlx::query_as::<_, OrderItem>(
        r#"
        SELECT id, order_id, menu_item_id, name, description, price_cents, quantity, customizations, notes, status
        FROM order_items WHERE order_id = $1
        ORDER BY id
        "#,
    )
    .bind(order_id)
    .fetch_all(db)
    .await
}

/// Atomically transition order status. Returns updated row, or None if
/// transition was invalid (status didn't match expected `from`).
pub async fn transition_status(
    db: &PgPool,
    id: Uuid,
    new_status: OrderStatus,
    now: DateTime<Utc>,
) -> SqlxResult<Option<Order>> {
    // For statuses with a dedicated timestamp column, set both that column
    // and updated_at. For others (e.g. AutoRejected), only set updated_at.
    let sql = match new_status {
        OrderStatus::Accepted => r#"
            UPDATE orders SET status = $2, accepted_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('pending_accept')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::Preparing => r#"
            UPDATE orders SET status = $2, preparing_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('accepted')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::Ready => r#"
            UPDATE orders SET status = $2, ready_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('preparing')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::PickedUp => r#"
            UPDATE orders SET status = $2, picked_up_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('ready')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::Delivering => r#"
            UPDATE orders SET status = $2, updated_at = $3
            WHERE id = $1 AND status IN ('picked_up')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::Delivered => r#"
            UPDATE orders SET status = $2, delivered_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('delivering')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        OrderStatus::Canceled | OrderStatus::AutoRejected => r#"
            UPDATE orders SET status = $2, canceled_at = $3, updated_at = $3
            WHERE id = $1 AND status IN ('pending_accept', 'accepted', 'preparing', 'ready', 'picked_up', 'delivering')
            RETURNING id, customer_id, restaurant_id, driver_id,
                status as "status", payment_status as "payment_status",
                snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
                total_cents, currency, delivery_address, notes,
                placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
                cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
        _ => return Ok(None),
    };

    sqlx::query_as::<_, Order>(sql)
        .bind(id)
        .bind(new_status)
        .bind(now)
        .fetch_optional(db)
        .await
}

pub async fn set_payment_status(
    db: &PgPool,
    id: Uuid,
    status: PaymentStatus,
) -> SqlxResult<()> {
    sqlx::query("UPDATE orders SET payment_status = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(status)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_cancellation(
    db: &PgPool,
    id: Uuid,
    reason: &str,
) -> SqlxResult<()> {
    sqlx::query("UPDATE orders SET cancellation_reason = $2, canceled_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(reason)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn assign_driver(
    db: &PgPool,
    id: Uuid,
    driver_id: Uuid,
) -> SqlxResult<Option<Order>> {
    sqlx::query_as::<_, Order>(
        r#"
        UPDATE orders
        SET driver_id = $2, updated_at = NOW()
        WHERE id = $1 AND driver_id IS NULL
        RETURNING
            id, customer_id, restaurant_id, driver_id,
            status as "status",
            payment_status as "payment_status",
            snapshot, subtotal_cents, delivery_fee_cents, tax_cents, tip_cents, discount_cents,
            total_cents, currency, delivery_address, notes,
            placed_at, accepted_at, preparing_at, ready_at, picked_up_at, delivered_at, canceled_at,
            cancellation_reason, estimated_delivery_at, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(driver_id)
    .fetch_optional(db)
    .await
}

/// Append an event to order_events (used for WS replay).
pub async fn append_event(
    db: &PgPool,
    order_id: Uuid,
    sequence: i64,
    event_type: &str,
    payload: &serde_json::Value,
) -> SqlxResult<()> {
    sqlx::query(
        r#"
        INSERT INTO order_events (order_id, sequence, event_type, payload)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT (order_id, sequence) DO NOTHING
        "#,
    )
    .bind(order_id)
    .bind(sequence)
    .bind(event_type)
    .bind(payload)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn next_event_sequence(db: &PgPool, order_id: Uuid) -> SqlxResult<i64> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COALESCE(MAX(sequence), 0) FROM order_events WHERE order_id = $1",
    )
    .bind(order_id)
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| r.0).unwrap_or(0) + 1)
}

pub async fn events_after(
    db: &PgPool,
    order_id: Uuid,
    after_sequence: i64,
    limit: i64,
) -> SqlxResult<Vec<(i64, String, serde_json::Value, chrono::DateTime<Utc>)>> {
    sqlx::query_as(
        r#"
        SELECT sequence, event_type, payload, created_at
        FROM order_events
        WHERE order_id = $1 AND sequence > $2
        ORDER BY sequence ASC
        LIMIT $3
        "#,
    )
    .bind(order_id)
    .bind(after_sequence)
    .bind(limit)
    .fetch_all(db)
    .await
}
