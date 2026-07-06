//! Driver domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "driver_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum DriverStatus {
    Offline,
    Available,
    Assigned,
    EnRoute,
    AtRestaurant,
    PickedUp,
    Delivering,
    Delivered,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Driver {
    pub id: Uuid,
    pub user_id: Uuid,
    pub vehicle_type: String,
    pub license_plate: Option<String>,
    pub current_lat: Option<f64>,
    pub current_lng: Option<f64>,
    pub status: DriverStatus,
    pub current_order_id: Option<Uuid>,
    pub rating_avg: Option<f64>,
    pub rating_count: i64,
    pub acceptance_rate: f64,
    pub total_deliveries: i64,
    pub stripe_account_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
