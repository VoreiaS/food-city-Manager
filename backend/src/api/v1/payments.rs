//! Payment endpoints + Stripe webhook handler.

use axum::{
    body::Bytes,
    extract::{Path, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use serde::Serialize;
use uuid::Uuid;

use crate::api::AppState;
use crate::db::repos::{order_repo, payment_repo};
use crate::domain::payment::PaymentStatus;
use crate::error::{AppError, AppResult};
use crate::services::stripe_client::{extract_payment_intent_id, WebhookEvent};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/payments/intents/:order_id", get(get_intent))
        .route("/payments/webhooks/stripe", post(stripe_webhook))
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentDto {
    pub intent_id: Uuid,
    pub order_id: Uuid,
    pub provider: String,
    pub provider_intent_id: Option<String>,
    pub client_secret: Option<String>,
    pub amount_cents: i64,
    pub currency: String,
    pub status: String,
    pub mock_mode: bool,
}

async fn get_intent(
    State(state): State<AppState>,
    Path(order_id): Path<Uuid>,
) -> AppResult<Json<PaymentIntentDto>> {
    let intent = payment_repo::find_by_order(&state.db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("payment intent"))?;
    let status = format!("{:?}", intent.status).to_lowercase();
    let mock_mode = intent.provider == "mock";
    let provider_intent_id = intent.provider_intent_id.clone();
    Ok(Json(PaymentIntentDto {
        intent_id: intent.id,
        order_id: intent.order_id,
        provider: intent.provider,
        provider_intent_id,
        client_secret: None, // not returned on retrieval; only at creation
        amount_cents: intent.amount_cents,
        currency: intent.currency,
        status,
        mock_mode,
    }))
}

/// Stripe webhook handler.
///
/// Verifies the HMAC-SHA256 signature, then dispatches the event type:
/// - `payment_intent.succeeded` → mark order + intent as paid
/// - `payment_intent.payment_failed` → mark intent as failed
/// - `charge.refunded` → update intent refund status
/// - `charge.dispute.created` → flag dispute for admin review
///
/// Returns 200 only after successful processing (Stripe retries on 5xx).
/// Idempotent via `payment_webhooks.provider_event_id` UNIQUE.
async fn stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    let sig_header = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or_else(|| AppError::Stripe("missing Stripe-Signature header".into()))?;

    let webhook_secret = &state.config.stripe.webhook_secret;
    if webhook_secret.is_empty() {
        return Err(AppError::Stripe(
            "STRIPE_WEBHOOK_SECRET not configured".into(),
        ));
    }

    // Verify signature (also enforces 5-min replay window)
    let client = crate::services::stripe_client::StripeClient::new(
        state.config.stripe.secret_key.clone(),
        webhook_secret.clone(),
    );
    client.verify_webhook_signature(sig_header, &body)?;

    // Parse event
    let event: WebhookEvent = serde_json::from_slice(&body)
        .map_err(|e| AppError::Stripe(format!("parse event failed: {}", e)))?;

    // Idempotency: insert into payment_webhooks. If event ID already exists,
    // we already processed it — return 200.
    let existing: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM payment_webhooks WHERE provider_event_id = $1")
            .bind(&event.id)
            .fetch_optional(&state.db)
            .await
            .map_err(AppError::Database)?;
    if existing.is_some() {
        tracing::info!(event_id = %event.id, "stripe webhook already processed");
        return Ok(Json(serde_json::json!({"received": true, "duplicate": true})));
    }

    // Record event
    let payload: serde_json::Value = serde_json::from_slice(&body).unwrap_or(serde_json::json!({}));
    sqlx::query(
        r#"
        INSERT INTO payment_webhooks (provider_event_id, event_type, payload)
        VALUES ($1, $2, $3)
        ON CONFLICT (provider_event_id) DO NOTHING
        "#,
    )
    .bind(&event.id)
    .bind(&event.event_type)
    .bind(&payload)
    .execute(&state.db)
    .await
    .map_err(AppError::Database)?;

    // Dispatch
    let pi_id = extract_payment_intent_id(&event);
    tracing::info!(
        event_id = %event.id,
        event_type = %event.event_type,
        pi_id = ?pi_id,
        "processing stripe webhook"
    );

    match event.event_type.as_str() {
        "payment_intent.succeeded" => {
            if let Some(pi_id) = pi_id {
                handle_payment_succeeded(&state.db, &pi_id).await?;
            }
        }
        "payment_intent.payment_failed" => {
            if let Some(pi_id) = pi_id {
                handle_payment_failed(&state.db, &pi_id).await?;
            }
        }
        "charge.refunded" => {
            if let Some(pi_id) = pi_id {
                handle_refunded(&state.db, &pi_id, &event).await?;
            }
        }
        "charge.dispute.created" => {
            // Just log — admin will investigate via Stripe dashboard
            tracing::warn!(
                event_id = %event.id,
                "stripe dispute created — admin review required"
            );
        }
        _ => {
            tracing::debug!(event_type = %event.event_type, "unhandled stripe event type");
        }
    }

    Ok(Json(serde_json::json!({"received": true, "processed": true})))
}

async fn handle_payment_succeeded(db: &sqlx::PgPool, pi_id: &str) -> AppResult<()> {
    // Find our payment_intent by provider_intent_id
    let intent: Option<payment_repo::PaymentIntent> = sqlx::query_as::<_, payment_repo::PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE provider_intent_id = $1
        "#,
    )
    .bind(pi_id)
    .fetch_optional(db)
    .await
    .map_err(AppError::Database)?;

    let intent = match intent {
        Some(i) => i,
        None => {
            tracing::warn!(pi_id = %pi_id, "payment_intent not found for webhook");
            return Ok(());
        }
    };

    // Guard: only update if currently pending
    if intent.status == PaymentStatus::Pending {
        payment_repo::set_status(db, intent.id, PaymentStatus::Succeeded, None).await?;
        order_repo::set_payment_status(db, intent.order_id, PaymentStatus::Succeeded).await?;

        // Append event for WS fan-out
        let seq = order_repo::next_event_sequence(db, intent.order_id).await?;
        order_repo::append_event(
            db,
            intent.order_id,
            seq,
            "payment.succeeded",
            &serde_json::json!({"amount_cents": intent.amount_cents}),
        )
        .await?;
    }
    Ok(())
}

