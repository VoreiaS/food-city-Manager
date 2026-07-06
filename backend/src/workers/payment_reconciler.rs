//! Payment reconciler — polls for stuck payments and heals them.
//!
//! Two reconciliation passes:
//! 1. **Stripe poll** (every 5 min): for payment_intents in `pending` status
//!    with a `provider_intent_id`, fetch the current status from Stripe API
//!    and update our DB. Handles missed webhooks.
//! 2. **Local heal** (every 2 min): for orders where `payment_status = 'succeeded'`
//!    but `status = 'pending_accept'` for > 10 min, append a `payment.succeeded`
//!    event to unblock the WS subscriber. (Defensive — shouldn't happen but
//!    guards against event-publishing bugs.)
//!
//! Skipped entirely in mock mode (no Stripe key).

use std::sync::Arc;

use chrono::Utc;
use sqlx::PgPool;
use tracing::{info, warn};

use crate::config::Config;
use crate::db::repos::{order_repo, payment_repo};
use crate::domain::payment::PaymentStatus;
use crate::services::stripe_client::StripeClient;

const STRIPE_POLL_INTERVAL_SECS: u64 = 300; // 5 min
const LOCAL_HEAL_INTERVAL_SECS: u64 = 120; // 2 min
const STUCK_ORDER_THRESHOLD_SECS: i64 = 600; // 10 min

pub async fn run(db: PgPool, config: Arc<Config>) {
    info!("payment_reconciler worker started");

    let stripe_configured = !config.stripe.secret_key.is_empty();
    if !stripe_configured {
        info!("payment_reconciler: STRIPE_SECRET_KEY not set — skipping Stripe poll (mock mode)");
    }

    let mut stripe_timer = tokio::time::interval(std::time::Duration::from_secs(STRIPE_POLL_INTERVAL_SECS));
    let mut heal_timer = tokio::time::interval(std::time::Duration::from_secs(LOCAL_HEAL_INTERVAL_SECS));
    stripe_timer.tick().await; // skip immediate
    heal_timer.tick().await;

    loop {
        tokio::select! {
            _ = stripe_timer.tick() => {
                if stripe_configured {
                    if let Err(e) = poll_stripe(&db, &config).await {
                        warn!(error = ?e, "payment_reconciler: stripe poll failed");
                    }
                }
            }
            _ = heal_timer.tick() => {
                if let Err(e) = heal_stuck_orders(&db).await {
                    warn!(error = ?e, "payment_reconciler: local heal failed");
                }
            }
        }
    }
}

/// Poll Stripe for pending payment intents and update our DB.
async fn poll_stripe(db: &PgPool, config: &Config) -> anyhow::Result<()> {
    // Find pending intents with a Stripe ID
    let pending: Vec<payment_repo::PaymentIntent> = sqlx::query_as::<_, payment_repo::PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE status = 'pending'
          AND provider = 'stripe'
          AND provider_intent_id IS NOT NULL
          AND created_at < NOW() - INTERVAL '2 minutes'
        ORDER BY created_at ASC
        LIMIT 50
        "#,
    )
    .fetch_all(db)
    .await?;

    if pending.is_empty() {
        return Ok(());
    }

    let client = StripeClient::new(
        config.stripe.secret_key.clone(),
        config.stripe.webhook_secret.clone(),
    );

    for intent in pending {
        let pi_id = match &intent.provider_intent_id {
            Some(id) => id.clone(),
            None => continue,
        };

        // Fetch PaymentIntent status from Stripe
        match fetch_stripe_intent_status(&client, &pi_id).await {
            Ok(stripe_status) => {
                let new_status = map_stripe_status(&stripe_status);
                if new_status != intent.status && new_status != PaymentStatus::Pending {
                    info!(
                        intent_id = %intent.id,
                        pi_id = %pi_id,
                        old = ?intent.status,
                        new = ?new_status,
                        "reconciler: healing payment intent"
                    );
                    payment_repo::set_status(db, intent.id, new_status, None).await?;
                    order_repo::set_payment_status(db, intent.order_id, new_status).await?;

                    // Append event
                    let seq = order_repo::next_event_sequence(db, intent.order_id).await?;
                    let event_type = match new_status {
                        PaymentStatus::Succeeded => "payment.succeeded",
                        PaymentStatus::Failed => "payment.failed",
                        _ => "payment.updated",
                    };
                    order_repo::append_event(
                        db,
                        intent.order_id,
                        seq,
                        event_type,
                        &serde_json::json!({
                            "intent_id": intent.id,
                            "reconciled": true,
                            "stripe_status": stripe_status,
                        }),
                    )
                    .await?;
                }
            }
            Err(e) => {
                warn!(error = ?e, pi_id = %pi_id, "reconciler: failed to fetch stripe intent");
            }
        }
    }

    Ok(())
}

/// Heal orders that are stuck: payment succeeded but order still in pending_accept.
async fn heal_stuck_orders(db: &PgPool) -> anyhow::Result<()> {
    let stuck: Vec<uuid::Uuid> = sqlx::query_scalar(
        r#"
        SELECT o.id
        FROM orders o
        WHERE o.status = 'pending_accept'
          AND o.payment_status = 'succeeded'
          AND o.placed_at < NOW() - ($1 || ' seconds')::interval
        LIMIT 20
        "#,
    )
    .bind(STUCK_ORDER_THRESHOLD_SECS)
    .fetch_all(db)
    .await?;

    for order_id in stuck {
        warn!(order_id = %order_id, "reconciler: found stuck order, appending heal event");
        let seq = order_repo::next_event_sequence(db, order_id).await?;
        order_repo::append_event(
            db,
            order_id,
            seq,
            "payment.reconciled",
            &serde_json::json!({"healed_at": Utc::now()}),
        )
        .await?;
    }

    Ok(())
}

/// Fetch a PaymentIntent's status from Stripe via GET /v1/payment_intents/:id.
async fn fetch_stripe_intent_status(
    client: &StripeClient,
    pi_id: &str,
) -> anyhow::Result<String> {
    // Use reqwest directly — we don't expose a public method on StripeClient for GET.
    // For simplicity we re-build the request here.
    let resp = reqwest::Client::new()
        .get(format!("https://api.stripe.com/v1/payment_intents/{}", pi_id))
        .bearer_auth(client.secret_key_for_reconciler())
        .send()
        .await?;
    let body = resp.text().await?;
    let v: serde_json::Value = serde_json::from_str(&body)?;
    let status = v
        .get("status")
        .and_then(|s| s.as_str())
        .unwrap_or("unknown")
        .to_string();
    Ok(status)
}

fn map_stripe_status(s: &str) -> PaymentStatus {
    match s {
        "succeeded" => PaymentStatus::Succeeded,
        "canceled" => PaymentStatus::Canceled,
        "requires_payment_method" | "requires_confirmation" | "requires_action" => {
            PaymentStatus::Pending
        }
        _ => PaymentStatus::Pending,
    }
}
