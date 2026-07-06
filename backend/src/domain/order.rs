//! Order domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

use crate::domain::payment::PaymentStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    PendingAccept,
    Accepted,
    Preparing,
    Ready,
    PickedUp,
    Delivering,
    Delivered,
    Canceled,
    AutoRejected,
}

impl OrderStatus {
    pub fn can_transition_to(&self, target: Self) -> bool {
        use OrderStatus::*;
        matches!(
            (self, target),
            (PendingAccept, Accepted)
                | (PendingAccept, Canceled)
                | (PendingAccept, AutoRejected)
                | (Accepted, Preparing)
                | (Accepted, Canceled)
                | (Preparing, Ready)
                | (Preparing, Canceled)
                | (Ready, PickedUp)
                | (Ready, Canceled)
                | (PickedUp, Delivering)
                | (Delivering, Delivered)
                | (Delivering, Canceled)
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Order {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub restaurant_id: Uuid,
    pub driver_id: Option<Uuid>,
    pub status: OrderStatus,
    pub payment_status: PaymentStatus,
    pub snapshot: serde_json::Value,
    pub subtotal_cents: i64,
    pub delivery_fee_cents: i64,
    pub tax_cents: i64,
    pub tip_cents: i64,
    pub discount_cents: i64,
    pub total_cents: i64,
    pub currency: String,
    pub delivery_address: serde_json::Value,
    pub notes: Option<String>,
    pub placed_at: DateTime<Utc>,
    pub accepted_at: Option<DateTime<Utc>>,
    pub preparing_at: Option<DateTime<Utc>>,
    pub ready_at: Option<DateTime<Utc>>,
    pub picked_up_at: Option<DateTime<Utc>>,
    pub delivered_at: Option<DateTime<Utc>>,
    pub canceled_at: Option<DateTime<Utc>>,
    pub cancellation_reason: Option<String>,
    pub estimated_delivery_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct OrderItem {
    pub id: Uuid,
    pub order_id: Uuid,
    pub menu_item_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub quantity: i32,
    pub customizations: serde_json::Value,
    pub notes: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItemDto {
    pub id: Uuid,
    pub menu_item_id: Option<Uuid>,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub quantity: i32,
    pub customizations: serde_json::Value,
    pub notes: Option<String>,
    pub status: String,
    pub line_total_cents: i64,
}

impl From<OrderItem> for OrderItemDto {
    fn from(i: OrderItem) -> Self {
        let line_total = i.price_cents * i.quantity as i64;
        Self {
            id: i.id,
            menu_item_id: i.menu_item_id,
            name: i.name,
            description: i.description,
            price_cents: i.price_cents,
            quantity: i.quantity,
            customizations: i.customizations,
            notes: i.notes,
            status: i.status,
            line_total_cents: line_total,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderDto {
    #[serde(flatten)]
    pub order: Order,
    pub items: Vec<OrderItemDto>,
    pub restaurant_name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateOrderRequest {
    pub address_id: Uuid,
    pub payment_method_id: Option<String>,
    pub promo_code: Option<String>,
    pub tip_cents: Option<i64>,
    pub loyalty_points_to_redeem: Option<i64>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CreateOrderResponse {
    pub order: OrderDto,
    pub payment: PaymentIntentDto,
}

#[derive(Debug, Serialize)]
pub struct PaymentIntentDto {
    pub intent_id: Uuid,
    pub provider_intent_id: Option<String>,
    pub client_secret: Option<String>,
    pub status: PaymentStatus,
    pub amount_cents: i64,
    pub currency: String,
    /// For mock mode, true means payment auto-succeeded (no real money moved).
    pub mock_mode: bool,
}
