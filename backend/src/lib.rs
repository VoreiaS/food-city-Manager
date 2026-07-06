//! Library entry point — re-exports modules for integration tests.

pub mod api;
pub mod config;
pub mod db;
pub mod domain;
pub mod error;
pub mod services;
pub mod utils;
pub mod workers;

pub use config::Config;
pub use error::{AppError, AppResult};
