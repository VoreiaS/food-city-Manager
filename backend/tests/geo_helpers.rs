//! Tests for the haversine distance function and restaurant helpers.

use chrono::{DateTime, Utc};
use food_city_backend::domain::restaurant::{haversine_m, is_open_at};
use serde_json::json;

#[test]
fn haversine_same_point_is_zero() {
    let d = haversine_m(6.9271, 79.8612, 6.9271, 79.8612);
    assert_eq!(d, 0);
}

#[test]
fn haversine_known_distance() {
    // Colombo to Kandy is ~94km as the crow flies
    let d = haversine_m(6.9271, 79.8612, 7.2906, 80.6337);
    assert!((d - 94_000).abs() < 5_000, "expected ~94km, got {}m", d);
}

#[test]
fn haversine_short_distance() {
    // Two points ~250m apart
    let d = haversine_m(6.9271, 79.8612, 6.9293, 79.8612);
    assert!((d - 240).abs() < 30, "expected ~240m, got {}m", d);
}

#[test]
fn haversine_is_symmetric() {
    let d1 = haversine_m(6.9, 79.8, 7.0, 80.0);
    let d2 = haversine_m(7.0, 80.0, 6.9, 79.8);
    assert_eq!(d1, d2);
}

#[test]
fn is_open_at_with_valid_hours() {
    let hours = json!({
        "monday": [{"open": "09:00", "close": "22:00"}],
        "tuesday": [{"open": "09:00", "close": "22:00"}],
    });
    // Monday at noon
    let monday_noon = DateTime::parse_from_rfc3339("2026-07-06T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(is_open_at(&hours, monday_noon));

    // Monday at 23:00 (after close)
    let monday_late = DateTime::parse_from_rfc3339("2026-07-06T23:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(!is_open_at(&hours, monday_late));
}

#[test]
fn is_open_at_returns_false_for_missing_day() {
    let hours = json!({
        "monday": [{"open": "09:00", "close": "22:00"}],
    });
    // Sunday (not in hours)
    let sunday = DateTime::parse_from_rfc3339("2026-07-05T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(!is_open_at(&hours, sunday));
}

#[test]
fn is_open_at_returns_false_for_empty_hours() {
    let hours = json!({});
    let now = Utc::now();
    assert!(!is_open_at(&hours, now));
}

#[test]
fn is_open_at_handles_multiple_slots_per_day() {
    let hours = json!({
        "monday": [
            {"open": "09:00", "close": "14:00"},
            {"open": "17:00", "close": "22:00"}
        ],
    });
    let monday_lunch = DateTime::parse_from_rfc3339("2026-07-06T12:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(is_open_at(&hours, monday_lunch));

    let monday_afternoon = DateTime::parse_from_rfc3339("2026-07-06T15:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(!is_open_at(&hours, monday_afternoon));

    let monday_dinner = DateTime::parse_from_rfc3339("2026-07-06T19:00:00Z")
        .unwrap()
        .with_timezone(&Utc);
    assert!(is_open_at(&hours, monday_dinner));
}
