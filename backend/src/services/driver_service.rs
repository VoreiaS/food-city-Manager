//! Driver service.

use uuid::Uuid;

use crate::db::repos::driver_repo;
use crate::domain::driver::{Driver, DriverStatus};
use crate::error::{AppError, AppResult};
use crate::services::realtime_service;

pub async fn go_online(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
    vehicle_type: Option<String>,
) -> AppResult<Driver> {
    // Validate vehicle type
    let vt = vehicle_type.unwrap_or_else(|| "bike".to_string());
    let vt = vt.to_lowercase();
    if !["bike", "scooter", "car"].contains(&vt.as_str()) {
        return Err(AppError::validation(
            "vehicle_type must be one of: bike, scooter, car",
        ));
    }

    let mut driver = match driver_repo::find_by_user(db, user_id).await? {
        Some(d) => d,
        None => {
            driver_repo::create(db, user_id, &vt, None).await?
        }
    };
    if driver.status == DriverStatus::Offline {
        driver = driver_repo::transition_status(
            db,
            driver.id,
            DriverStatus::Offline,
            DriverStatus::Available,
        )
        .await?
        .ok_or_else(|| AppError::conflict("driver not offline"))?;
    } else if driver.status != DriverStatus::Available {
        return Err(AppError::conflict(format!(
            "cannot go online while status is {:?}",
            driver.status
        )));
    }
    realtime_service::set_driver_available(redis, driver.id).await?;
    realtime_service::heartbeat_driver(redis, driver.id, 60).await?;
    Ok(driver)
}

pub async fn go_offline(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
) -> AppResult<Driver> {
    let driver = driver_repo::find_by_user(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver profile not found"))?;
    if driver.status == DriverStatus::Available {
        let updated = driver_repo::transition_status(
            db,
            driver.id,
            DriverStatus::Available,
            DriverStatus::Offline,
        )
        .await?
        .ok_or_else(|| AppError::conflict("driver not available"))?;
        realtime_service::set_driver_unavailable(redis, driver.id).await?;
        Ok(updated)
    } else {
        Err(AppError::conflict(format!(
            "cannot go offline while status is {:?}; only available drivers can go offline",
            driver.status
        )))
    }
}

pub async fn update_location(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
    lat: f64,
    lng: f64,
    heading: Option<f64>,
    speed_kph: Option<f64>,
) -> AppResult<()> {
    let driver = driver_repo::find_by_user(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver profile not found"))?;
    driver_repo::update_location(db, driver.id, lat, lng).await?;
    realtime_service::publish_driver_location(
        redis,
        driver.id,
        driver.current_order_id,
        lat,
        lng,
        heading,
        speed_kph,
    )
    .await?;
    Ok(())
}

pub async fn get_profile(db: &sqlx::PgPool, user_id: Uuid) -> AppResult<Driver> {
    driver_repo::find_by_user(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver profile not found; go online first"))
}

/// Accept an order. Atomic transition: only succeeds if driver is `available`
/// AND the order is in `ready` state (food prepared, waiting for driver).
pub async fn accept_order(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<crate::domain::order::OrderDto> {
    let driver = driver_repo::find_by_user(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver profile not found"))?;

    // Verify the order exists and is in 'ready' state (food prepared, needs driver)
    let order = crate::db::repos::order_repo::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    if order.status != crate::domain::order::OrderStatus::Ready {
        return Err(AppError::conflict(format!(
            "order is not ready for pickup (current status: {:?}); can only accept orders in 'ready' state",
            order.status
        )));
    }
    // Verify no driver is already assigned
    if order.driver_id.is_some() {
        return Err(AppError::conflict("order already has a driver assigned"));
    }

    // Atomic transition: available → assigned
    let updated = driver_repo::transition_status(
        db,
        driver.id,
        DriverStatus::Available,
        DriverStatus::Assigned,
    )
    .await?
    .ok_or_else(|| AppError::conflict("driver is not available (already on another order?)"))?;

    realtime_service::set_driver_unavailable(redis, driver.id).await?;

    // Assign driver to order
    let _order = crate::db::repos::order_repo::assign_driver(db, order_id, driver.id)
        .await?
        .ok_or_else(|| AppError::conflict("order already has a driver assigned"))?;

    // Link driver ↔ order
    driver_repo::set_current_order(db, driver.id, Some(order_id)).await?;

    // Append event
    let seq = crate::db::repos::order_repo::next_event_sequence(db, order_id).await?;
    crate::db::repos::order_repo::append_event(
        db,
        order_id,
        seq,
        "driver.assigned",
        &serde_json::json!({"driver_id": driver.id, "driver_user_id": driver.user_id}),
    )
    .await?;

    let _ = updated;
    crate::services::order_service::get_order(db, order_id).await
}

pub async fn pickup_order(
    db: &sqlx::PgPool,
    _redis: &deadpool_redis::Pool,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<crate::domain::order::OrderDto> {
    // Ownership check: verify this driver is assigned to the order
    verify_driver_owns_order(db, user_id, order_id).await?;
    crate::services::order_service::mark_picked_up(db, order_id).await
}

pub async fn deliver_order(
    db: &sqlx::PgPool,
    redis: &deadpool_redis::Pool,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<crate::domain::order::OrderDto> {
    // Ownership check: verify this driver is assigned to the order
    verify_driver_owns_order(db, user_id, order_id).await?;
    let order = crate::services::order_service::mark_delivered(db, order_id).await?;

    // Driver back to available
    if let Some(driver) = driver_repo::find_by_user(db, user_id).await? {
        let _ = driver_repo::transition_status(
            db,
            driver.id,
            DriverStatus::Assigned, // from any non-offline status really; we use Assigned as placeholder
            DriverStatus::Available,
        )
        .await;
        driver_repo::set_current_order(db, driver.id, None).await?;
        realtime_service::set_driver_available(redis, driver.id).await?;
    }
    Ok(order)
}

/// Verify that the driver (by user_id) is currently assigned to the order.
/// Prevents a driver from marking another driver's order as picked up / delivered.
async fn verify_driver_owns_order(
    db: &sqlx::PgPool,
    user_id: Uuid,
    order_id: Uuid,
) -> AppResult<()> {
    let driver = driver_repo::find_by_user(db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("driver profile not found"))?;
    let order = crate::db::repos::order_repo::find_by_id(db, order_id)
        .await?
        .ok_or_else(|| AppError::not_found("order"))?;
    match order.driver_id {
        Some(assigned_id) if assigned_id == driver.id => Ok(()),
        _ => Err(AppError::forbidden(
            "you are not assigned to this order",
        )),
    }
}

/// Find drivers near a restaurant that can be assigned to an order.
/// Returns the closest available driver IDs.
pub async fn find_candidate_drivers(
    redis: &deadpool_redis::Pool,
    lat: f64,
    lng: f64,
    radius_m: f64,
    limit: usize,
) -> AppResult<Vec<(Uuid, f64)>> {
    realtime_service::find_nearby_drivers(redis, lat, lng, radius_m, limit).await
}
