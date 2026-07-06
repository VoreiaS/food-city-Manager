//! Application configuration loaded from environment variables.
//!
//! Plain `std::env` reads with defaults. Avoids figment's nested-key
//! complexity. Production MUST override secrets (JWT_SECRET, STRIPE_SECRET_KEY,
//! DATABASE_URL, REDIS_URL).

use serde::{Deserialize, Serialize};
use std::env;
use std::time::Duration;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    pub app: AppConfig,
    pub database: DatabaseConfig,
    pub redis: RedisConfig,
    pub jwt: JwtConfig,
    pub cors: CorsConfig,
    pub stripe: StripeConfig,
    pub rate_limit: RateLimitConfig,
    pub geo: GeoConfig,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub env: String,
    pub port: u16,
    pub host: String,
    pub log_level: String,
    pub request_id_header: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub acquire_timeout_secs: u64,
    pub replica_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RedisConfig {
    pub url: String,
    pub pool_size: usize,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct JwtConfig {
    pub secret: String,
    pub issuer: String,
    pub audience: String,
    pub access_ttl_secs: u64,
    pub refresh_ttl_secs: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct CorsConfig {
    pub allowed_origins: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct StripeConfig {
    pub secret_key: String,
    pub webhook_secret: String,
    pub connect_client_id: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RateLimitConfig {
    pub enabled: bool,
    pub redis_url: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeoConfig {
    pub provider: String,
    pub api_url: String,
    pub api_key: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct FeatureFlags {
    pub realtime_ws: bool,
    pub driver_matching: bool,
    pub loyalty: bool,
}

impl Config {
    pub fn from_env() -> anyhow::Result<Self> {
        let _ = dotenvy::dotenv();

        fn env_or(key: &str, default: &str) -> String {
            env::var(key).unwrap_or_else(|_| default.into())
        }
        fn env_or_opt(key: &str) -> Option<String> {
            env::var(key).ok().filter(|s| !s.is_empty())
        }
        fn env_parse<T: std::str::FromStr>(key: &str, default: T) -> T {
            env::var(key)
                .ok()
                .and_then(|s| s.parse().ok())
                .unwrap_or(default)
        }
        fn env_bool(key: &str, default: bool) -> bool {
            env::var(key)
                .ok()
                .map(|s| matches!(s.to_ascii_lowercase().as_str(), "1" | "true" | "yes" | "on"))
                .unwrap_or(default)
        }

        Ok(Self {
            app: AppConfig {
                env: env_or("APP_ENV", "development"),
                port: env_parse("APP_PORT", 8080),
                host: env_or("APP_HOST", "0.0.0.0"),
                log_level: env_or("APP_LOG_LEVEL", "info"),
                request_id_header: env_or("APP_REQUEST_ID_HEADER", "X-Request-Id"),
            },
            database: DatabaseConfig {
                url: env_or(
                    "DATABASE_URL",
                    "postgres://foodcity:foodcity@localhost:5432/foodcity",
                ),
                max_connections: env_parse("DATABASE_MAX_CONNECTIONS", 20),
                acquire_timeout_secs: env_parse("DATABASE_ACQUIRE_TIMEOUT_SECS", 5),
                replica_url: env_or_opt("DATABASE_REPLICA_URL"),
            },
            redis: RedisConfig {
                url: env_or("REDIS_URL", "redis://localhost:6379/0"),
                pool_size: env_parse("REDIS_POOL_SIZE", 16),
            },
            jwt: JwtConfig {
                secret: env_or("JWT_SECRET", "dev-only-change-me"),
                issuer: env_or("JWT_ISSUER", "food-city"),
                audience: env_or("JWT_AUDIENCE", "food-city-client"),
                access_ttl_secs: env_parse("JWT_ACCESS_TTL_SECS", 900),
                refresh_ttl_secs: env_parse("JWT_REFRESH_TTL_SECS", 604800),
            },
            cors: CorsConfig {
                allowed_origins: env_or(
                    "CORS_ALLOWED_ORIGINS",
                    "http://localhost:5173,http://localhost:3000",
                ),
            },
            stripe: StripeConfig {
                secret_key: env_or("STRIPE_SECRET_KEY", ""),
                webhook_secret: env_or("STRIPE_WEBHOOK_SECRET", ""),
                connect_client_id: env_or("STRIPE_CONNECT_CLIENT_ID", ""),
            },
            rate_limit: RateLimitConfig {
                enabled: env_bool("RATE_LIMIT_ENABLED", true),
                redis_url: env_or("RATE_LIMIT_REDIS_URL", "redis://localhost:6379/1"),
            },
            geo: GeoConfig {
                provider: env_or("GEOCODER_PROVIDER", "nominatim"),
                api_url: env_or(
                    "GEOCODER_API_URL",
                    "https://nominatim.openstreetmap.org",
                ),
                api_key: env_or_opt("GEOCODER_API_KEY"),
            },
            features: FeatureFlags {
                realtime_ws: env_bool("FEATURE_REALTIME_WS", true),
                driver_matching: env_bool("FEATURE_DRIVER_MATCHING", true),
                loyalty: env_bool("FEATURE_LOYALTY", true),
            },
        })
    }

    pub fn cors_origins(&self) -> Vec<String> {
        self.cors
            .allowed_origins
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect()
    }

    pub fn access_ttl(&self) -> Duration {
        Duration::from_secs(self.jwt.access_ttl_secs)
    }

    pub fn refresh_ttl(&self) -> Duration {
        Duration::from_secs(self.jwt.refresh_ttl_secs)
    }
}
