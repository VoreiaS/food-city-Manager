//! Restaurant domain types.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Type;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "restaurant_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum RestaurantStatus {
    PendingVerification,
    Active,
    Paused,
    Closing,
    Closed,
}

impl Default for RestaurantStatus {
    fn default() -> Self {
        Self::PendingVerification
    }
}

/// Full restaurant record (DB shape).
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Restaurant {
    pub id: Uuid,
    pub owner_user_id: Uuid,
    pub group_id: Option<Uuid>,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub cuisine_types: Vec<String>,
    pub price_range: i16,
    pub logo_url: Option<String>,
    pub cover_url: Option<String>,
    pub lat: f64,
    pub lng: f64,
    pub delivery_radius_m: i32,
    pub delivery_fee_cents: i64,
    pub min_order_cents: i64,
    pub status: RestaurantStatus,
    pub hours_json: serde_json::Value,
    pub rating_avg: Option<f64>,
    pub rating_count: i64,
    pub stripe_account_id: Option<String>,
    pub commission_percent: rust_decimal::Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

/// Lightweight restaurant for list/search results (no DB-heavy fields).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestaurantCard {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub cuisine_types: Vec<String>,
    pub price_range: i16,
    pub logo_url: Option<String>,
    pub cover_url: Option<String>,
    pub delivery_fee_cents: i64,
    pub min_order_cents: i64,
    pub rating_avg: Option<f64>,
    pub rating_count: i64,
    pub status: RestaurantStatus,
    pub is_open: bool,
    pub distance_m: Option<i64>,
    pub delivery_eta_min: Option<i32>,
}

/// Query params for restaurant search.
#[derive(Debug, Clone, Deserialize)]
pub struct RestaurantQuery {
    pub lat: Option<f64>,
    pub lng: Option<f64>,
    pub radius_m: Option<i32>,
    pub cuisine: Option<String>,
    pub price_range: Option<i16>,
    pub veg_only: Option<bool>,
    pub rating_min: Option<f64>,
    pub sort: Option<String>,
    pub q: Option<String>,
    pub page: Option<u32>,
    pub page_size: Option<u32>,
}

impl RestaurantQuery {
    pub fn page_num(&self) -> u32 {
        self.page.unwrap_or(1).max(1)
    }
    pub fn page_size_num(&self) -> u32 {
        self.page_size.unwrap_or(20).clamp(1, 100)
    }
    pub fn offset(&self) -> i64 {
        ((self.page_num() - 1) * self.page_size_num()) as i64
    }
    pub fn limit(&self) -> i64 {
        self.page_size_num() as i64
    }
}

/// Haversine distance in meters between two lat/lng points.
pub fn haversine_m(lat1: f64, lng1: f64, lat2: f64, lng2: f64) -> i64 {
    const R: f64 = 6_371_000.0; // earth radius in meters
    let lat1r = lat1.to_radians();
    let lat2r = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlng = (lng2 - lng1).to_radians();
    let a = (dlat / 2.0).sin().powi(2)
        + lat1r.cos() * lat2r.cos() * (dlng / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().asin();
    (R * c) as i64
}

/// Check if restaurant is open at the given time based on hours_json.
/// hours_json format: { "mon": [{"open": "09:00", "close": "22:00"}], ... }
pub fn is_open_at(hours_json: &serde_json::Value, at: DateTime<Utc>) -> bool {
    let obj = match hours_json.as_object() {
        Some(o) => o,
        None => return false,
    };
    let weekday = at.format("%A").to_string().to_lowercase();
    let day_arr = match obj.get(&weekday).and_then(|v| v.as_array()) {
        Some(arr) => arr,
        None => return false,
    };
    let now_time = at.format("%H:%M").to_string();
    for slot in day_arr {
        let open = slot.get("open").and_then(|v| v.as_str()).unwrap_or("00:00");
        let close = slot.get("close").and_then(|v| v.as_str()).unwrap_or("23:59");
        if now_time.as_str() >= open && now_time.as_str() <= close {
            return true;
        }
    }
    false
}

impl Restaurant {
    /// Returns true if customer at (lat,lng) is within delivery_radius_m.
    pub fn delivers_to(&self, lat: f64, lng: f64) -> bool {
        let dist = haversine_m(self.lat, self.lng, lat, lng);
        dist <= self.delivery_radius_m as i64
    }

    pub fn is_open(&self, at: DateTime<Utc>) -> bool {
        is_open_at(&self.hours_json, at)
    }
}
