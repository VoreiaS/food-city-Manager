//! Cart service — manages user cart, validates against restaurant + menu.

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::{cart_repo, menu_repo, restaurant_repo};
use crate::domain::cart::{
    AddCartItemRequest, CartResponse, CartStatus, CartItemCustomizationDto, CartItemDto,
};
use crate::error::{AppError, AppResult};

/// Get the user's current cart (or empty response if none).
pub async fn get_cart(db: &PgPool, user_id: Uuid) -> AppResult<Option<CartResponse>> {
    let cart = match cart_repo::find_active(db, user_id).await? {
        Some(c) => c,
        None => return Ok(None),
    };
    let restaurant = restaurant_repo::find_by_id(db, cart.restaurant_id)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))?;
    let items = build_cart_items(db, cart.id).await?;
    let subtotal = items.iter().map(|i| i.line_total_cents).sum::<i64>();
    let delivery_fee = restaurant.delivery_fee_cents;
    let total = subtotal + delivery_fee;
    let meets_min = subtotal >= restaurant.min_order_cents;

    Ok(Some(CartResponse {
        id: cart.id,
        user_id: cart.user_id,
        restaurant_id: cart.restaurant_id,
        restaurant_name: restaurant.name,
        status: cart.status,
        items,
        subtotal_cents: subtotal,
        delivery_fee_cents: delivery_fee,
        total_cents: total,
        min_order_cents: restaurant.min_order_cents,
        meets_min_order: meets_min,
    }))
}

/// Add an item to the cart. If the user has an active cart for a different
/// restaurant, returns a conflict error (frontend should confirm clear).
pub async fn add_item(
    db: &PgPool,
    user_id: Uuid,
    req: AddCartItemRequest,
) -> AppResult<CartResponse> {
    if req.quantity < 1 {
        return Err(AppError::validation("quantity must be >= 1"));
    }
    if req.quantity > 99 {
        return Err(AppError::validation("max 99 per item"));
    }

    // Validate menu item exists and belongs to the requested restaurant.
    let menu_version = menu_repo::active_version(db, req.restaurant_id)
        .await?
        .ok_or_else(|| AppError::not_found("no active menu for restaurant"))?;
    let item = menu_repo::find_item_by_id(db, req.menu_item_id)
        .await?
        .ok_or_else(|| AppError::not_found("menu item"))?;
    // Verify the item belongs to this restaurant's menu version.
    let categories = menu_repo::categories(db, menu_version.id).await?;
    let category_ids: Vec<Uuid> = categories.iter().map(|c| c.0).collect();
    if !category_ids.contains(&item.category_id) {
        return Err(AppError::conflict("menu item does not belong to this restaurant"));
    }

    // Validate item is available (not hidden / out of stock)
    if item.status != crate::domain::menu::MenuItemStatus::Available {
        return Err(AppError::business_rule(format!("{} is not available", item.name)));
    }

    // Validate in-stock
    if item.track_stock && item.stock_count < req.quantity {
        return Err(AppError::business_rule(format!(
            "only {} in stock for item {}",
            item.stock_count, item.name
        )));
    }

    // Validate restaurant is open + active (don't let customers build carts for closed restaurants)
    let restaurant = restaurant_repo::find_by_id(db, req.restaurant_id)
        .await?
        .ok_or_else(|| AppError::not_found("restaurant"))?;
    if restaurant.status != crate::domain::restaurant::RestaurantStatus::Active {
        return Err(AppError::business_rule("restaurant is not currently accepting orders"));
    }
    if !restaurant.is_open(chrono::Utc::now()) {
        return Err(AppError::business_rule("restaurant is closed right now"));
    }

    // Find or create active cart.
    let cart = match cart_repo::find_active(db, user_id).await? {
        Some(c) if c.restaurant_id == req.restaurant_id => c,
        Some(c) => {
            // Different restaurant — must clear first.
            return Err(AppError::Conflict(format!(
                "you have an active cart from a different restaurant ({}). Clear it first.",
                c.restaurant_id
            )));
        }
        None => cart_repo::create(db, user_id, req.restaurant_id).await?,
    };

    // Enforce max items per cart (50 distinct items)
    let existing_items = cart_repo::items(db, cart.id).await?;
    if existing_items.len() >= 50 {
        return Err(AppError::business_rule("cart is full (max 50 distinct items)"));
    }

    // Resolve customization option prices.
    let customizations_json = serde_json::to_value(&req.customizations)
        .map_err(|e| AppError::internal(format!("serde error: {}", e)))?;

    cart_repo::add_item(
        db,
        cart.id,
        req.menu_item_id,
        menu_version.version,
        req.quantity,
        &customizations_json,
        req.notes.as_deref(),
    )
    .await?;

    get_cart(db, user_id).await?.ok_or_else(|| AppError::internal("cart vanished"))
}

