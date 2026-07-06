//! Payment service — Stripe integration with mock-mode fallback.
//!
//! When `STRIPE_SECRET_KEY` is empty (dev), payments auto-succeed without
//! hitting Stripe. When set, real Stripe PaymentIntents are created.
//! Idempotency is enforced via the `payment_intents.idempotency_key` UNIQUE
//! constraint AND Stripe's `Idempotency-Key` HTTP header.

use chrono::Utc;
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::payment_repo;
use crate::domain::payment::PaymentStatus;
use crate::error::{AppError, AppResult};
use crate::services::stripe_client::{
    CreatePaymentIntentParams, StripeClient,
};

pub struct PaymentResult {
    pub intent_id: Uuid,
    pub provider_intent_id: Option<String>,
    pub client_secret: Option<String>,
    pub status: PaymentStatus,
    pub amount_cents: i64,
    pub currency: String,
    pub mock_mode: bool,
}

/// Create or return existing payment intent for an order.
/// `idempotency_key` should be the same UUID across retries.
pub async fn create_intent(
    db: &PgPool,
    stripe_secret_key: &str,
    order_id: Uuid,
    amount_cents: i64,
    currency: &str,
    idempotency_key: &str,
) -> AppResult<PaymentResult> {
    // Idempotency check: return existing intent if same key.
    if let Some(existing) = payment_repo::find_by_idempotency_key(db, idempotency_key).await? {
        // If we have a provider intent ID, fetch the client_secret from Stripe
        let client_secret = if existing.provider == "stripe" {
            if let Some(_pi_id) = &existing.provider_intent_id {
                // For security, Stripe returns client_secret only at creation time.
                // If we need it again, we can re-create via confirm or store it.
                // For now, we return None on retrieval (frontend should hold the
                // secret from the initial create_intent call).
                None
            } else {
                None
            }
        } else if existing.provider == "mock" {
            Some(format!("mock_client_secret_{}", existing.id))
        } else {
            None
        };
        return Ok(PaymentResult {
            intent_id: existing.id,
            provider_intent_id: existing.provider_intent_id,
            client_secret,
            status: existing.status,
            amount_cents: existing.amount_cents,
            currency: existing.currency,
            mock_mode: stripe_secret_key.is_empty(),
        });
    }

    let intent_id = Uuid::now_v7();

    if stripe_secret_key.is_empty() {
        // Mock mode: auto-succeed without calling Stripe.
        let intent = payment_repo::insert(
            db,
            intent_id,
            order_id,
            "mock",
            idempotency_key,
            amount_cents,
            currency,
            PaymentStatus::Succeeded,
            Some(&format!("mock_{}", intent_id)),
        )
        .await?;

        // Mark order payment_status as succeeded.
        crate::db::repos::order_repo::set_payment_status(db, order_id, PaymentStatus::Succeeded)
            .await?;

        return Ok(PaymentResult {
            intent_id: intent.id,
            provider_intent_id: intent.provider_intent_id,
            client_secret: Some(format!("mock_client_secret_{}", intent.id)),
            status: PaymentStatus::Succeeded,
            amount_cents: intent.amount_cents,
            currency: intent.currency,
            mock_mode: true,
        });
    }

    // Real Stripe flow.
    let client = StripeClient::new(stripe_secret_key.to_string(), String::new());
    let pi = client
        .create_payment_intent(CreatePaymentIntentParams {
            amount: amount_cents,
            currency,
            metadata_order_id: &order_id.to_string(),
            idempotency_key,
            automatic_payment_methods: true,
        })
        .await?;

    // Map Stripe status to our PaymentStatus.
    let status = match pi.status.as_str() {
        "succeeded" => PaymentStatus::Succeeded,
        "canceled" => PaymentStatus::Canceled,
        "requires_payment_method" | "requires_confirmation" | "requires_action" => {
            PaymentStatus::Pending
        }
        _ => PaymentStatus::Pending,
    };

    let intent = payment_repo::insert(
        db,
        intent_id,
        order_id,
        "stripe",
        idempotency_key,
        amount_cents,
        currency,
        status,
        Some(&pi.id),
    )
    .await?;

    Ok(PaymentResult {
        intent_id: intent.id,
        provider_intent_id: Some(pi.id),
        client_secret: Some(pi.client_secret),
        status,
        amount_cents: intent.amount_cents,
        currency: intent.currency,
        mock_mode: false,
    })
}

/// Mark payment as succeeded (called by webhook handler or mock flow).
pub async fn mark_succeeded(
    db: &PgPool,
    intent_id: Uuid,
    provider_intent_id: Option<&str>,
) -> AppResult<()> {
    payment_repo::set_status(db, intent_id, PaymentStatus::Succeeded, provider_intent_id).await?;
    Ok(())
}

/// Refund an order (partial if amount < original).
/// Calls Stripe if the intent was created there.
pub async fn refund(
    db: &PgPool,
    order_id: Uuid,
    amount_cents: i64,
    reason: &str,
) -> AppResult<()> {
    let intent = payment_repo::find_by_order(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("payment intent"))?;
    let new_status = if amount_cents >= intent.amount_cents {
        PaymentStatus::Refunded
    } else {
        PaymentStatus::PartiallyRefunded
    };

    // If real Stripe intent, call Stripe refund API.
    if intent.provider == "stripe" {
        if let Some(pi_id) = &intent.provider_intent_id {
            let client = StripeClient::new(
                std::env::var("STRIPE_SECRET_KEY").unwrap_or_default(),
                String::new(),
            );
            let _ = client
                .create_refund(
                    pi_id,
                    Some(amount_cents),
                    if reason.is_empty() { "requested_by_customer" } else { reason },
                    &format!("refund-{}-{}", order_id, amount_cents),
                )
                .await?;
        }
    }

    payment_repo::set_status(db, intent.id, new_status, None).await?;
    crate::db::repos::order_repo::set_payment_status(db, order_id, new_status).await?;
    Ok(())
}

/// Record payout splits to restaurant + driver on delivery complete.
pub async fn record_payout_splits(
    db: &PgPool,
    order_id: Uuid,
    restaurant_id: Uuid,
    restaurant_amount_cents: i64,
    driver_id: Option<Uuid>,
    driver_amount_cents: i64,
    platform_amount_cents: i64,
    currency: &str,
) -> AppResult<()> {
    let now = Utc::now();
    let _ = now;
    payment_repo::insert_payout(
        db,
        Uuid::now_v7(),
        order_id,
        "restaurant",
        restaurant_id,
        restaurant_amount_cents,
        currency,
    )
    .await?;
    if let Some(driver_id) = driver_id {
        payment_repo::insert_payout(
            db,
            Uuid::now_v7(),
            order_id,
            "driver",
            driver_id,
            driver_amount_cents,
            currency,
        )
        .await?;
    }
    payment_repo::insert_payout(
        db,
        Uuid::now_v7(),
        order_id,
        "platform",
        order_id, // platform isn't a user; using order_id as payee_id placeholder
        platform_amount_cents,
        currency,
    )
    .await?;
    Ok(())
}
