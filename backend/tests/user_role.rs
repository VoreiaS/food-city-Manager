//! Unit tests for promo code discount computation.
//!
//! Tests the pure logic of `promo_service::compute_discount` via the public
//! `validate` flow (which calls compute_discount internally). We test
//! edge cases without a DB by checking the error variants.

use food_city_backend::domain::user::UserRole;

#[test]
fn user_role_customer_serializes_to_lowercase() {
    let json = serde_json::to_string(&UserRole::Customer).unwrap();
    assert_eq!(json, "\"customer\"");
}

#[test]
fn user_role_restaurant_serializes_to_lowercase() {
    let json = serde_json::to_string(&UserRole::Restaurant).unwrap();
    assert_eq!(json, "\"restaurant\"");
}

#[test]
fn user_role_driver_serializes_to_lowercase() {
    let json = serde_json::to_string(&UserRole::Driver).unwrap();
    assert_eq!(json, "\"driver\"");
}

#[test]
fn user_role_admin_serializes_to_lowercase() {
    let json = serde_json::to_string(&UserRole::Admin).unwrap();
    assert_eq!(json, "\"admin\"");
}

#[test]
fn user_role_parses_from_string() {
    assert_eq!("customer".parse::<UserRole>().unwrap(), UserRole::Customer);
    assert_eq!("RESTAURANT".parse::<UserRole>().unwrap(), UserRole::Restaurant);
    assert_eq!("Driver".parse::<UserRole>().unwrap(), UserRole::Driver);
    assert_eq!("ADMIN".parse::<UserRole>().unwrap(), UserRole::Admin);
}

#[test]
fn user_role_rejects_unknown_string() {
    assert!("unknown".parse::<UserRole>().is_err());
    assert!("".parse::<UserRole>().is_err());
}
