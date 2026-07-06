//! DB pool setup. Primary + optional read replica.

use sqlx::postgres::PgPoolOptions;
use std::time::Duration;

pub struct DbPools {
    pub primary: sqlx::PgPool,
    pub replica: Option<sqlx::PgPool>,
}

impl DbPools {
    /// Returns the replica if available, else the primary.
    /// Use for read-heavy endpoints.
    pub fn read(&self) -> &sqlx::PgPool {
        self.replica.as_ref().unwrap_or(&self.primary)
    }

    pub fn write(&self) -> &sqlx::PgPool {
        &self.primary
    }
}

pub async fn create_pools(
    url: &str,
    replica_url: Option<&str>,
    max_connections: u32,
    acquire_timeout: Duration,
) -> anyhow::Result<DbPools> {
    let primary = PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout)
        .connect(url)
        .await?;

    let replica = if let Some(rurl) = replica_url {
        Some(
            PgPoolOptions::new()
                .max_connections(max_connections)
                .acquire_timeout(acquire_timeout)
                .connect(rurl)
                .await?,
        )
    } else {
        None
    };

    Ok(DbPools { primary, replica })
}

/// Convenience: build a single pool (used by main.rs).
pub async fn create_db_pool(
    url: &str,
    max_connections: u32,
    acquire_timeout: Duration,
) -> anyhow::Result<sqlx::PgPool> {
    Ok(PgPoolOptions::new()
        .max_connections(max_connections)
        .acquire_timeout(acquire_timeout)
        .connect(url)
        .await?)
}
