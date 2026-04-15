use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DbApiKey {
    pub id: i32,
    pub key: String,
    pub label: Option<String>,
    pub active: Option<bool>,
    pub quota_per_day: Option<i32>,
    pub plan_id: Option<i32>,
    pub plan_name: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i32,
    pub key: String,
    pub label: String,
    pub user_id: Option<i32>,
    pub plan_id: Option<i32>,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

fn generate_api_key() -> String {
    let random_bytes: [u8; 16] = rand::random();
    format!(
        "sk-{}",
        base64::Engine::encode(
            &base64::engine::general_purpose::URL_SAFE_NO_PAD,
            &random_bytes
        )
    )
}

fn map_api_key_row(row: sqlx::postgres::PgRow) -> ApiKey {
    ApiKey {
        id: row.get("id"),
        key: row.get("key"),
        label: row.get::<Option<String>, _>("label").unwrap_or_default(),
        user_id: row.get("user_id"),
        plan_id: row.get("plan_id"),
        active: row.get::<Option<bool>, _>("active").unwrap_or(true),
        created_at: row
            .get::<Option<DateTime<Utc>>, _>("created_at")
            .unwrap_or(Utc::now()),
        last_used_at: row.get("last_used_at"),
    }
}

pub async fn list_by_user(pool: &sqlx::PgPool, user_id: i32) -> Result<Vec<ApiKey>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, key, label, user_id, plan_id, active, created_at, last_used_at
        FROM api_keys
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.into_iter().map(map_api_key_row).collect())
}

pub async fn create_key(
    pool: &sqlx::PgPool,
    label: &str,
    user_id: Option<i32>,
) -> Result<ApiKey, sqlx::Error> {
    let key = generate_api_key();

    let row = sqlx::query(
        r#"
        INSERT INTO api_keys (key, label, user_id, plan_id, active)
        VALUES ($1, $2, $3, NULL, true)
        RETURNING id, key, label, user_id, plan_id, active, created_at, last_used_at
        "#,
    )
    .bind(&key)
    .bind(label)
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(map_api_key_row(row))
}

pub async fn get_key(pool: &sqlx::PgPool, id: i32) -> Result<Option<ApiKey>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, key, label, user_id, plan_id, active, created_at, last_used_at
        FROM api_keys
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_api_key_row))
}

pub async fn get_key_by_value(
    pool: &sqlx::PgPool,
    key: &str,
) -> Result<Option<ApiKey>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, key, label, user_id, plan_id, active, created_at, last_used_at
        FROM api_keys
        WHERE key = $1 AND active = true
        "#,
    )
    .bind(key)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(map_api_key_row))
}

pub async fn touch_last_used(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE api_keys
        SET last_used_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn revoke_key(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query(
        r#"
        UPDATE api_keys SET active = false WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn list_api_keys(pool: &sqlx::PgPool) -> Result<Vec<DbApiKey>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT ak.id, ak.key, ak.label, ak.active, ak.quota_per_day, ak.plan_id, p.name AS plan_name, ak.created_at
        FROM api_keys ak
        LEFT JOIN plans p ON ak.plan_id = p.id
        ORDER BY ak.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| DbApiKey {
            id: row.get("id"),
            key: row.get("key"),
            label: row.get("label"),
            active: row.get("active"),
            quota_per_day: row.get("quota_per_day"),
            plan_id: row.get("plan_id"),
            plan_name: row.get("plan_name"),
            created_at: row.get("created_at"),
        })
        .collect())
}

pub async fn get_api_key(pool: &sqlx::PgPool, id: i32) -> Result<Option<DbApiKey>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT ak.id, ak.key, ak.label, ak.active, ak.quota_per_day, ak.plan_id, p.name AS plan_name, ak.created_at
        FROM api_keys ak
        LEFT JOIN plans p ON ak.plan_id = p.id
        WHERE ak.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|row| DbApiKey {
        id: row.get("id"),
        key: row.get("key"),
        label: row.get("label"),
        active: row.get("active"),
        quota_per_day: row.get("quota_per_day"),
        plan_id: row.get("plan_id"),
        plan_name: row.get("plan_name"),
        created_at: row.get("created_at"),
    }))
}

#[allow(dead_code)]
pub async fn validate_key(pool: &sqlx::PgPool, key: &str) -> Result<bool, sqlx::Error> {
    let exists: Option<bool> = sqlx::query_scalar(
        r#"
        SELECT EXISTS(SELECT 1 FROM api_keys WHERE key = $1 AND active = true)
        "#,
    )
    .bind(key)
    .fetch_one(pool)
    .await?;

    Ok(exists.unwrap_or(false))
}

pub async fn create_api_key(
    pool: &sqlx::PgPool,
    key: &str,
    label: Option<&str>,
    quota_per_day: Option<i32>,
    plan_id: Option<i32>,
) -> Result<i32, sqlx::Error> {
    let result = sqlx::query(
        r#"
        INSERT INTO api_keys (key, label, quota_per_day, plan_id)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(key)
    .bind(label)
    .bind(quota_per_day)
    .bind(plan_id)
    .fetch_one(pool)
    .await?;

    Ok(result.get("id"))
}

pub async fn update_api_key(
    pool: &sqlx::PgPool,
    id: i32,
    label: Option<&str>,
    active: Option<bool>,
    quota_per_day: Option<i32>,
    plan_id: Option<i32>,
) -> Result<bool, sqlx::Error> {
    if let Some(value) = label {
        sqlx::query(
            r#"
            UPDATE api_keys SET label = $1 WHERE id = $2
            "#,
        )
        .bind(value)
        .bind(id)
        .execute(pool)
        .await?;
    }
    if let Some(value) = active {
        sqlx::query(
            r#"
            UPDATE api_keys SET active = $1 WHERE id = $2
            "#,
        )
        .bind(value)
        .bind(id)
        .execute(pool)
        .await?;
    }
    if let Some(value) = quota_per_day {
        sqlx::query(
            r#"
            UPDATE api_keys SET quota_per_day = $1 WHERE id = $2
            "#,
        )
        .bind(value)
        .bind(id)
        .execute(pool)
        .await?;
    }
    if let Some(value) = plan_id {
        sqlx::query(
            r#"
            UPDATE api_keys SET plan_id = $1 WHERE id = $2
            "#,
        )
        .bind(value)
        .bind(id)
        .execute(pool)
        .await?;
    }
    Ok(true)
}

pub async fn delete_api_key(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM api_keys WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}
