//! JWT issuance and verification.
//!
//! Uses the application's JWT secret. Access tokens are short-lived (15min);
//! refresh tokens are long-lived (7d). Token type is embedded in claims
//! to prevent misuse (e.g., refresh token used as access).

use chrono::Utc;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};

use crate::config::Config;
use crate::domain::user::UserRole;
use crate::error::{AppError, AppResult};

static CONFIG: OnceCell<JwtConfig> = OnceCell::new();

struct JwtConfig {
    #[allow(dead_code)]
    secret: String,
    issuer: String,
    audience: String,
    encoding: EncodingKey,
    decoding: DecodingKey,
}

pub fn init(cfg: &Config) {
    let secret = cfg.jwt.secret.clone();
    let _ = CONFIG.set(JwtConfig {
        encoding: EncodingKey::from_secret(secret.as_bytes()),
        decoding: DecodingKey::from_secret(secret.as_bytes()),
        secret,
        issuer: cfg.jwt.issuer.clone(),
        audience: cfg.jwt.audience.clone(),
    });
}

fn cfg() -> Result<&'static JwtConfig, AppError> {
    CONFIG.get().ok_or_else(|| AppError::internal("JWT not initialized — call jwt::init() at startup"))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TokenType {
    Access,
    Refresh,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub email: String,
    pub role: UserRole,
    pub token_type: TokenType,
    pub exp: i64,
    pub iat: i64,
    pub iss: String,
    pub aud: String,
}

pub fn sign_access(claims: &Claims) -> AppResult<String> {
    let c = cfg()?;
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some("access".into());
    encode(&header, claims, &c.encoding).map_err(|e| AppError::internal(format!("jwt encode: {}", e)))
}

pub fn sign_refresh(claims: &Claims) -> AppResult<String> {
    let c = cfg()?;
    let mut header = Header::new(Algorithm::HS256);
    header.kid = Some("refresh".into());
    encode(&header, claims, &c.encoding).map_err(|e| AppError::internal(format!("jwt encode: {}", e)))
}

pub fn verify_access(token: &str) -> AppResult<Claims> {
    verify(token, TokenType::Access)
}

pub fn verify_refresh(token: &str) -> AppResult<Claims> {
    verify(token, TokenType::Refresh)
}

fn verify(token: &str, expected_type: TokenType) -> AppResult<Claims> {
    let c = cfg()?;
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_issuer(&[&c.issuer]);
    validation.set_audience(&[&c.audience]);
    validation.validate_exp = true;

    let data = decode::<Claims>(token, &c.decoding, &validation)
        .map_err(|e| AppError::unauthenticated(format!("jwt invalid: {}", e)))?;

    if data.claims.token_type != expected_type {
        return Err(AppError::unauthenticated("wrong token type"));
    }

    let _ = Utc::now(); // could compare exp explicitly but Validation already does
    Ok(data.claims)
}
