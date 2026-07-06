//! Repository modules (one per aggregate).
//!
//! Each repo exposes async functions taking `&PgPool` and returning
//! domain types. No business logic here — just DB queries.

pub mod user_repo;
pub mod restaurant_repo;
pub mod menu_repo;
pub mod cart_repo;
pub mod address_repo;
pub mod order_repo;
pub mod payment_repo;
pub mod driver_repo;
pub mod review_repo;
pub mod loyalty_repo;
pub mod dispute_repo;
pub mod promo_repo;
