use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub provider_id: Option<String>,
    pub locale: String,
    pub active: bool,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateUserInput {
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub provider_id: Option<String>,
    pub locale: Option<String>,
}

/// Find user by email or create new user
pub async fn find_or_create_user(
    pool: &sqlx::PgPool,
    input: CreateUserInput,
) -> Result<User, sqlx::Error> {
    // Try to find existing user
    let row = sqlx::query(
        r#"
        SELECT id, email, name, avatar_url, provider, provider_id, locale, active, created_at
        FROM users
        WHERE email = $1 AND active = true
        LIMIT 1
        "#,
    )
    .bind(&input.email)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok(User {
            id: row.get("id"),
            email: row.get("email"),
            name: row.get("name"),
            avatar_url: row.get("avatar_url"),
            provider: row.get("provider"),
            provider_id: row.get("provider_id"),
            locale: row.get("locale"),
            active: row.get("active"),
            created_at: row.get("created_at"),
        });
    }

    // Create new user
    let row = sqlx::query(
        r#"
        INSERT INTO users (email, name, avatar_url, provider, provider_id, locale)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, email, name, avatar_url, provider, provider_id, locale, active, created_at
        "#,
    )
    .bind(&input.email)
    .bind(&input.name)
    .bind(&input.avatar_url)
    .bind(&input.provider)
    .bind(&input.provider_id)
    .bind(input.locale.as_deref().unwrap_or("en"))
    .fetch_one(pool)
    .await?;

    Ok(User {
        id: row.get("id"),
        email: row.get("email"),
        name: row.get("name"),
        avatar_url: row.get("avatar_url"),
        provider: row.get("provider"),
        provider_id: row.get("provider_id"),
        locale: row.get("locale"),
        active: row.get("active"),
        created_at: row.get("created_at"),
    })
}

/// Get user by ID
pub async fn get_user(pool: &sqlx::PgPool, id: i32) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, email, name, avatar_url, provider, provider_id, locale, active, created_at
        FROM users
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: r.get("id"),
        email: r.get("email"),
        name: r.get("name"),
        avatar_url: r.get("avatar_url"),
        provider: r.get("provider"),
        provider_id: r.get("provider_id"),
        locale: r.get("locale"),
        active: r.get("active"),
        created_at: r.get("created_at"),
    }))
}

/// Get user by email
#[allow(dead_code)]
pub async fn get_user_by_email(
    pool: &sqlx::PgPool,
    email: &str,
) -> Result<Option<User>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, email, name, avatar_url, provider, provider_id, locale, active, created_at
        FROM users
        WHERE email = $1 AND active = true
        "#,
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| User {
        id: r.get("id"),
        email: r.get("email"),
        name: r.get("name"),
        avatar_url: r.get("avatar_url"),
        provider: r.get("provider"),
        provider_id: r.get("provider_id"),
        locale: r.get("locale"),
        active: r.get("active"),
        created_at: r.get("created_at"),
    }))
}
