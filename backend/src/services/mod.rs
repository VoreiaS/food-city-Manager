//! Service layer — business logic. Calls repos, no HTTP/DB-direct.

pub mod auth_service;
pub mod restaurant_service;
pub mod menu_service;
pub mod cart_service;
pub mod order_service;
pub mod payment_service;
pub mod realtime_service;
pub mod driver_service;
pub mod cache_service;
pub mod stripe_client;
pub mod promo_service;