/// Update cart item (quantity / notes).
pub async fn update_item(
    db: &PgPool,
    user_id: Uuid,
    item_id: Uuid,
    quantity: Option<i32>,
    notes: Option<String>,
) -> AppResult<CartResponse> {
    if let Some(q) = quantity {
        if q < 1 {
            return Err(AppError::validation("quantity must be >= 1"));
        }
        if q > 99 {
            return Err(AppError::validation("max 99 per item"));
        }
    }
    let cart = cart_repo::find_active(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("no active cart"))?;
    // Ensure item belongs to user's cart.
    let items = cart_repo::items(db, cart.id).await?;
    let cart_item = items
        .iter()
        .find(|i| i.id == item_id)
        .ok_or_else(|| AppError::not_found("cart item"))?;

    // If increasing quantity, check stock for inventory-tracked items
    if let Some(q) = quantity {
        if q > cart_item.quantity {
            let menu_item = menu_repo::find_item_by_id(db, cart_item.menu_item_id)
                .await?
                .ok_or_else(|| AppError::not_found("menu item"))?;
            if menu_item.track_stock && menu_item.stock_count < q {
                return Err(AppError::business_rule(format!(
                    "only {} in stock for item {}",
                    menu_item.stock_count, menu_item.name
                )));
            }
        }
    }

    cart_repo::update_item(db, item_id, quantity, notes.as_deref()).await?;
    get_cart(db, user_id).await?.ok_or_else(|| AppError::internal("cart vanished"))
}

/// Delete a single cart item.
pub async fn delete_item(db: &PgPool, user_id: Uuid, item_id: Uuid) -> AppResult<CartResponse> {
    let cart = cart_repo::find_active(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("no active cart"))?;
    let items = cart_repo::items(db, cart.id).await?;
    if !items.iter().any(|i| i.id == item_id) {
        return Err(AppError::not_found("cart item"));
    }
    cart_repo::delete_item(db, item_id).await?;
    // If no items remain, abandon the cart.
    let remaining = cart_repo::items(db, cart.id).await?;
    if remaining.is_empty() {
        cart_repo::set_status(db, cart.id, CartStatus::Abandoned).await?;
        return Ok(CartResponse {
            id: cart.id,
            user_id,
            restaurant_id: cart.restaurant_id,
            restaurant_name: String::new(),
            status: CartStatus::Abandoned,
            items: vec![],
            subtotal_cents: 0,
            delivery_fee_cents: 0,
            total_cents: 0,
            min_order_cents: 0,
            meets_min_order: true,
        });
    }
    get_cart(db, user_id).await?.ok_or_else(|| AppError::internal("cart vanished"))
}

/// Clear the entire cart (delete cart + all items).
pub async fn clear(db: &PgPool, user_id: Uuid) -> AppResult<()> {
    if let Some(cart) = cart_repo::find_active(db, user_id).await? {
        cart_repo::delete(db, cart.id).await?;
    }
    Ok(())
}

// --- helpers ---

async fn build_cart_items(db: &PgPool, cart_id: Uuid) -> AppResult<Vec<CartItemDto>> {
    let items = cart_repo::items(db, cart_id).await?;
    let mut result = vec![];
    for item in items {
        // Resolve menu item data (name, price, image).
        let menu_item = menu_repo::find_item_by_id(db, item.menu_item_id)
            .await?
            .ok_or_else(|| AppError::not_found("menu item no longer exists"))?;

        // Resolve customization selections.
        let selections: Vec<CartItemCustomizationDto> = if item.customizations.is_null() {
            vec![]
        } else {
            let parsed: Vec<crate::domain::cart::AddCustomizationSelection> =
                serde_json::from_value(item.customizations.clone())
                    .map_err(|e| AppError::internal(format!("invalid customizations JSON: {}", e)))?;
            let mut custs = vec![];
            for sel in parsed {
                // Look up customization + option name + price.
                let custs_list = menu_repo::customizations_for_items(db, &[item.menu_item_id]).await?;
                let cust = custs_list.iter().find(|c| c.id == sel.customization_id);
                let opts = if let Some(c) = cust {
                    menu_repo::options_for_customizations(db, &[c.id]).await?
                } else {
                    vec![]
                };
                let opt = opts.iter().find(|o| o.id == sel.option_id);
                if let (Some(cust), Some(opt)) = (cust, opt) {
                    custs.push(CartItemCustomizationDto {
                        customization_id: cust.id,
                        customization_name: cust.name.clone(),
                        option_id: opt.id,
                        option_name: opt.name.clone(),
                        price_cents: opt.price_cents,
                    });
                }
            }
            custs
        };

        let addon_total: i64 = selections.iter().map(|s| s.price_cents).sum();
        let line_total = (menu_item.price_cents + addon_total) * item.quantity as i64;

        result.push(CartItemDto {
            id: item.id,
            cart_id: item.cart_id,
            menu_item_id: item.menu_item_id,
            menu_item_name: menu_item.name,
            menu_item_image_url: menu_item.image_url,
            base_price_cents: menu_item.price_cents,
            quantity: item.quantity,
            customizations: selections,
            notes: item.notes,
            line_total_cents: line_total,
        });
    }
    Ok(result)
}
