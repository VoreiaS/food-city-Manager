//! Authentication middleware.
//!
//! Extracts the JWT from `Authorization: Bearer <token>`, validates it,
//! and exposes the authenticated user via the `AuthUser` extractor.

use axum::{
    extract::FromRequestParts,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::domain::user::UserRole;
use crate::error::{AppError, AppResult};
use crate::utils::jwt;

/// Authenticated user extracted from the JWT. Available as an extractor
/// in any handler that needs the current user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthUser {
    pub user_id: String,
    pub email: String,
    pub role: UserRole,
    pub exp: DateTime<Utc>,
}

impl AuthUser {
    pub fn require_role(&self, role: UserRole) -> AppResult<()> {
        if self.role == role {
            Ok(())
        } else {
            Err(AppError::forbidden(format!(
                "role {:?} required, you are {:?}",
                role, self.role
            )))
        }
    }

    pub fn require_any_role(&self, roles: &[UserRole]) -> AppResult<()> {
        if roles.contains(&self.role) {
            Ok(())
        } else {
            Err(AppError::forbidden(format!(
                "one of {:?} required, you are {:?}",
                roles, self.role
            )))
        }
    }
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut axum::http::request::Parts,
        _state: &S,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = parts
            .headers
            .get(axum::http::header::AUTHORIZATION)
            .and_then(|v| v.to_str().ok())
            .ok_or_else(|| AppError::unauthenticated("missing Authorization header"))?;

        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or_else(|| AppError::unauthenticated("invalid Authorization scheme"))?;

        let claims = jwt::verify_access(token)
            .map_err(|e| AppError::unauthenticated(format!("invalid token: {}", e)))?;

        let user = AuthUser {
            user_id: claims.sub,
            email: claims.email,
            role: claims.role,
            exp: chrono::DateTime::from_timestamp(claims.exp, 0).unwrap_or_else(|| Utc::now()),
        };

        Ok(user)
    }
}
