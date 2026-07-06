//! Menu repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::menu::{
    MenuItem, MenuItemCustomization, MenuItemCustomizationOption,
    MenuVersion,
};

/// Get the active menu version for a restaurant.
pub async fn active_version(
    db: &PgPool,
    restaurant_id: Uuid,
) -> SqlxResult<Option<MenuVersion>> {
    sqlx::query_as::<_, MenuVersion>(
        r#"
        SELECT id, restaurant_id, version, published_at, is_active
        FROM menu_versions
        WHERE restaurant_id = $1 AND is_active = true
        ORDER BY version DESC
        LIMIT 1
        "#,
    )
    .bind(restaurant_id)
    .fetch_optional(db)
    .await
}

/// Get all categories for a menu version, ordered by sort_order.
pub async fn categories(
    db: &PgPool,
    menu_version_id: Uuid,
) -> SqlxResult<Vec<(Uuid, String, i32)>> {
    let rows: Vec<(Uuid, String, i32)> = sqlx::query_as(
        r#"
        SELECT id, name, sort_order
        FROM menu_categories
        WHERE menu_version_id = $1
        ORDER BY sort_order, name
        "#,
    )
    .bind(menu_version_id)
    .fetch_all(db)
    .await?;
    Ok(rows)
}

/// Get all items in a list of category IDs.
pub async fn items_for_categories(
    db: &PgPool,
    category_ids: &[Uuid],
) -> SqlxResult<Vec<MenuItem>> {
    if category_ids.is_empty() {
        return Ok(vec![]);
    }
    sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT id, category_id, name, description, price_cents, image_url,
               is_veg, is_vegan, is_halal, spice_level, allergens,
               track_stock, stock_count, sort_order,
               status as "status",
               created_at, updated_at
        FROM menu_items
        WHERE category_id = ANY($1)
        ORDER BY sort_order, name
        "#,
    )
    .bind(category_ids)
    .fetch_all(db)
    .await
}

/// Get all customizations for a list of menu item IDs.
pub async fn customizations_for_items(
    db: &PgPool,
    item_ids: &[Uuid],
) -> SqlxResult<Vec<MenuItemCustomization>> {
    if item_ids.is_empty() {
        return Ok(vec![]);
    }
    sqlx::query_as::<_, MenuItemCustomization>(
        r#"
        SELECT id, item_id, name, is_required, max_select, sort_order
        FROM menu_item_customizations
        WHERE item_id = ANY($1)
        ORDER BY sort_order, name
        "#,
    )
    .bind(item_ids)
    .fetch_all(db)
    .await
}

/// Get all customization options for a list of customization IDs.
pub async fn options_for_customizations(
    db: &PgPool,
    customization_ids: &[Uuid],
) -> SqlxResult<Vec<MenuItemCustomizationOption>> {
    if customization_ids.is_empty() {
        return Ok(vec![]);
    }
    sqlx::query_as::<_, MenuItemCustomizationOption>(
        r#"
        SELECT id, customization_id, name, price_cents, is_default, sort_order
        FROM menu_item_customization_options
        WHERE customization_id = ANY($1)
        ORDER BY sort_order, name
        "#,
    )
    .bind(customization_ids)
    .fetch_all(db)
    .await
}

/// Get a single menu item by ID (with customizations loaded).
pub async fn find_item_by_id(db: &PgPool, item_id: Uuid) -> SqlxResult<Option<MenuItem>> {
    sqlx::query_as::<_, MenuItem>(
        r#"
        SELECT id, category_id, name, description, price_cents, image_url,
               is_veg, is_vegan, is_halal, spice_level, allergens,
               track_stock, stock_count, sort_order,
               status as "status",
               created_at, updated_at
        FROM menu_items
        WHERE id = $1
        "#,
    )
    .bind(item_id)
    .fetch_optional(db)
    .await
}

/// Atomically decrement stock (returns updated count, or None if sold out).
/// Use for inventory-tracked items at order time.
pub async fn decrement_stock(
    db: &PgPool,
    item_id: Uuid,
    qty: i32,
) -> SqlxResult<Option<i32>> {
    let row: Option<(Option<i32>,)> = sqlx::query_as(
        r#"
        UPDATE menu_items
        SET stock_count = stock_count - $2,
            updated_at = NOW()
        WHERE id = $1
          AND track_stock = true
          AND status = 'available'
          AND stock_count >= $2
        RETURNING stock_count
        "#,
    )
    .bind(item_id)
    .bind(qty)
    .fetch_optional(db)
    .await?;
    Ok(row.and_then(|r| r.0))
}
