//! Cart repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::cart::{Cart, CartItem, CartStatus};

/// Find the user's active cart (one per user).
pub async fn find_active(db: &PgPool, user_id: Uuid) -> SqlxResult<Option<Cart>> {
    sqlx::query_as::<_, Cart>(
        r#"
        SELECT id, user_id, restaurant_id, status as "status", created_at, updated_at
        FROM carts
        WHERE user_id = $1 AND status = 'active'
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

/// Create a new active cart.
pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    restaurant_id: Uuid,
) -> SqlxResult<Cart> {
    sqlx::query_as::<_, Cart>(
        r#"
        INSERT INTO carts (user_id, restaurant_id, status)
        VALUES ($1, $2, 'active')
        RETURNING id, user_id, restaurant_id, status as "status", created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(restaurant_id)
    .fetch_one(db)
    .await
}

/// Set cart status (used for locking / converting / abandoning).
pub async fn set_status(
    db: &PgPool,
    cart_id: Uuid,
    status: CartStatus,
) -> SqlxResult<()> {
    sqlx::query("UPDATE carts SET status = $2, updated_at = NOW() WHERE id = $1")
        .bind(cart_id)
        .bind(status)
        .execute(db)
        .await?;
    Ok(())
}

/// Delete cart + all items (hard delete — abandoned carts gone for good).
pub async fn delete(db: &PgPool, cart_id: Uuid) -> SqlxResult<()> {
    sqlx::query("DELETE FROM carts WHERE id = $1")
        .bind(cart_id)
        .execute(db)
        .await?;
    Ok(())
}

/// List items in a cart, ordered by creation.
pub async fn items(db: &PgPool, cart_id: Uuid) -> SqlxResult<Vec<CartItem>> {
    sqlx::query_as::<_, CartItem>(
        r#"
        SELECT id, cart_id, menu_item_id, menu_version_at_add, quantity,
               customizations, notes, created_at
        FROM cart_items
        WHERE cart_id = $1
        ORDER BY created_at
        "#,
    )
    .bind(cart_id)
    .fetch_all(db)
    .await
}

/// Add an item to cart. If the same menu_item_id + customizations + notes
/// already exists, increment quantity instead of inserting a new row.
pub async fn add_item(
    db: &PgPool,
    cart_id: Uuid,
    menu_item_id: Uuid,
    menu_version_at_add: i32,
    quantity: i32,
    customizations: &serde_json::Value,
    notes: Option<&str>,
) -> SqlxResult<CartItem> {
    sqlx::query_as::<_, CartItem>(
        r#"
        INSERT INTO cart_items (cart_id, menu_item_id, menu_version_at_add, quantity, customizations, notes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, cart_id, menu_item_id, menu_version_at_add, quantity, customizations, notes, created_at
        "#,
    )
    .bind(cart_id)
    .bind(menu_item_id)
    .bind(menu_version_at_add)
    .bind(quantity)
    .bind(customizations)
    .bind(notes)
    .fetch_one(db)
    .await
}

/// Update quantity / notes on an existing cart item.
pub async fn update_item(
    db: &PgPool,
    item_id: Uuid,
    quantity: Option<i32>,
    notes: Option<&str>,
) -> SqlxResult<CartItem> {
    sqlx::query_as::<_, CartItem>(
        r#"
        UPDATE cart_items
        SET quantity = COALESCE($2, quantity),
            notes = COALESCE($3, notes)
        WHERE id = $1
        RETURNING id, cart_id, menu_item_id, menu_version_at_add, quantity, customizations, notes, created_at
        "#,
    )
    .bind(item_id)
    .bind(quantity)
    .bind(notes)
    .fetch_one(db)
    .await
}

/// Delete a single cart item.
pub async fn delete_item(db: &PgPool, item_id: Uuid) -> SqlxResult<()> {
    sqlx::query("DELETE FROM cart_items WHERE id = $1")
        .bind(item_id)
        .execute(db)
        .await?;
    Ok(())
}

/// Delete all items in a cart (clear cart).
pub async fn clear_items(db: &PgPool, cart_id: Uuid) -> SqlxResult<()> {
    sqlx::query("DELETE FROM cart_items WHERE cart_id = $1")
        .bind(cart_id)
        .execute(db)
        .await?;
    Ok(())
}
