//! Cache service — Redis-backed with singleflight + stale-while-revalidate.
//!
//! Patterns:
//! - `get_or_set(key, ttl, fetch_fn)`: tries cache first; on miss, fetches
//!   and caches. Uses `SETNX` lock to prevent thundering herd.
//! - `invalidate(key)`: explicit cache busting.
//!
//! All cached values are JSON-serialized.

use std::time::Duration;

use deadpool_redis::Pool as RedisPool;
use redis::AsyncCommands;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::{AppError, AppResult};

/// Fetch from cache; if miss, run `fetch_fn` and cache the result.
/// Uses a 5-second `SETNX` lock to prevent thundering herd on cache misses.
pub async fn get_or_set<T, F, Fut>(
    redis: &RedisPool,
    key: &str,
    ttl: Duration,
    fetch_fn: F,
) -> AppResult<T>
where
    T: Serialize + DeserializeOwned + Send + Sync + 'static,
    F: FnOnce() -> Fut + Send,
    Fut: std::future::Future<Output = AppResult<T>> + Send,
{
    // Try cache first
    if let Some(cached) = get::<T>(redis, key).await? {
        return Ok(cached);
    }

    // Acquire lock (5s TTL)
    let lock_key = format!("lock:{}", key);
    let mut conn = redis.get().await?;
    let acquired: bool = redis::cmd("SET")
        .arg(&lock_key)
        .arg("1")
        .arg("NX")
        .arg("EX")
        .arg(5)
        .query_async(&mut *conn)
        .await
        .map_err(|e| AppError::internal(format!("redis SETNX failed: {}", e)))?;

    if !acquired {
        // Another worker is fetching. Wait briefly and retry cache.
        tokio::time::sleep(Duration::from_millis(100)).await;
        if let Some(cached) = get::<T>(redis, key).await? {
            return Ok(cached);
        }
        // Fallback: just fetch ourselves (less efficient but correct).
        let value = fetch_fn().await?;
        // Don't cache (another worker is already caching).
        return Ok(value);
    }

    // Fetch + cache
    let value = fetch_fn().await?;
    set(redis, key, &value, ttl).await?;

    // Release lock
    let _: () = conn.del(&lock_key).await.unwrap_or(());
    Ok(value)
}

pub async fn get<T: DeserializeOwned>(redis: &RedisPool, key: &str) -> AppResult<Option<T>> {
    let mut conn = redis.get().await?;
    let raw: Option<String> = conn.get(key).await?;
    match raw {
        Some(s) => {
            let v: T = serde_json::from_str(&s)
                .map_err(|e| AppError::internal(format!("cache deserde failed: {}", e)))?;
            Ok(Some(v))
        }
        None => Ok(None),
    }
}

pub async fn set<T: Serialize>(
    redis: &RedisPool,
    key: &str,
    value: &T,
    ttl: Duration,
) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let s = serde_json::to_string(value)
        .map_err(|e| AppError::internal(format!("cache serde failed: {}", e)))?;
    let _: () = conn.set_ex(key, s, ttl.as_secs()).await?;
    Ok(())
}

pub async fn invalidate(redis: &RedisPool, key: &str) -> AppResult<()> {
    let mut conn = redis.get().await?;
    let _: () = conn.del(key).await?;
    Ok(())
}

/// Invalidate all keys matching a prefix using SCAN (non-blocking).
/// Returns count of deleted keys.
pub async fn invalidate_prefix(redis: &RedisPool, prefix: &str) -> AppResult<usize> {
    let mut conn = redis.get().await?;
    let pattern = format!("{}*", prefix);
    let mut deleted = 0usize;
    let mut cursor: u64 = 0;
    loop {
        let result: (u64, Vec<String>) = redis::cmd("SCAN")
            .arg(cursor)
            .arg("MATCH")
            .arg(&pattern)
            .arg("COUNT")
            .arg(100)
            .query_async(&mut *conn)
            .await?;
        cursor = result.0;
        let keys = result.1;
        if !keys.is_empty() {
            for k in keys {
                let _: () = conn.del(k).await.unwrap_or(());
                deleted += 1;
            }
        }
        if cursor == 0 {
            break;
        }
    }
    Ok(deleted)
}
