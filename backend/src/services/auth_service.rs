//! Auth service: register, login, JWT issuance.

use chrono::{Duration, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::db::repos::user_repo;
use crate::domain::user::{User, UserRole, UserPublic};
use crate::error::{AppError, AppResult};
use crate::utils::hash;
use crate::utils::jwt::{self, Claims, TokenType};

pub struct RegisterInput {
    pub email: String,
    pub phone: String,
    pub password: String,
    pub full_name: String,
    pub role: UserRole,
}

pub struct LoginInput {
    pub email: String,
    pub password: String,
}

pub struct AuthTokens {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
}

pub struct AuthResponse {
    pub user: UserPublic,
    pub tokens: AuthTokens,
}

pub async fn register(db: &PgPool, input: RegisterInput) -> AppResult<AuthResponse> {
    // Validate
    let email = input.email.trim().to_lowercase();
    if !email.contains('@') {
        return Err(AppError::validation("invalid email"));
    }
    if input.password.len() < 8 {
        return Err(AppError::validation("password must be at least 8 characters"));
    }
    // Password complexity: at least 1 letter + 1 digit
    let has_letter = input.password.chars().any(|c| c.is_alphabetic());
    let has_digit = input.password.chars().any(|c| c.is_numeric());
    if !has_letter || !has_digit {
        return Err(AppError::validation(
            "password must contain at least one letter and one digit",
        ));
    }
    if input.full_name.trim().is_empty() {
        return Err(AppError::validation("full_name is required"));
    }
    // Basic phone validation: at least 7 digits
    let digit_count = input.phone.chars().filter(|c| c.is_numeric()).count();
    if digit_count < 7 {
        return Err(AppError::validation("invalid phone number"));
    }

    // Check uniqueness
    if let Some(_existing) = user_repo::find_by_email(db, &email).await? {
        return Err(AppError::conflict("email already registered"));
    }

    // Hash password
    let password_hash = hash::hash_password(&input.password)?;

    // Insert
    let id = Uuid::now_v7();
    let user = user_repo::insert(
        db,
        id,
        &email,
        &input.phone,
        &input.full_name,
        input.role,
        &password_hash,
    )
    .await
    .map_err(|e| match e {
        sqlx::Error::Database(ref db_err) if db_err.is_unique_violation() => {
            AppError::conflict("email or phone already registered")
        }
        _ => AppError::Database(e),
    })?;

    let tokens = issue_tokens(&user)?;
    Ok(AuthResponse {
        user: user.into(),
        tokens,
    })
}

pub async fn login(db: &PgPool, input: LoginInput) -> AppResult<AuthResponse> {
    let email = input.email.trim().to_lowercase();
    let user = user_repo::find_by_email(db, &email)
        .await?
        .ok_or_else(|| AppError::unauthenticated("invalid email or password"))?;

    if !user.is_active {
        return Err(AppError::forbidden("account disabled"));
    }

    // Verify password
    let valid = hash::verify_password(&input.password, &user.password_hash)?;
    if !valid {
        return Err(AppError::unauthenticated("invalid email or password"));
    }

    let tokens = issue_tokens(&user)?;
    Ok(AuthResponse {
        user: user.into(),
        tokens,
    })
}

fn issue_tokens(user: &User) -> AppResult<AuthTokens> {
    let now = Utc::now();
    let access_exp = now + Duration::seconds(900);
    let refresh_exp = now + Duration::seconds(604800);

    let access_claims = Claims {
        sub: user.id.to_string(),
        email: user.email.clone(),
        role: user.role,
        token_type: TokenType::Access,
        exp: access_exp.timestamp(),
        iat: now.timestamp(),
        iss: "food-city".into(),
        aud: "food-city-client".into(),
    };

    let refresh_claims = Claims {
        exp: refresh_exp.timestamp(),
        token_type: TokenType::Refresh,
        ..access_claims.clone()
    };

    let access_token = jwt::sign_access(&access_claims)?;
    let refresh_token = jwt::sign_refresh(&refresh_claims)?;

    Ok(AuthTokens {
        access_token,
        refresh_token,
        expires_in: 900,
    })
}
