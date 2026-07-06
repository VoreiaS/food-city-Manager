//! Menu service — assembles the grouped menu response for a restaurant.

use std::collections::HashMap;

use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::menu_repo;
use crate::domain::menu::{
    MenuCategoryDto, MenuItemCustomizationDto, MenuItemCustomizationOptionDto, MenuItemDto,
    MenuResponse,
};
use crate::error::{AppError, AppResult};

/// Re-export of `MenuResponse` for use in handler signatures without
/// importing the full domain module.
pub type MenuResponseView = MenuResponse;

pub async fn get_active_menu(db: &PgPool, restaurant_id: Uuid) -> AppResult<MenuResponse> {
    let version = menu_repo::active_version(db, restaurant_id)
        .await?
        .ok_or_else(|| AppError::not_found("no active menu for restaurant"))?;
    let categories = build_categories(db, version.id).await?;

    Ok(MenuResponse {
        restaurant_id,
        menu_version: version.version,
        categories,
    })
}

async fn build_categories(db: &PgPool, version_id: Uuid) -> AppResult<Vec<MenuCategoryDto>> {
    let categories = menu_repo::categories(db, version_id).await?;
    if categories.is_empty() {
        return Ok(vec![]);
    }
    let category_ids: Vec<Uuid> = categories.iter().map(|c| c.0).collect();
    let items = menu_repo::items_for_categories(db, &category_ids).await?;

    // Load customizations + options in two passes.
    let item_ids: Vec<Uuid> = items.iter().map(|i| i.id).collect();
    let customizations = menu_repo::customizations_for_items(db, &item_ids).await?;
    let customization_ids: Vec<Uuid> = customizations.iter().map(|c| c.id).collect();
    let options = menu_repo::options_for_customizations(db, &customization_ids).await?;

    // Index options by customization_id.
    let mut options_by_cust: HashMap<Uuid, Vec<MenuItemCustomizationOptionDto>> = HashMap::new();
    for opt in options {
        options_by_cust
            .entry(opt.customization_id)
            .or_default()
            .push(MenuItemCustomizationOptionDto {
                id: opt.id,
                name: opt.name,
                price_cents: opt.price_cents,
                is_default: opt.is_default,
                sort_order: opt.sort_order,
            });
    }

    // Index customizations by item_id.
    let mut cust_by_item: HashMap<Uuid, Vec<MenuItemCustomizationDto>> = HashMap::new();
    for c in customizations {
        let opts = options_by_cust.remove(&c.id).unwrap_or_default();
        cust_by_item
            .entry(c.item_id)
            .or_default()
            .push(MenuItemCustomizationDto {
                id: c.id,
                name: c.name,
                is_required: c.is_required,
                max_select: c.max_select,
                sort_order: c.sort_order,
                options: opts,
            });
    }

    // Index items by category_id (DTOs include customizations).
    let mut items_by_cat: HashMap<Uuid, Vec<MenuItemDto>> = HashMap::new();
    for item in items {
        let custs = cust_by_item.remove(&item.id).unwrap_or_default();
        let dto = MenuItemDto::from_menu_item(item, custs);
        items_by_cat.entry(dto.category_id).or_default().push(dto);
    }

    // Compose categories with their items.
    let mut result: Vec<MenuCategoryDto> = categories
        .into_iter()
        .map(|(id, name, sort_order)| MenuCategoryDto {
            id,
            name,
            sort_order,
            items: items_by_cat.remove(&id).unwrap_or_default(),
        })
        .collect();
    result.sort_by_key(|c| c.sort_order);
    Ok(result)
}
