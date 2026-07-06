//! Payment domain types.

use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

/// Single source of truth for payment status. Used by both `payment_intents`
/// and `orders.payment_status` columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "payment_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentStatus {
    Pending,
    Succeeded,
    Failed,
    Canceled,
    Refunded,
    PartiallyRefunded,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentIntent {
    pub id: Uuid,
    pub order_id: Uuid,
    pub provider_intent_id: String,
    pub idempotency_key: String,
    pub amount_cents: i64,
    pub currency: String,
    pub status: PaymentStatus,
}
