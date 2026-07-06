//! Order service — placement, state machine, retrieval, cancellation.

use chrono::{DateTime, Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::{
    address_repo, cart_repo, menu_repo, order_repo, payment_repo, restaurant_repo,
};
use crate::domain::cart::CartStatus;
use crate::domain::order::{
    CreateOrderRequest, OrderDto, OrderItemDto, OrderStatus,
};
use crate::domain::payment::PaymentStatus;
use crate::domain::restaurant::haversine_m;
use crate::error::{AppError, AppResult};
use crate::services::payment_service;
use crate::utils::id::new_idempotency_key;

pub struct CreateOrderResult {
    pub order: OrderDto,
    pub payment: payment_service::PaymentResult,
}

/// Place an order from the user's active cart.
pub async fn place_order(
    db: &PgPool,
    stripe_secret_key: &str,
    customer_id: Uuid,
    req: CreateOrderRequest,
) -> AppResult<CreateOrderResult> {
    // 1. Validate cart
    let cart = cart_repo::find_active(db, customer_id)
        .await?
        .ok_or_else(|| AppError::not_found("no active cart"))?;
    if cart.status != CartStatus::Active {
        return Err(AppError::conflict("cart is not active (locked or converted)"));
    }
    let cart_items = cart_repo::items(db, cart.id).await?;
    if cart_items.is_empty() {
        return Err(AppError::validation("cart is empty"));
    }

    // 2. Validate address
    let address = address_repo::find_by_id(db, req.address_id, customer_id)
        .await?
        .ok_or_else(|| AppError::not_found("address"))?;

    // 3. Validate restaurant (open, delivers to address)
    let restaurant = restaurant_repo::find_by_id(db, cart.restaurant_id)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))?;
    if restaurant.status != crate::domain::restaurant::RestaurantStatus::Active {
        return Err(AppError::business_rule("restaurant is not active"));
    }
    let dist = haversine_m(restaurant.lat, restaurant.lng, address.lat, address.lng);
    if dist > restaurant.delivery_radius_m as i64 {
        return Err(AppError::business_rule(format!(
            "restaurant does not deliver to your address ({}m away, max {}m)",
            dist, restaurant.delivery_radius_m
        )));
    }
    if !restaurant.is_open(Utc::now()) {
        return Err(AppError::business_rule("restaurant is closed"));
    }

    // 4. Snapshot cart items into order_items + compute totals.
    let tip_cents = req.tip_cents.unwrap_or(0).max(0);
    let mut subtotal_cents: i64 = 0;

    // Lock cart so user can't modify during checkout.
    cart_repo::set_status(db, cart.id, CartStatus::Locked).await?;

    let order_id = Uuid::now_v7();
    let mut snapshot_items = Vec::new();

    for ci in &cart_items {
        // Resolve menu item to snapshot current price.
        let menu_item = menu_repo::find_item_by_id(db, ci.menu_item_id)
            .await?
            .ok_or_else(|| {
                AppError::business_rule(format!("menu item {} no longer exists", ci.menu_item_id))
            })?;
        // Verify stock if tracked.
        if menu_item.track_stock && menu_item.stock_count < ci.quantity {
            return Err(AppError::business_rule(format!(
                "only {} of {} in stock (you ordered {})",
                menu_item.stock_count, menu_item.name, ci.quantity
            )));
        }
        // Compute line total including customizations.
        let addon_total: i64 = if ci.customizations.is_null() {
            0
        } else {
            // Customizations stored as [{customization_id, option_id}, ...]
            let sels: Vec<crate::domain::cart::AddCustomizationSelection> =
                serde_json::from_value(ci.customizations.clone()).map_err(|e| {
                    AppError::internal(format!("invalid customizations JSON: {}", e))
                })?;
            let mut total = 0i64;
            for sel in &sels {
                let custs =
                    menu_repo::customizations_for_items(db, &[ci.menu_item_id]).await?;
                if let Some(c) = custs.iter().find(|c| c.id == sel.customization_id) {
                    let opts =
                        menu_repo::options_for_customizations(db, &[c.id]).await?;
                    if let Some(o) = opts.iter().find(|o| o.id == sel.option_id) {
                        total += o.price_cents;
                    }
                }
            }
            total
        };
        let line_total = (menu_item.price_cents + addon_total) * ci.quantity as i64;
        subtotal_cents += line_total;

        // Snapshot
        snapshot_items.push(serde_json::json!({
            "menu_item_id": ci.menu_item_id,
            "name": menu_item.name,
            "description": menu_item.description,
            "price_cents": menu_item.price_cents,
            "quantity": ci.quantity,
            "customizations": ci.customizations,
            "notes": ci.notes,
            "line_total_cents": line_total,
        }));
    }

    let delivery_fee_cents = restaurant.delivery_fee_cents;
    let tax_cents = (subtotal_cents as f64 * 0.0).round() as i64; // No tax for now; configurable later

    // Apply promo code if provided
    let mut discount_cents: i64 = 0;
    let mut promo_validation: Option<crate::services::promo_service::PromoValidation> = None;
    if let Some(code) = &req.promo_code {
        let validation = crate::services::promo_service::validate(
            db,
            code,
            customer_id,
            subtotal_cents,
            restaurant.id,
        )
        .await?;
        discount_cents = if validation.discount_type == "free_delivery" {
            delivery_fee_cents
        } else {
            validation.discount_cents
        };
        promo_validation = Some(validation);
    }

    let total_cents = subtotal_cents + delivery_fee_cents + tax_cents + tip_cents - discount_cents;
    if total_cents < 0 {
        return Err(AppError::validation("total cannot be negative"));
    }

    // Max order value safeguard (prevent $50k orders due to bugs / abuse)
    const MAX_ORDER_CENTS: i64 = 200_000; // $2,000
    if total_cents > MAX_ORDER_CENTS {
        return Err(AppError::validation(format!(
            "order total exceeds maximum (${:.2}). Split into multiple orders.",
            MAX_ORDER_CENTS as f64 / 100.0
        )));
    }

    // Stripe requires minimum charge of $0.50 (50 cents) — if using Stripe
    if total_cents < 50 && !stripe_secret_key.is_empty() {
        return Err(AppError::validation(
            "order total below Stripe minimum ($0.50)",
        ));
    }

    // Min order check
    if subtotal_cents < restaurant.min_order_cents {
        return Err(AppError::business_rule(format!(
            "minimum order is {} cents, your cart is {}",
            restaurant.min_order_cents, subtotal_cents
        )));
    }

    let snapshot = serde_json::json!({
        "restaurant_id": restaurant.id,
        "restaurant_name": restaurant.name,
        "items": snapshot_items,
        "placed_at": Utc::now(),
    });

    let delivery_address_json = serde_json::json!({
        "id": address.id,
        "label": address.label,
        "line1": address.line1,
        "line2": address.line2,
        "city": address.city,
        "postal_code": address.postal_code,
        "lat": address.lat,
        "lng": address.lng,
        "formatted_address": address.formatted_address,
    });

    let eta = Utc::now() + Duration::minutes(45);

    // 5. Insert order
    let _order = order_repo::insert(
        db,
        order_id,
        customer_id,
        restaurant.id,
        &snapshot,
        subtotal_cents,
        delivery_fee_cents,
        tax_cents,
        tip_cents,
        discount_cents,
        total_cents,
        "usd",
        &delivery_address_json,
        req.notes.as_deref(),
        Some(eta),
    )
    .await?;

    // 6. Insert order items (snapshot)
    for ci in &cart_items {
        let menu_item = menu_repo::find_item_by_id(db, ci.menu_item_id).await?.unwrap();
        order_repo::insert_item(
            db,
            order_id,
            Some(ci.menu_item_id),
            &menu_item.name,
            menu_item.description.as_deref(),
            menu_item.price_cents,
            ci.quantity,
            &ci.customizations,
            ci.notes.as_deref(),
        )
        .await?;

        // Decrement stock atomically if tracked.
        if menu_item.track_stock {
            let _ = menu_repo::decrement_stock(db, ci.menu_item_id, ci.quantity).await;
        }
    }

    // 7. Convert cart (no longer active)
    cart_repo::set_status(db, cart.id, CartStatus::Converted).await?;

    // 7b. Redeem promo code if used
    if let Some(validation) = &promo_validation {
        if let Err(e) = crate::services::promo_service::redeem(db, validation.promo_id, customer_id, order_id).await {
            tracing::warn!(error = ?e, order_id = %order_id, "failed to redeem promo code (order still created)");
        }
    }

    // 8. Append order_created event
    let seq = order_repo::next_event_sequence(db, order_id).await?;
    order_repo::append_event(
        db,
        order_id,
        seq,
        "order.created",
        &serde_json::json!({"order_id": order_id, "status": "pending_accept", "discount_cents": discount_cents}),
    )
    .await?;

    // 9. Create payment intent (mock auto-succeeds)
    let idempotency_key = new_idempotency_key();
    let payment = payment_service::create_intent(
        db,
        stripe_secret_key,
        order_id,
        total_cents,
        "usd",
        &idempotency_key,
    )
    .await?;

    // 10. If mock succeeded, append payment event.
    if payment.status == PaymentStatus::Succeeded {
        let seq = order_repo::next_event_sequence(db, order_id).await?;
        order_repo::append_event(
            db,
            order_id,
            seq,
            "payment.succeeded",
            &serde_json::json!({"amount_cents": total_cents}),
        )
        .await?;
    }

    let order_dto = load_order_dto(db, order_id).await?;
    Ok(CreateOrderResult {
        order: order_dto,
        payment,
    })
}

