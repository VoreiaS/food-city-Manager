//! Realtime service — Redis pub/sub fan-out + WebSocket helpers.
//!
//! Architecture:
//! - Each order has a Redis pub/sub channel: `order:{id}:events`
//! - WS workers subscribe to channels for their connected clients
//! - State changes publish to the channel; all subscribers receive
//! - On reconnect, clients send `last_event_id` and we replay from
//!   the `order_events` table
//!
//! For driver locations:
//! - Drivers push to a Redis GEO set `drivers:locations` (hot path)
//! - On each push, also publish to `driver:{id}:location` channel
//! - For order tracking, the customer's WS subscribes to the order channel
//!   and the realtime worker periodically publishes driver location
//!   updates received from the driver channel to the order channel.

use std::sync::Arc;

use deadpool_redis::Pool as RedisPool;
use serde::Serialize;
use tracing::warn;

use crate::error::AppResult;

/// Channel name for an order's events.
pub fn order_channel(order_id: uuid::Uuid) -> String {
    format!("order:{}:events", order_id)
}

/// Channel name for a user's notifications.
pub fn user_channel(user_id: uuid::Uuid) -> String {
    format!("user:{}:notifications", user_id)
}

/// Channel name for a driver's location updates.
pub fn driver_channel(driver_id: uuid::Uuid) -> String {
    format!("driver:{}:location", driver_id)
}

/// Publish an event payload to an order's channel.
/// Subscribers (WS workers) will receive and forward to connected clients.
pub async fn publish_order_event<T: Serialize>(
    redis: &RedisPool,
    order_id: uuid::Uuid,
    event_type: &str,
    payload: &T,
) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let channel = order_channel(order_id);
    let body = serde_json::json!({
        "type": "order_event",
        "channel": channel,
        "event_type": event_type,
        "payload": payload,
    });
    let _: () = redis::cmd("PUBLISH")
        .arg(&channel)
        .arg(body.to_string())
        .query_async(&mut *conn)
        .await
        .map_err(|e| {
            warn!(error = ?e, "redis PUBLISH failed");
            e
        })?;
    Ok(())
}

/// Publish driver location update to both the driver channel and
/// (if assigned to an order) the order channel.
pub async fn publish_driver_location(
    redis: &RedisPool,
    driver_id: uuid::Uuid,
    order_id: Option<uuid::Uuid>,
    lat: f64,
    lng: f64,
    heading: Option<f64>,
    speed_kph: Option<f64>,
) -> AppResult<()> {
    let mut conn = redis.get().await?;

    // Update GEO set with driver's current location.
    // Redis GEO expects (longitude, latitude, member).
    let _: () = redis::cmd("GEOADD")
        .arg("drivers:locations")
        .arg(lng)
        .arg(lat)
        .arg(driver_id.to_string())
        .query_async(&mut *conn)
        .await?;

    // Publish to driver channel.
    let channel = driver_channel(driver_id);
    let body = serde_json::json!({
        "type": "driver_location",
        "driver_id": driver_id,
        "lat": lat,
        "lng": lng,
        "heading": heading,
        "speed_kph": speed_kph,
    });
    let _: () = redis::cmd("PUBLISH")
        .arg(&channel)
        .arg(body.to_string())
        .query_async(&mut *conn)
        .await?;

    // If driver is on an order, also publish to order channel.
    if let Some(oid) = order_id {
        let order_chan = order_channel(oid);
        let body2 = serde_json::json!({
            "type": "driver_location",
            "channel": order_chan,
            "driver_id": driver_id,
            "lat": lat,
            "lng": lng,
        });
        let _: () = redis::cmd("PUBLISH")
            .arg(&order_chan)
            .arg(body2.to_string())
            .query_async(&mut *conn)
            .await?;
    }

    Ok(())
}

/// Find available drivers within `radius_m` of (lat, lng), sorted by distance.
/// Returns driver IDs + distances.
pub async fn find_nearby_drivers(
    redis: &RedisPool,
    lat: f64,
    lng: f64,
    radius_m: f64,
    limit: usize,
) -> AppResult<Vec<(uuid::Uuid, f64)>> {
    let mut conn = redis.get().await?;
    // Use raw cmd since redis 0.27's typed geo_radius API is unstable across versions.
    // GEORADIUS key lng lat radius m ASC WITHDIST COUNT n
    let result: Vec<(String, f64)> = redis::cmd("GEORADIUS")
        .arg("drivers:locations")
        .arg(lng) // Redis GEO uses (lng, lat) order
        .arg(lat)
        .arg(radius_m)
        .arg("m")
        .arg("ASC")
        .arg("WITHDIST")
        .arg("COUNT")
        .arg(limit)
        .query_async(&mut *conn)
        .await?;
    let mut drivers = Vec::with_capacity(result.len());
    for (id_str, dist_m) in result {
        if let Ok(id) = uuid::Uuid::parse_str(&id_str) {
            drivers.push((id, dist_m));
        }
    }
    Ok(drivers)
}

/// Mark a driver as available (add to available-driver set).
pub async fn set_driver_available(
    redis: &RedisPool,
    driver_id: uuid::Uuid,
) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let _: () = redis::cmd("SADD")
        .arg("drivers:available")
        .arg(driver_id.to_string())
        .query_async(&mut *conn)
        .await?;
    Ok(())
}

/// Mark a driver as unavailable (remove from available-driver set).
pub async fn set_driver_unavailable(
    redis: &RedisPool,
    driver_id: uuid::Uuid,
) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let _: () = redis::cmd("SREM")
        .arg("drivers:available")
        .arg(driver_id.to_string())
        .query_async(&mut *conn)
        .await?;
    Ok(())
}

/// Set a driver heartbeat (TTL'd key). If expires, driver is considered offline.
pub async fn heartbeat_driver(
    redis: &RedisPool,
    driver_id: uuid::Uuid,
    ttl_secs: u64,
) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let key = format!("driver:hb:{}", driver_id);
    let _: () = redis::cmd("SETEX")
        .arg(&key)
        .arg(ttl_secs)
        .arg("1")
        .query_async(&mut *conn)
        .await?;
    Ok(())
}

/// Returns true if the driver's heartbeat is still alive.
pub async fn is_driver_alive(
    redis: &RedisPool,
    driver_id: uuid::Uuid,
) -> AppResult<bool> {
    let mut conn = redis.get().await?;
    let exists: bool = redis::cmd("EXISTS")
        .arg(format!("driver:hb:{}", driver_id))
        .query_async(&mut *conn)
        .await?;
    Ok(exists)
}

/// Wrap redis pool in Arc for sharing across tasks (already Arc internally,
/// but this is for clarity in callers).
pub fn shared(_pool: RedisPool) -> Arc<RedisPool> {
    Arc::new(_pool)
}
