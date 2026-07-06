//! Driver repository.

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::driver::{Driver, DriverStatus};

pub async fn find_by_user(db: &PgPool, user_id: Uuid) -> SqlxResult<Option<Driver>> {
    sqlx::query_as::<_, Driver>(
        r#"
        SELECT id, user_id, vehicle_type, license_plate, current_lat, current_lng,
               status as "status",
               current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
               stripe_account_id, created_at, updated_at
        FROM drivers WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(db)
    .await
}

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<Driver>> {
    sqlx::query_as::<_, Driver>(
        r#"
        SELECT id, user_id, vehicle_type, license_plate, current_lat, current_lng,
               status as "status",
               current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
               stripe_account_id, created_at, updated_at
        FROM drivers WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await
}

/// Create a driver row tied to a user. Called on first "go online".
pub async fn create(
    db: &PgPool,
    user_id: Uuid,
    vehicle_type: &str,
    license_plate: Option<&str>,
) -> SqlxResult<Driver> {
    sqlx::query_as::<_, Driver>(
        r#"
        INSERT INTO drivers (user_id, vehicle_type, license_plate)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, vehicle_type, license_plate, current_lat, current_lng,
                  status as "status",
                  current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
                  stripe_account_id, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(vehicle_type)
    .bind(license_plate)
    .fetch_one(db)
    .await
}

/// Atomically transition driver status. Only succeeds if from_status matches.
pub async fn transition_status(
    db: &PgPool,
    id: Uuid,
    from: DriverStatus,
    to: DriverStatus,
) -> SqlxResult<Option<Driver>> {
    sqlx::query_as::<_, Driver>(
        r#"
        UPDATE drivers
        SET status = $3, updated_at = NOW()
        WHERE id = $1 AND status = $2
        RETURNING id, user_id, vehicle_type, license_plate, current_lat, current_lng,
                  status as "status",
                  current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
                  stripe_account_id, created_at, updated_at
        "#,
    )
    .bind(id)
    .bind(from)
    .bind(to)
    .fetch_optional(db)
    .await
}

pub async fn update_location(
    db: &PgPool,
    id: Uuid,
    lat: f64,
    lng: f64,
) -> SqlxResult<()> {
    sqlx::query("UPDATE drivers SET current_lat = $2, current_lng = $3, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(lat)
        .bind(lng)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_current_order(
    db: &PgPool,
    id: Uuid,
    order_id: Option<Uuid>,
) -> SqlxResult<()> {
    sqlx::query("UPDATE drivers SET current_order_id = $2, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .bind(order_id)
        .execute(db)
        .await?;
    Ok(())
}

/// List drivers currently available (for matching).
pub async fn list_available(db: &PgPool, limit: i64) -> SqlxResult<Vec<Driver>> {
    sqlx::query_as::<_, Driver>(
        r#"
        SELECT id, user_id, vehicle_type, license_plate, current_lat, current_lng,
               status as "status",
               current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
               stripe_account_id, created_at, updated_at
        FROM drivers
        WHERE status = 'available'
        ORDER BY updated_at DESC
        LIMIT $1
        "#,
    )
    .bind(limit)
    .fetch_all(db)
    .await
}

/// Find drivers who are currently online (any non-offline status).
pub async fn list_online(db: &PgPool) -> SqlxResult<Vec<Driver>> {
    sqlx::query_as::<_, Driver>(
        r#"
        SELECT id, user_id, vehicle_type, license_plate, current_lat, current_lng,
               status as "status",
               current_order_id, rating_avg, rating_count, acceptance_rate, total_deliveries,
               stripe_account_id, created_at, updated_at
        FROM drivers
        WHERE status != 'offline'
        ORDER BY updated_at DESC
        "#,
    )
    .fetch_all(db)
    .await
}
