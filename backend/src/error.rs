//! Unified error type for the application.
//!
//! `AppError` converts into an HTTP response via `IntoResponse`.
//! All handlers return `Result<T, AppError>`. Errors carry:
//!   - HTTP status code
//!   - Stable error code string (for client-side handling)
//!   - Human-readable message
//!   - Optional field-level details (for validation errors)
//!   - Request ID (injected by middleware)

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::fmt;
use thiserror::Error;
use tracing::error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("unauthenticated: {0}")]
    Unauthenticated(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("business rule violated: {0}")]
    BusinessRule(String),

    #[error("rate limited")]
    RateLimited,

    #[error("service unavailable: {0}")]
    ServiceUnavailable(String),

    #[error("stripe error: {0}")]
    Stripe(String),

    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("redis error: {0}")]
    Redis(#[from] redis::RedisError),

    #[error("redis pool error: {0}")]
    RedisPool(#[from] deadpool_redis::PoolError),

    #[error("jwt error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    pub fn status_code(&self) -> StatusCode {
        match self {
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Unauthenticated(_) => StatusCode::UNAUTHORIZED,
            Self::Forbidden(_) => StatusCode::FORBIDDEN,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Conflict(_) => StatusCode::CONFLICT,
            Self::BusinessRule(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::ServiceUnavailable(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Stripe(_) => StatusCode::BAD_GATEWAY,
            Self::Database(_e) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Redis(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::RedisPool(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Jwt(_) => StatusCode::UNAUTHORIZED,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    pub fn code(&self) -> &'static str {
        match self {
            Self::Validation(_) => "validation_error",
            Self::Unauthenticated(_) => "unauthenticated",
            Self::Forbidden(_) => "forbidden",
            Self::NotFound(_) => "not_found",
            Self::Conflict(_) => "conflict",
            Self::BusinessRule(_) => "business_rule_violation",
            Self::RateLimited => "rate_limited",
            Self::ServiceUnavailable(_) => "service_unavailable",
            Self::Stripe(_) => "stripe_error",
            Self::Database(_) => "database_error",
            Self::Redis(_) => "redis_error",
            Self::RedisPool(_) => "redis_pool_error",
            Self::Jwt(_) => "jwt_error",
            Self::Internal(_) => "internal_error",
        }
    }

    /// Should the message be exposed to the client?
    pub fn safe_message(&self) -> String {
        match self {
            Self::Validation(m)
            | Self::Unauthenticated(m)
            | Self::Forbidden(m)
            | Self::NotFound(m)
            | Self::Conflict(m)
            | Self::BusinessRule(m)
            | Self::ServiceUnavailable(m)
            | Self::Stripe(m) => m.clone(),
            Self::RateLimited => "rate limited".into(),
            Self::Database(_) => "database error".into(),
            Self::Redis(_) => "cache error".into(),
            Self::RedisPool(_) => "cache busy, try again".into(),
            Self::Jwt(_) => "invalid token".into(),
            Self::Internal(_) => "internal server error".into(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub error: ErrorDetail,
}

#[derive(Debug, Serialize)]
pub struct ErrorDetail {
    pub code: &'static str,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub request_id: Option<String>,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let code = self.code();

        // Log internal errors with full detail; client-facing ones at info.
        match &self {
            Self::Database(e) => error!(error = ?e, "database error"),
            Self::Redis(e) => error!(error = ?e, "redis error"),
            Self::RedisPool(e) => error!(error = ?e, "redis pool error"),
            Self::Internal(msg) => error!(msg = %msg, "internal error"),
            Self::Stripe(msg) => error!(msg = %msg, "stripe error"),
            _ => tracing::info!(code = code, msg = %self.safe_message(), "client error"),
        }

        let body = ErrorBody {
            error: ErrorDetail {
                code,
                message: self.safe_message(),
                details: None,
                request_id: None,
            },
        };

        (status, Json(body)).into_response()
    }
}

// Convenience constructors --------------------------------------------------

impl AppError {
    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn unauthenticated(msg: impl Into<String>) -> Self {
        Self::Unauthenticated(msg.into())
    }

    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::Forbidden(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn business_rule(msg: impl Into<String>) -> Self {
        Self::BusinessRule(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

impl From<anyhow::Error> for AppError {
    fn from(e: anyhow::Error) -> Self {
        Self::Internal(e.to_string())
    }
}

impl fmt::Display for ErrorDetail {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}
