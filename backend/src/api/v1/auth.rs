//! Auth endpoints: POST /register, /login, /refresh, /logout, GET /me.

use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::api::AppState;
use crate::domain::user::UserRole;
use crate::error::{AppError, AppResult};
use crate::services::auth_service::{self, AuthResponse, LoginInput, RegisterInput};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/refresh", post(refresh))
        .route("/auth/me", get(me))
}

#[derive(Debug, Deserialize, Validate)]
pub struct RegisterRequest {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 8, max = 128))]
    pub password: String,
    pub phone: String,
    pub full_name: String,
    #[serde(default = "default_role")]
    pub role: String,
}

fn default_role() -> String {
    "customer".into()
}

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponseDto {
    pub user: crate::domain::user::UserPublic,
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

impl From<AuthResponse> for AuthResponseDto {
    fn from(a: AuthResponse) -> Self {
        Self {
            user: a.user,
            access_token: a.tokens.access_token,
            refresh_token: a.tokens.refresh_token,
            expires_in: a.tokens.expires_in,
        }
    }
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<AuthResponseDto>> {
    req.validate()
        .map_err(|e| AppError::validation(e.to_string()))?;

    let role: UserRole = req
        .role
        .parse()
        .map_err(|e| AppError::validation(format!("invalid role: {}", e)))?;

    let input = RegisterInput {
        email: req.email,
        phone: req.phone,
        password: req.password,
        full_name: req.full_name,
        role,
    };

    let resp = auth_service::register(&state.db, input).await?;
    Ok(Json(resp.into()))
}

async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<Json<AuthResponseDto>> {
    let resp = auth_service::login(
        &state.db,
        LoginInput {
            email: req.email,
            password: req.password,
        },
    )
    .await?;
    Ok(Json(resp.into()))
}

async fn refresh(
    State(state): State<AppState>,
    Json(req): Json<RefreshRequest>,
) -> AppResult<Json<AuthResponseDto>> {
    use crate::utils::jwt;
    let claims = jwt::verify_refresh(&req.refresh_token)?;

    let user_id: uuid::Uuid = claims
        .sub
        .parse()
        .map_err(|_| AppError::unauthenticated("invalid subject"))?;

    let user = crate::db::repos::user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("user"))?;

    if !user.is_active {
        return Err(AppError::forbidden("account disabled"));
    }

    let now = chrono::Utc::now();
    let access_exp = now + chrono::Duration::seconds(state.config.jwt.access_ttl_secs as i64);
    let refresh_exp = now + chrono::Duration::seconds(state.config.jwt.refresh_ttl_secs as i64);

    let access_claims = jwt::Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role,
        token_type: jwt::TokenType::Access,
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
        iss: state.config.jwt.issuer.clone(),
        aud: state.config.jwt.audience.clone(),
    };

    let refresh_claims = jwt::Claims {
        exp: refresh_exp.timestamp(),
        token_type: jwt::TokenType::Refresh,
        ..access_claims.clone()
    };

    let access_token = jwt::sign_access(&access_claims)?;
    let refresh_token = jwt::sign_refresh(&refresh_claims)?;

    Ok(Json(AuthResponseDto {
        user: user.into(),
        access_token,
        refresh_token,
        expires_in: state.config.jwt.access_ttl_secs as i64,
    }))
}

async fn me(
    State(state): State<AppState>,
    auth: crate::api::mw::auth::AuthUser,
) -> AppResult<Json<crate::domain::user::UserPublic>> {
    let user_id: uuid::Uuid = auth
        .user_id
        .parse()
        .map_err(|_| AppError::internal("invalid user id in token"))?;

    let user = crate::db::repos::user_repo::find_by_id(&state.db, user_id)
        .await?
        .ok_or_else(|| AppError::not_found("user"))?;

    Ok(Json(user.into()))
}
