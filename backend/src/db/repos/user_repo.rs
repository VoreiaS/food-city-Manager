//! User repository.
//!
//! Uses runtime `sqlx::query_as` with `FromRow` derive (no compile-time
//! DB connection needed). The `query!` macros would require either a live
//! DATABASE_URL at compile time or a prepared `sqlx-data.json` cache; we
//! trade off compile-time query validation for simpler dev workflow.
//! Switch to `query_as!` macros in Phase 9 (production hardening).

use sqlx::{PgPool, Result as SqlxResult};
use uuid::Uuid;

use crate::domain::user::{User, UserRole};

pub async fn find_by_id(db: &PgPool, id: Uuid) -> SqlxResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT
            id, email, phone, full_name,
            role as "role",
            password_hash, is_active, created_at, updated_at, deleted_at
        FROM users
        WHERE id = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(id)
    .fetch_optional(db)
    .await?;
    Ok(user)
}

pub async fn find_by_email(db: &PgPool, email: &str) -> SqlxResult<Option<User>> {
    let user = sqlx::query_as::<_, User>(
        r#"
        SELECT
            id, email, phone, full_name,
            role as "role",
            password_hash, is_active, created_at, updated_at, deleted_at
        FROM users
        WHERE email = $1 AND deleted_at IS NULL
        "#,
    )
    .bind(email)
    .fetch_optional(db)
    .await?;
    Ok(user)
}

pub async fn insert(
    db: &PgPool,
    id: Uuid,
    email: &str,
    phone: &str,
    full_name: &str,
    role: UserRole,
    password_hash: &str,
) -> SqlxResult<User> {
    let user = sqlx::query_as::<_, User>(
        r#"
        INSERT INTO users (id, email, phone, full_name, role, password_hash, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        RETURNING
            id, email, phone, full_name,
            role as "role",
            password_hash, is_active, created_at, updated_at, deleted_at
        "#,
    )
    .bind(id)
    .bind(email)
    .bind(phone)
    .bind(full_name)
    .bind(role)
    .bind(password_hash)
    .fetch_one(db)
    .await?;
    Ok(user)
}

pub async fn soft_delete(db: &PgPool, id: Uuid) -> SqlxResult<()> {
    sqlx::query("UPDATE users SET deleted_at = NOW(), updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(db)
        .await?;
    Ok(())
}
