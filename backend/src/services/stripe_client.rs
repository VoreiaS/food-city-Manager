//! Stripe API client — raw reqwest calls (no SDK dependency).
//!
//! Implements:
//! - `create_payment_intent`: POST /v1/payment_intents
//! - `create_refund`: POST /v1/refunds
//! - `verify_webhook_signature`: HMAC-SHA256 with timestamp
//!
//! All Stripe API calls use Bearer auth with the secret key and pass
//! `Idempotency-Key` header on POSTs to prevent double-charges on retry.

use hmac::{Hmac, Mac};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::time::Duration;

use crate::error::{AppError, AppResult};

const STRIPE_API_BASE: &str = "https://api.stripe.com/v1";

type HmacSha256 = Hmac<Sha256>;

#[derive(Debug, Clone)]
pub struct StripeClient {
    secret_key: String,
    webhook_secret: String,
    http: Client,
}

#[derive(Debug, Serialize)]
pub struct CreatePaymentIntentParams<'a> {
    pub amount: i64, // cents
    pub currency: &'a str,
    pub metadata_order_id: &'a str,
    pub idempotency_key: &'a str,
    pub automatic_payment_methods: bool,
}

#[derive(Debug, Deserialize)]
pub struct PaymentIntentResponse {
    pub id: String,
    pub client_secret: String,
    pub status: String, // "requires_payment_method" | "succeeded" | "canceled" | ...
    pub amount: i64,
    pub currency: String,
}

#[derive(Debug, Deserialize)]
pub struct RefundResponse {
    pub id: String,
    pub status: String, // "succeeded" | "pending" | "failed"
    pub amount: i64,
}

#[derive(Debug, Deserialize)]
pub struct StripeError {
    #[serde(default)]
    pub error: Option<StripeErrorBody>,
}

#[derive(Debug, Deserialize)]
pub struct StripeErrorBody {
    #[serde(default)]
    pub r#type: String,
    #[serde(default)]
    pub message: String,
    #[serde(default)]
    pub code: Option<String>,
}

