use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::FromRow;

use crate::db::account_sessions::SESSION_STATUS_HEALTHY;

#[derive(Debug, Clone, FromRow)]
pub struct DbAccount {
    pub id: i32,
    pub name: String,
    pub cookies: Value,
    pub active: Option<bool>,
    pub proxy_id: Option<i32>,
    pub profile_dir: Option<String>,
    pub session_status: Option<String>,
    pub session_error: Option<String>,
    pub request_count: Option<i64>,
    pub fail_count: Option<i32>,
    pub success_count: Option<i64>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub session_checked_at: Option<DateTime<Utc>>,
    pub cookies_synced_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct RuntimeAccountRow {
    pub id: i32,
    pub name: String,
    pub cookies: Value,
    pub proxy_id: Option<i32>,
    pub proxy_url: Option<String>,
}

pub async fn list_accounts(pool: &sqlx::PgPool) -> Result<Vec<DbAccount>, sqlx::Error> {
    sqlx::query_as::<_, DbAccount>(
        r#"
        SELECT
            id,
            name,
            cookies,
            active,
            proxy_id,
            profile_dir,
            session_status,
            session_error,
            request_count,
            fail_count,
            success_count,
            last_used,
            created_at,
            session_checked_at,
            cookies_synced_at
        FROM accounts
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_runtime_accounts(
    pool: &sqlx::PgPool,
) -> Result<Vec<RuntimeAccountRow>, sqlx::Error> {
    sqlx::query_as::<_, RuntimeAccountRow>(
        r#"
        SELECT
            a.id,
            a.name,
            a.cookies,
            a.proxy_id,
            p.url AS proxy_url
        FROM accounts a
        LEFT JOIN proxies p
            ON p.id = a.proxy_id
           AND p.active = true
        WHERE a.active = true
        ORDER BY a.created_at ASC, a.id ASC
        "#,
    )
    .fetch_all(pool)
    .await
}

#[allow(dead_code)]
pub async fn get_account(pool: &sqlx::PgPool, id: i32) -> Result<Option<DbAccount>, sqlx::Error> {
    sqlx::query_as::<_, DbAccount>(
        r#"
        SELECT
            id,
            name,
            cookies,
            active,
            proxy_id,
            profile_dir,
            session_status,
            session_error,
            request_count,
            fail_count,
            success_count,
            last_used,
            created_at,
            session_checked_at,
            cookies_synced_at
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_account(
    pool: &sqlx::PgPool,
    name: &str,
    cookies: &Value,
    proxy_id: Option<i32>,
    profile_dir: Option<&str>,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        r#"
        INSERT INTO accounts (name, cookies, proxy_id, profile_dir)
        VALUES ($1, $2, $3, $4)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(cookies)
    .bind(proxy_id)
    .bind(profile_dir)
    .fetch_one(pool)
    .await
}

pub async fn update_account(
    pool: &sqlx::PgPool,
    id: i32,
    cookies: Option<&Value>,
    active: Option<bool>,
    proxy_id: Option<Option<i32>>,
    profile_dir: Option<Option<String>>,
) -> Result<bool, sqlx::Error> {
    if let Some(c) = cookies {
        sqlx::query("UPDATE accounts SET cookies = $1 WHERE id = $2")
            .bind(c)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(a) = active {
        if a {
            sqlx::query(
                r#"
                UPDATE accounts
                SET active = true, fail_count = 0
                WHERE id = $1
                "#,
            )
            .bind(id)
            .execute(pool)
            .await?;
        } else {
            sqlx::query("UPDATE accounts SET active = false WHERE id = $1")
                .bind(id)
                .execute(pool)
                .await?;
        }
    }

    if let Some(p) = proxy_id {
        sqlx::query("UPDATE accounts SET proxy_id = $1 WHERE id = $2")
            .bind(p)
            .bind(id)
            .execute(pool)
            .await?;
    }

    if let Some(dir) = profile_dir {
        sqlx::query("UPDATE accounts SET profile_dir = $1 WHERE id = $2")
            .bind(dir)
            .bind(id)
            .execute(pool)
            .await?;
    }

    Ok(true)
}

pub async fn delete_account(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM accounts WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn increment_request_count(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET request_count = COALESCE(request_count, 0) + 1, last_used = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_account_ids_by_proxy(
    pool: &sqlx::PgPool,
    proxy_id: i32,
) -> Result<Vec<i32>, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT id
        FROM accounts
        WHERE proxy_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(proxy_id)
    .fetch_all(pool)
    .await
}

pub async fn assign_proxy_to_account(
    pool: &sqlx::PgPool,
    account_id: i32,
    proxy_id: Option<i32>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET proxy_id = $1
        WHERE id = $2
        "#,
    )
    .bind(proxy_id)
    .bind(account_id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn record_rate_limited_attempt(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET request_count = COALESCE(request_count, 0) + 1, last_used = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_health_counts(
    pool: &sqlx::PgPool,
    id: i32,
    success: bool,
) -> Result<bool, sqlx::Error> {
    if success {
        sqlx::query(
            r#"
            UPDATE accounts
            SET
                request_count = COALESCE(request_count, 0) + 1,
                success_count = COALESCE(success_count, 0) + 1,
                fail_count = 0,
                last_used = NOW(),
                session_status = $2,
                session_error = NULL,
                session_checked_at = NOW()
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(SESSION_STATUS_HEALTHY)
        .execute(pool)
        .await?;
        return Ok(false);
    }

    let fail_count = sqlx::query_scalar::<_, i32>(
        r#"
        UPDATE accounts
        SET
            request_count = COALESCE(request_count, 0) + 1,
            fail_count = COALESCE(fail_count, 0) + 1,
            last_used = NOW()
        WHERE id = $1
        RETURNING fail_count
        "#,
    )
    .bind(id)
    .fetch_one(pool)
    .await?;

    if fail_count >= 3 {
        sqlx::query("UPDATE accounts SET active = false WHERE id = $1")
            .bind(id)
            .execute(pool)
            .await?;
        return Ok(true);
    }

    Ok(false)
}