pub async fn get_order(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    load_order_dto(db, order_id).await
}

pub async fn list_for_customer(
    db: &PgPool,
    customer_id: Uuid,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<OrderDto>> {
    let limit = page_size as i64;
    let offset = ((page.max(1) - 1) * page_size) as i64;
    let orders = order_repo::list_for_customer(db, customer_id, limit, offset).await?;
    let mut dtos = Vec::with_capacity(orders.len());
    for o in orders {
        dtos.push(load_order_dto_from_row(db, o).await?);
    }
    Ok(dtos)
}

pub async fn list_for_restaurant(
    db: &PgPool,
    restaurant_id: Uuid,
    page: u32,
    page_size: u32,
) -> AppResult<Vec<OrderDto>> {
    let limit = page_size as i64;
    let offset = ((page.max(1) - 1) * page_size) as i64;
    let orders = order_repo::list_for_restaurant(db, restaurant_id, limit, offset).await?;
    let mut dtos = Vec::with_capacity(orders.len());
    for o in orders {
        dtos.push(load_order_dto_from_row(db, o).await?);
    }
    Ok(dtos)
}

/// Cancel an order. Refund policy based on current state:
/// - pending_accept / accepted: full refund
/// - preparing: 50% refund
/// - ready / picked_up / delivering: no refund
///
/// **Security:** Verifies that `customer_id` owns the order. Only the customer
/// who placed the order can cancel it (admins use a separate admin endpoint).
pub async fn cancel(
    db: &PgPool,
    order_id: Uuid,
    customer_id: Uuid,
    reason: &str,
) -> AppResult<OrderDto> {
    let order = order_repo::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;

    // Ownership check: only the customer who placed the order can cancel
    if order.customer_id != customer_id {
        return Err(AppError::forbidden(
            "you can only cancel your own orders",
        ));
    }

    if !matches!(
        order.status,
        OrderStatus::PendingAccept | OrderStatus::Accepted | OrderStatus::Preparing
            | OrderStatus::Ready | OrderStatus::PickedUp | OrderStatus::Delivering
    ) {
        return Err(AppError::conflict(format!(
            "cannot cancel order in status {:?}",
            order.status
        )));
    }

    let now = Utc::now();
    let updated = order_repo::transition_status(db, order_id, OrderStatus::Canceled, now)
        .await?
        .ok_or_else(|| AppError::conflict("status transition rejected"))?;
    order_repo::set_cancellation(db, order_id, reason).await?;

    // Refund policy
    let refund_amount = match order.status {
        OrderStatus::PendingAccept | OrderStatus::Accepted => order.total_cents,
        OrderStatus::Preparing => order.total_cents / 2,
        _ => 0,
    };
    if refund_amount > 0 && order.payment_status == PaymentStatus::Succeeded {
        payment_service::refund(db, order_id, refund_amount, reason).await?;
    }

    let seq = order_repo::next_event_sequence(db, order_id).await?;
    order_repo::append_event(
        db,
        order_id,
        seq,
        "order.canceled",
        &serde_json::json!({"reason": reason, "refund_cents": refund_amount}),
    )
    .await?;

    load_order_dto_from_row(db, updated).await
}

// Restaurant-side transitions
pub async fn accept(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let now = Utc::now();
    let order = order_repo::transition_status(db, order_id, OrderStatus::Accepted, now)
        .await?
        .ok_or_else(|| AppError::conflict("order cannot be accepted (invalid status)"))?;
    append_event_safe(db, order_id, "order.accepted", &serde_json::json!({})).await?;
    load_order_dto_from_row(db, order).await
}

pub async fn mark_preparing(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let now = Utc::now();
    let order = order_repo::transition_status(db, order_id, OrderStatus::Preparing, now)
        .await?
        .ok_or_else(|| AppError::conflict("order cannot transition to preparing"))?;
    append_event_safe(db, order_id, "order.preparing", &serde_json::json!({})).await?;
    load_order_dto_from_row(db, order).await
}

pub async fn mark_ready(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let now = Utc::now();
    let order = order_repo::transition_status(db, order_id, OrderStatus::Ready, now)
        .await?
        .ok_or_else(|| AppError::conflict("order cannot transition to ready"))?;
    append_event_safe(db, order_id, "order.ready", &serde_json::json!({})).await?;
    load_order_dto_from_row(db, order).await
}

pub async fn mark_picked_up(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let now = Utc::now();
    let order = order_repo::transition_status(db, order_id, OrderStatus::PickedUp, now)
        .await?
        .ok_or_else(|| AppError::conflict("order cannot transition to picked_up"))?;
    append_event_safe(db, order_id, "order.picked_up", &serde_json::json!({})).await?;
    load_order_dto_from_row(db, order).await
}

pub async fn mark_delivered(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let now = Utc::now();
    let order = order_repo::transition_status(db, order_id, OrderStatus::Delivered, now)
        .await?
        .ok_or_else(|| AppError::conflict("order cannot transition to delivered"))?;

    // Record payout splits (platform 15% commission of subtotal, restaurant rest, driver fee+tip).
    let restaurant_amount = (order.subtotal_cents as f64 * 0.85).round() as i64;
    let platform_amount = order.subtotal_cents - restaurant_amount;
    let driver_amount = order.delivery_fee_cents + order.tip_cents;
    let _ = payment_service::record_payout_splits(
        db,
        order_id,
        order.restaurant_id,
        restaurant_amount,
        order.driver_id,
        driver_amount,
        platform_amount,
        &order.currency,
    )
    .await;

    // Award loyalty points (1 point per $1 spent)
    let points_earned = order.subtotal_cents / 100;
    if points_earned > 0 {
        let _ = crate::db::repos::loyalty_repo::adjust_points(
            db,
            order.customer_id,
            points_earned,
            "order_delivered",
            Some(order_id),
        )
        .await;
    }

    append_event_safe(db, order_id, "order.delivered", &serde_json::json!({})).await?;
    load_order_dto_from_row(db, order).await
}

async fn append_event_safe(
    db: &PgPool,
    order_id: Uuid,
    event_type: &str,
    payload: &serde_json::Value,
) -> AppResult<()> {
    let seq = order_repo::next_event_sequence(db, order_id).await?;
    order_repo::append_event(db, order_id, seq, event_type, payload).await?;
    Ok(())
}

async fn load_order_dto(db: &PgPool, order_id: Uuid) -> AppResult<OrderDto> {
    let order = order_repo::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    load_order_dto_from_row(db, order).await
}

async fn load_order_dto_from_row(db: &PgPool, order: crate::domain::order::Order) -> AppResult<OrderDto> {
    let items = order_repo::items_for_order(db, order.id).await?;
    let item_dtos: Vec<OrderItemDto> = items.into_iter().map(|i| i.into()).collect();
    let restaurant = restaurant_repo::find_by_id(db, order.restaurant_id).await?;
    let restaurant_name = restaurant.map(|r| r.name).unwrap_or_default();
    Ok(OrderDto {
        order,
        items: item_dtos,
        restaurant_name,
    })
}

/// Returns the current time (used for ETA calculations).
pub fn _now() -> DateTime<Utc> {
    Utc::now()
}

// Payment intent lookup by order (used by handlers)
pub async fn get_payment_intent(
    db: &PgPool,
    order_id: Uuid,
) -> AppResult<Option<payment_repo::PaymentIntent>> {
    Ok(payment_repo::find_by_order(db, order_id).await?)
}