impl StripeClient {
    pub fn new(secret_key: String, webhook_secret: String) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");
        Self {
            secret_key,
            webhook_secret,
            http,
        }
    }

    pub fn is_configured(&self) -> bool {
        !self.secret_key.is_empty()
    }

    /// Read the secret key (used by payment_reconciler to make direct GET requests).
    pub fn secret_key_for_reconciler(&self) -> &str {
        &self.secret_key
    }

    /// Create a PaymentIntent on Stripe.
    /// Returns (provider_intent_id, client_secret, stripe_status).
    pub async fn create_payment_intent(
        &self,
        params: CreatePaymentIntentParams<'_>,
    ) -> AppResult<PaymentIntentResponse> {
        let mut form = vec![
            ("amount".to_string(), params.amount.to_string()),
            ("currency".to_string(), params.currency.to_string()),
            (
                "metadata[order_id]".to_string(),
                params.metadata_order_id.to_string(),
            ),
        ];
        if params.automatic_payment_methods {
            form.push((
                "automatic_payment_methods[enabled]".to_string(),
                "true".to_string(),
            ));
        }

        let resp = self
            .http
            .post(format!("{}/payment_intents", STRIPE_API_BASE))
            .bearer_auth(&self.secret_key)
            .header("Idempotency-Key", params.idempotency_key)
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Stripe(format!("stripe request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Stripe(format!("read body failed: {}", e)))?;

        if !status.is_success() {
            let err: StripeError = serde_json::from_str(&body).unwrap_or(StripeError { error: None });
            let msg = err
                .error
                .as_ref()
                .map(|e| format!("{}: {}", e.r#type, e.message))
                .unwrap_or_else(|| format!("stripe error: HTTP {}", status));
            return Err(AppError::Stripe(msg));
        }

        let pi: PaymentIntentResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::Stripe(format!("parse payment_intent failed: {}", e)))?;
        Ok(pi)
    }

    /// Create a refund on a previously-charged PaymentIntent.
    /// `amount` is in cents; if None, refunds full amount.
    pub async fn create_refund(
        &self,
        payment_intent_id: &str,
        amount: Option<i64>,
        reason: &str,
        idempotency_key: &str,
    ) -> AppResult<RefundResponse> {
        let mut form = vec![
            ("payment_intent".to_string(), payment_intent_id.to_string()),
            ("reason".to_string(), reason.to_string()),
        ];
        if let Some(amt) = amount {
            form.push(("amount".to_string(), amt.to_string()));
        }

        let resp = self
            .http
            .post(format!("{}/refunds", STRIPE_API_BASE))
            .bearer_auth(&self.secret_key)
            .header("Idempotency-Key", idempotency_key)
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::Stripe(format!("stripe refund request failed: {}", e)))?;

        let status = resp.status();
        let body = resp
            .text()
            .await
            .map_err(|e| AppError::Stripe(format!("read refund body failed: {}", e)))?;

        if !status.is_success() {
            let err: StripeError = serde_json::from_str(&body).unwrap_or(StripeError { error: None });
            let msg = err
                .error
                .as_ref()
                .map(|e| format!("{}: {}", e.r#type, e.message))
                .unwrap_or_else(|| format!("stripe refund error: HTTP {}", status));
            return Err(AppError::Stripe(msg));
        }

        let refund: RefundResponse = serde_json::from_str(&body)
            .map_err(|e| AppError::Stripe(format!("parse refund failed: {}", e)))?;
        Ok(refund)
    }

    /// Verify Stripe webhook signature.
    ///
    /// Stripe sends `Stripe-Signature` header: `t=<timestamp>,v1=<signature>`.
    /// We compute HMAC-SHA256(webhook_secret, "{timestamp}.{body}") and compare
    /// to the v1 signature. We also reject timestamps older than 5 min to
    /// prevent replay attacks.
    pub fn verify_webhook_signature(
        &self,
        signature_header: &str,
        body: &[u8],
    ) -> AppResult<()> {
        if self.webhook_secret.is_empty() {
            return Err(AppError::Stripe(
                "STRIPE_WEBHOOK_SECRET not configured".into(),
            ));
        }

        let mut timestamp: Option<&str> = None;
        let mut signatures: Vec<&str> = Vec::new();
        for kv in signature_header.split(',') {
            let kv = kv.trim();
            if let Some(t) = kv.strip_prefix("t=") {
                timestamp = Some(t);
            } else if let Some(v) = kv.strip_prefix("v1=") {
                signatures.push(v);
            }
        }

        let ts = timestamp.ok_or_else(|| AppError::Stripe("missing t= in signature".into()))?;
        let ts_num: i64 = ts
            .parse()
            .map_err(|_| AppError::Stripe("invalid timestamp".into()))?;

        // Replay protection: reject > 5 min old
        let now = chrono::Utc::now().timestamp();
        if (now - ts_num).abs() > 300 {
            return Err(AppError::Stripe("webhook timestamp too old (replay?)".into()));
        }

        // Compute expected signature: HMAC-SHA256(secret, "{ts}.{body}")
        let signed_payload = format!("{}.", ts);
        let mut mac = HmacSha256::new_from_slice(self.webhook_secret.as_bytes())
            .map_err(|e| AppError::internal(format!("hmac init failed: {}", e)))?;
        mac.update(signed_payload.as_bytes());
        mac.update(body);
        let expected = hex::encode(mac.finalize().into_bytes());

        // Compare against any of the provided v1 signatures (Stripe may send multiple)
        if signatures.iter().any(|s| {
            // Constant-time comparison to prevent timing attacks
            hmac_equal(s.as_bytes(), expected.as_bytes())
        }) {
            Ok(())
        } else {
            Err(AppError::Stripe("webhook signature mismatch".into()))
        }
    }
}

/// Constant-time comparison to mitigate timing attacks.
fn hmac_equal(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut acc: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        acc |= x ^ y;
    }
    acc == 0
}

/// Parsed webhook event (subset of fields we care about).
#[derive(Debug, Deserialize)]
pub struct WebhookEvent {
    pub id: String,
    #[serde(rename = "type")]
    pub event_type: String,
    pub data: WebhookEventData,
}

#[derive(Debug, Deserialize)]
pub struct WebhookEventData {
    pub object: serde_json::Value,
}

/// Extract PaymentIntent ID from a webhook event payload.
/// Different event types nest it differently.
pub fn extract_payment_intent_id(event: &WebhookEvent) -> Option<String> {
    let obj = &event.data.object;
    // payment_intent.* events: object.id is the PI id
    if let Some(id) = obj.get("id").and_then(|v| v.as_str()) {
        if id.starts_with("pi_") {
            return Some(id.to_string());
        }
    }
    // charge.* events: object.payment_intent is the PI id
    if let Some(pi) = obj.get("payment_intent").and_then(|v| v.as_str()) {
        return Some(pi.to_string());
    }
    // dispute.* events: object.payment_intent or object.charge (then look up PI)
    if let Some(charge) = obj.get("charge").and_then(|v| v.as_str()) {
        // We don't have the charge → PI mapping here; admin will investigate
        let _ = charge;
    }
    None
}
