//! Promo code service — validation, redemption, discount computation.

use chrono::Utc;
use rust_decimal::Decimal;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::promo_repo;
use crate::error::{AppError, AppResult};

pub struct PromoValidation {
    pub promo_id: Uuid,
    pub code: String,
    pub discount_type: String,
    pub discount_value: Decimal,
    pub discount_cents: i64,
    pub description: Option<String>,
}

/// Validate a promo code against an order. Returns the discount to apply (in cents).
/// Does NOT consume the promo — caller must call `redeem()` after order is placed.
pub async fn validate(
    db: &PgPool,
    code: &str,
    user_id: Uuid,
    order_subtotal_cents: i64,
    restaurant_id: Uuid,
) -> AppResult<PromoValidation> {
    let code_upper = code.trim().to_uppercase();
    if code_upper.is_empty() {
        return Err(AppError::validation("promo code is empty"));
    }

    let promo = promo_repo::find_by_code(db, &code_upper)
        .await?
        .ok_or_else(|| AppError::business_rule(format!("promo code '{}' not found or inactive", code_upper)))?;

    // Check validity window
    let now = Utc::now();
    if promo.valid_from > now {
        return Err(AppError::business_rule("promo code not yet active"));
    }
    if let Some(until) = promo.valid_until {
        if until < now {
            return Err(AppError::business_rule("promo code expired"));
        }
    }

    // Check global usage cap
    if let Some(max) = promo.max_uses {
        if promo.used_count >= max {
            return Err(AppError::business_rule("promo code fully redeemed"));
        }
    }

    // Check daily cap
    if let Some(daily_cap) = promo.daily_cap {
        let today_count = promo_repo::count_today_redemptions(db, promo.id).await?;
        if today_count >= daily_cap as i64 {
            return Err(AppError::business_rule("promo code daily limit reached"));
        }
    }

    // Check per-user cap
    let user_count = promo_repo::count_user_redemptions(db, promo.id, user_id).await?;
    if user_count >= promo.per_user_cap as i64 {
        return Err(AppError::business_rule(
            "you've already used this promo code the maximum number of times",
        ));
    }

    // Check minimum order
    if order_subtotal_cents < promo.min_order_cents {
        return Err(AppError::business_rule(format!(
            "promo requires minimum order of ${:.2}",
            promo.min_order_cents as f64 / 100.0
        )));
    }

    // Check restaurant applicability
    if !promo.applicable_restaurants.is_empty()
        && !promo.applicable_restaurants.contains(&restaurant_id)
    {
        return Err(AppError::business_rule(
            "promo code not valid for this restaurant",
        ));
    }

    // Check customer segment
    // - "all": anyone can use
    // - "new": only users who have never placed a delivered order
    // - "vip": only users with loyalty tier >= gold
    match promo.customer_segment.as_str() {
        "all" => { /* no restriction */ }
        "new" => {
            let delivered_count: i64 = sqlx::query_scalar(
                "SELECT COUNT(*) FROM orders WHERE customer_id = $1 AND status = 'delivered'",
            )
            .bind(user_id)
            .fetch_one(db)
            .await
            .map_err(AppError::Database)?;
            if delivered_count > 0 {
                return Err(AppError::business_rule(
                    "promo code is only valid for first-time customers",
                ));
            }
        }
        "vip" => {
            let account = crate::db::repos::loyalty_repo::find_by_user(db, user_id).await?;
            let tier = account
                .as_ref()
                .map(|a| a.tier)
                .unwrap_or(crate::domain::loyalty::LoyaltyTier::Silver);
            if !matches!(
                tier,
                crate::domain::loyalty::LoyaltyTier::Gold | crate::domain::loyalty::LoyaltyTier::Platinum
            ) {
                return Err(AppError::business_rule(
                    "promo code is only valid for Gold+ loyalty members",
                ));
            }
        }
        _ => { /* unknown segment — allow */ }
    }

    // Compute discount
    let discount_cents = compute_discount(
        &promo.discount_type,
        promo.discount_value,
        order_subtotal_cents,
    )?;

    // Discount cannot exceed subtotal
    let discount_cents = discount_cents.min(order_subtotal_cents);

    Ok(PromoValidation {
        promo_id: promo.id,
        code: promo.code,
        discount_type: promo.discount_type,
        discount_value: promo.discount_value,
        discount_cents,
        description: promo.description,
    })
}

/// Compute the discount amount in cents based on type.
fn compute_discount(
    discount_type: &str,
    value: Decimal,
    subtotal_cents: i64,
) -> AppResult<i64> {
    match discount_type {
        "percent" => {
            let pct = value;
            if pct > Decimal::from(100) || pct < Decimal::ZERO {
                return Err(AppError::internal(format!(
                    "invalid percent discount: {}",
                    pct
                )));
            }
            // discount = subtotal * pct / 100
            let sub = Decimal::from(subtotal_cents);
            let disc = sub * pct / Decimal::from(100);
            Ok(disc.to_i64().unwrap_or(0))
        }
        "flat" => {
            // value is in cents directly
            Ok(value.to_i64().unwrap_or(0))
        }
        "free_delivery" => {
            // Discount = delivery fee, but we don't know it here.
            // Caller should override: set discount = delivery_fee_cents.
            // For now return 0; the order service will handle this case.
            Ok(0)
        }
        other => Err(AppError::internal(format!(
            "unknown discount type: {}",
            other
        ))),
    }
}

/// Redeem a promo code for an order. Atomically increments used_count and
/// records the redemption. Should be called AFTER order is created.
pub async fn redeem(
    db: &PgPool,
    promo_id: Uuid,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<()> {
    let allowed = promo_repo::try_increment_used_count(db, promo_id).await?;
    if !allowed {
        return Err(AppError::business_rule(
            "promo code could not be redeemed (exhausted or expired)",
        ));
    }
    promo_repo::record_redemption(db, promo_id, user_id, Some(order_id)).await?;
    Ok(())
}

/// Reverse a redemption (when order is canceled). Decrements used_count.
pub async fn unredeem(db: &PgPool, promo_id: Uuid) -> AppResult<()> {
    promo_repo::decrement_used_count(db, promo_id).await?;
    Ok(())
}

// Helper trait for Decimal → i64 conversion (rust_decimal provides this via ToPrimitive).
use rust_decimal::prelude::ToPrimitive;
