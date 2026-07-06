//! v1 API routes.
//!
//! Each module exposes a `routes()` function returning a `Router<AppState>`.

pub mod auth;
pub mod restaurants;
pub mod menus;
pub mod cart;
pub mod orders;
pub mod drivers;
pub mod reviews;
pub mod loyalty;
pub mod payments;
pub mod admin;
pub mod ws;
pub mod addresses;
pub mod restaurant_dashboard;
pub mod disputes;
pub mod promos;

use axum::Router;

use super::AppState;

pub fn routes() -> Router<AppState> {
    Router::new()
        .merge(auth::routes())
        .merge(restaurants::public_routes())
        .merge(menus::public_routes())
        .merge(cart::routes())
        .merge(orders::routes())
        .merge(addresses::routes())
        .merge(payments::routes())
        .merge(drivers::routes())
        .merge(restaurant_dashboard::routes())
        .merge(reviews::routes())
        .merge(loyalty::routes())
        .merge(disputes::routes())
        .merge(promos::routes())
        .merge(admin::routes())
        .merge(ws::routes())
}
