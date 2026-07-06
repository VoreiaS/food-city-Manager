//! Cart domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "cart_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum CartStatus {
    Active,
    Locked,
    Converted,
    Abandoned,
}

impl Default for CartStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Cart {
    pub id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub status: CartStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct CartItem {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub menu_item_id: Uuid,
    pub menu_version_at_add: i32,
    pub quantity: i32,
    pub customizations: serde_json::Value,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Client-facing cart item with resolved menu data (name, price).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemDto {
    pub id: Uuid,
    pub cart_id: Uuid,
    pub menu_item_id: Uuid,
    pub menu_item_name: String,
    pub menu_item_image_url: Option<String>,
    pub base_price_cents: i64,
    pub quantity: i32,
    pub customizations: Vec<CartItemCustomizationDto>,
    pub notes: Option<String>,
    pub line_total_cents: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartItemCustomizationDto {
    pub customization_id: Uuid,
    pub customization_name: String,
    pub option_id: Uuid,
    pub option_name: String,
    pub price_cents: i64,
}

/// Full cart response with items + totals.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CartResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    pub restaurant_id: Uuid,
    pub restaurant_name: String,
    pub status: CartStatus,
    pub items: Vec<CartItemDto>,
    pub subtotal_cents: i64,
    pub delivery_fee_cents: i64,
    pub total_cents: i64,
    pub min_order_cents: i64,
    pub meets_min_order: bool,
}

#[derive(Debug, Deserialize)]
pub struct AddCartItemRequest {
    pub restaurant_id: Uuid,
    pub menu_item_id: Uuid,
    pub quantity: i32,
    #[serde(default)]
    pub customizations: Vec<AddCustomizationSelection>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AddCustomizationSelection {
    pub customization_id: Uuid,
    pub option_id: Uuid,
}

#[derive(Debug, Deserialize)]
pub struct UpdateCartItemRequest {
    pub quantity: Option<i32>,
    pub notes: Option<String>,
}
