//! Menu domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuVersion {
    pub id: Uuid,
    pub restaurant_id: Uuid,
    pub version: i32,
    pub published_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "menu_item_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum MenuItemStatus {
    Available,
    OutOfStock,
    Hidden,
}

impl Default for MenuItemStatus {
    fn default() -> Self {
        Self::Available
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuCategory {
    pub id: Uuid,
    pub menu_version_id: Uuid,
    pub name: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuItem {
    pub id: Uuid,
    pub category_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub image_url: Option<String>,
    pub is_veg: bool,
    pub is_vegan: bool,
    pub is_halal: bool,
    pub spice_level: i16,
    pub allergens: Vec<String>,
    pub track_stock: bool,
    pub stock_count: i32,
    pub sort_order: i32,
    pub status: MenuItemStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuItemCustomization {
    pub id: Uuid,
    pub item_id: Uuid,
    pub name: String,
    pub is_required: bool,
    pub max_select: Option<i32>,
    pub sort_order: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct MenuItemCustomizationOption {
    pub id: Uuid,
    pub customization_id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub is_default: bool,
    pub sort_order: i32,
}

/// Grouped menu response for the customer-facing API.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuResponse {
    pub restaurant_id: Uuid,
    pub menu_version: i32,
    pub categories: Vec<MenuCategoryDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuCategoryDto {
    pub id: Uuid,
    pub name: String,
    pub sort_order: i32,
    pub items: Vec<MenuItemDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemDto {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub price_cents: i64,
    pub image_url: Option<String>,
    pub is_veg: bool,
    pub is_vegan: bool,
    pub is_halal: bool,
    pub spice_level: i16,
    pub allergens: Vec<String>,
    pub in_stock: bool,
    pub sort_order: i32,
    pub customizations: Vec<MenuItemCustomizationDto>,
    /// Original category ID — kept for server-side menu assembly,
    /// skipped in JSON since the client gets category_id from the parent.
    #[serde(skip)]
    pub category_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemCustomizationDto {
    pub id: Uuid,
    pub name: String,
    pub is_required: bool,
    pub max_select: Option<i32>,
    pub sort_order: i32,
    pub options: Vec<MenuItemCustomizationOptionDto>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MenuItemCustomizationOptionDto {
    pub id: Uuid,
    pub name: String,
    pub price_cents: i64,
    pub is_default: bool,
    pub sort_order: i32,
}

impl MenuItemDto {
    pub fn from_menu_item(item: MenuItem, customizations: Vec<MenuItemCustomizationDto>) -> Self {
        let in_stock = item.status == MenuItemStatus::Available
            && (!item.track_stock || item.stock_count > 0);
        Self {
            id: item.id,
            name: item.name,
            description: item.description,
            price_cents: item.price_cents,
            image_url: item.image_url,
            is_veg: item.is_veg,
            is_vegan: item.is_vegan,
            is_halal: item.is_halal,
            spice_level: item.spice_level,
            allergens: item.allergens,
            in_stock,
            sort_order: item.sort_order,
            customizations,
            category_id: item.category_id,
        }
    }
}