async fn handle_payment_failed(db: &sqlx::PgPool, pi_id: &str) -> AppResult<()> {
    let intent: Option<payment_repo::PaymentIntent> = sqlx::query_as::<_, payment_repo::PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE provider_intent_id = $1
        "#,
    )
    .bind(pi_id)
    .fetch_optional(db)
    .await
    .map_err(AppError::Database)?;

    let intent = match intent {
        Some(i) => i,
        None => return Ok(()),
    };

    payment_repo::set_status(db, intent.id, PaymentStatus::Failed, None).await?;
    order_repo::set_payment_status(db, intent.order_id, PaymentStatus::Failed).await?;

    // Auto-cancel the order since payment failed
    let now = chrono::Utc::now();
    let _ = order_repo::transition_status(
        db,
        intent.order_id,
        crate::domain::order::OrderStatus::Canceled,
        now,
    )
    .await;
    order_repo::set_cancellation(db, intent.order_id, "payment_failed").await?;

    let seq = order_repo::next_event_sequence(db, intent.order_id).await?;
    order_repo::append_event(
        db,
        intent.order_id,
        seq,
        "order.canceled",
        &serde_json::json!({"reason": "payment_failed"}),
    )
    .await?;

    Ok(())
}

async fn handle_refunded(
    db: &sqlx::PgPool,
    pi_id: &str,
    event: &WebhookEvent,
) -> AppResult<()> {
    let intent: Option<payment_repo::PaymentIntent> = sqlx::query_as::<_, payment_repo::PaymentIntent>(
        r#"
        SELECT id, order_id, provider, provider_intent_id, idempotency_key,
               amount_cents, currency,
               status as "status",
               created_at, updated_at
        FROM payment_intents
        WHERE provider_intent_id = $1
        "#,
    )
    .bind(pi_id)
    .fetch_optional(db)
    .await
    .map_err(AppError::Database)?;

    let intent = match intent {
        Some(i) => i,
        None => return Ok(()),
    };

    // Determine if partial or full refund
    let refunded_amount = event
        .data
        .object
        .get("amount_refunded")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let new_status = if refunded_amount >= intent.amount_cents {
        PaymentStatus::Refunded
    } else {
        PaymentStatus::PartiallyRefunded
    };
    payment_repo::set_status(db, intent.id, new_status, None).await?;
    order_repo::set_payment_status(db, intent.order_id, new_status).await?;
    Ok(())
}
