use chrono::{DateTime, Utc};
use serde_json::{Value, json};
use sqlx::FromRow;

use crate::account::types::{
    AUTH_MODE_GROK_COOKIES, ROUTING_STATE_AUTH_INVALID, ROUTING_STATE_CANDIDATE,
    ROUTING_STATE_COOLING_DOWN, ROUTING_STATE_HEALTHY, ROUTING_STATE_PAUSED,
    ROUTING_STATE_REFRESH_FAILED,
};
use crate::db::account_sessions::SESSION_STATUS_HEALTHY;

const DEFAULT_FAILURE_COOLDOWN_SECONDS: i64 = 30;
const ESCALATED_FAILURE_COOLDOWN_SECONDS: i64 = 300;
const RATE_LIMIT_COOLDOWN_SECONDS: i64 = 180;
const PROXY_FAILURE_COOLDOWN_SECONDS: i64 = 90;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct DbAccount {
    pub id: i32,
    pub name: String,
    pub provider_slug: String,
    pub account_label: Option<String>,
    pub external_account_id: Option<String>,
    pub auth_mode: Option<String>,
    pub metadata: Value,
    pub cookies: Value,
    pub credential_type: Option<String>,
    pub credential_payload: Option<Value>,
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
    pub routing_state: String,
    pub cooldown_until: Option<DateTime<Utc>>,
    pub last_routing_error: Option<String>,
    pub rate_limit_streak: i32,
    pub auth_failure_streak: i32,
    pub refresh_failure_streak: i32,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct RuntimeAccountRow {
    pub id: i32,
    pub name: String,
    pub provider_slug: String,
    pub account_label: Option<String>,
    pub external_account_id: Option<String>,
    pub auth_mode: Option<String>,
    pub metadata: Value,
    pub cookies: Value,
    pub credential_type: Option<String>,
    pub credential_payload: Option<Value>,
    pub proxy_id: Option<i32>,
    pub proxy_url: Option<String>,
    pub session_status: Option<String>,
    pub last_used: Option<DateTime<Utc>>,
    pub created_at: Option<DateTime<Utc>>,
    pub routing_state: String,
    pub cooldown_until: Option<DateTime<Utc>>,
    pub last_routing_error: Option<String>,
    pub rate_limit_streak: i32,
    pub auth_failure_streak: i32,
    pub refresh_failure_streak: i32,
}

pub async fn list_accounts(pool: &sqlx::PgPool) -> Result<Vec<DbAccount>, sqlx::Error> {
    sqlx::query_as::<_, DbAccount>(
        r#"
        SELECT
            a.id,
            a.name,
            a.provider_slug,
            a.account_label,
            a.external_account_id,
            a.auth_mode,
            COALESCE(a.metadata, '{}'::jsonb) AS metadata,
            a.cookies,
            ac.credential_type,
            ac.payload AS credential_payload,
            a.active,
            a.proxy_id,
            a.profile_dir,
            a.session_status,
            a.session_error,
            a.request_count,
            a.fail_count,
            a.success_count,
            a.last_used,
            a.created_at,
            a.session_checked_at,
            a.cookies_synced_at,
            COALESCE(a.routing_state, 'candidate') AS routing_state,
            a.cooldown_until,
            a.last_routing_error,
            COALESCE(a.rate_limit_streak, 0) AS rate_limit_streak,
            COALESCE(a.auth_failure_streak, 0) AS auth_failure_streak,
            COALESCE(a.refresh_failure_streak, 0) AS refresh_failure_streak
        FROM accounts a
        LEFT JOIN account_credentials ac ON ac.account_id = a.id
        ORDER BY a.provider_slug ASC, a.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await
}

pub async fn list_runtime_accounts_by_provider(
    pool: &sqlx::PgPool,
    provider_slug: &str,
) -> Result<Vec<RuntimeAccountRow>, sqlx::Error> {
    sqlx::query_as::<_, RuntimeAccountRow>(
        r#"
        SELECT
            a.id,
            a.name,
            a.provider_slug,
            a.account_label,
            a.external_account_id,
            a.auth_mode,
            COALESCE(a.metadata, '{}'::jsonb) AS metadata,
            a.cookies,
            ac.credential_type,
            ac.payload AS credential_payload,
            a.proxy_id,
            p.url AS proxy_url,
            a.session_status,
            a.last_used,
            a.created_at,
            COALESCE(a.routing_state, 'candidate') AS routing_state,
            a.cooldown_until,
            a.last_routing_error,
            COALESCE(a.rate_limit_streak, 0) AS rate_limit_streak,
            COALESCE(a.auth_failure_streak, 0) AS auth_failure_streak,
            COALESCE(a.refresh_failure_streak, 0) AS refresh_failure_streak
        FROM accounts a
        LEFT JOIN proxies p
            ON p.id = a.proxy_id
           AND p.active = true
        LEFT JOIN account_credentials ac ON ac.account_id = a.id
        WHERE a.active = true
          AND a.provider_slug = $1
        ORDER BY a.last_used ASC NULLS FIRST, a.created_at ASC, a.id ASC
        "#,
    )
    .bind(provider_slug)
    .fetch_all(pool)
    .await
}

pub async fn get_account(pool: &sqlx::PgPool, id: i32) -> Result<Option<DbAccount>, sqlx::Error> {
    sqlx::query_as::<_, DbAccount>(
        r#"
        SELECT
            a.id,
            a.name,
            a.provider_slug,
            a.account_label,
            a.external_account_id,
            a.auth_mode,
            COALESCE(a.metadata, '{}'::jsonb) AS metadata,
            a.cookies,
            ac.credential_type,
            ac.payload AS credential_payload,
            a.active,
            a.proxy_id,
            a.profile_dir,
            a.session_status,
            a.session_error,
            a.request_count,
            a.fail_count,
            a.success_count,
            a.last_used,
            a.created_at,
            a.session_checked_at,
            a.cookies_synced_at,
            COALESCE(a.routing_state, 'candidate') AS routing_state,
            a.cooldown_until,
            a.last_routing_error,
            COALESCE(a.rate_limit_streak, 0) AS rate_limit_streak,
            COALESCE(a.auth_failure_streak, 0) AS auth_failure_streak,
            COALESCE(a.refresh_failure_streak, 0) AS refresh_failure_streak
        FROM accounts a
        LEFT JOIN account_credentials ac ON ac.account_id = a.id
        WHERE a.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_account(
    pool: &sqlx::PgPool,
    name: &str,
    provider_slug: &str,
    legacy_cookies: &Value,
    proxy_id: Option<i32>,
    profile_dir: Option<&str>,
    auth_mode: Option<&str>,
) -> Result<i32, sqlx::Error> {
    sqlx::query_scalar::<_, i32>(
        r#"
        INSERT INTO accounts (name, provider_slug, cookies, proxy_id, profile_dir, auth_mode, metadata)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(name)
    .bind(provider_slug)
    .bind(legacy_cookies)
    .bind(proxy_id)
    .bind(profile_dir)
    .bind(auth_mode.unwrap_or(AUTH_MODE_GROK_COOKIES))
    .bind(json!({}))
    .fetch_one(pool)
    .await
}

pub async fn update_account(
    pool: &sqlx::PgPool,
    id: i32,
    legacy_cookies: Option<&Value>,
    active: Option<bool>,
    proxy_id: Option<Option<i32>>,
    profile_dir: Option<Option<String>>,
) -> Result<bool, sqlx::Error> {
    if let Some(c) = legacy_cookies {
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
            sqlx::query(
                r#"
                UPDATE accounts
                SET
                    routing_state = $2,
                    cooldown_until = NULL,
                    last_routing_error = NULL
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(ROUTING_STATE_CANDIDATE)
            .execute(pool)
            .await?;
        } else {
            sqlx::query(
                r#"
                UPDATE accounts
                SET
                    active = false,
                    routing_state = $2,
                    cooldown_until = NULL
                WHERE id = $1
                "#,
            )
            .bind(id)
            .bind(ROUTING_STATE_PAUSED)
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

pub async fn update_account_identity(
    pool: &sqlx::PgPool,
    id: i32,
    account_label: Option<&str>,
    external_account_id: Option<&str>,
    auth_mode: Option<&str>,
    metadata: Option<&Value>,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            account_label = COALESCE($1, account_label),
            external_account_id = COALESCE($2, external_account_id),
            auth_mode = COALESCE($3, auth_mode),
            metadata = COALESCE($4, metadata)
        WHERE id = $5
        "#,
    )
    .bind(account_label)
    .bind(external_account_id)
    .bind(auth_mode)
    .bind(metadata)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
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
    mark_account_cooling_down(
        pool,
        id,
        "Upstream rate limited this account.",
        RATE_LIMIT_COOLDOWN_SECONDS,
        true,
        true,
    )
    .await
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
                session_checked_at = NOW(),
                routing_state = $3,
                cooldown_until = NULL,
                last_routing_error = NULL,
                rate_limit_streak = 0,
                auth_failure_streak = 0,
                refresh_failure_streak = 0
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(SESSION_STATUS_HEALTHY)
        .bind(ROUTING_STATE_HEALTHY)
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

    let cooldown_seconds = if fail_count >= 3 {
        ESCALATED_FAILURE_COOLDOWN_SECONDS
    } else {
        DEFAULT_FAILURE_COOLDOWN_SECONDS
    };

    mark_account_cooling_down(
        pool,
        id,
        "Generic upstream failure pushed this account into cooldown.",
        cooldown_seconds,
        false,
        false,
    )
    .await?;

    Ok(fail_count >= 3)
}

pub async fn mark_account_auth_invalid(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            routing_state = $2,
            last_routing_error = $3,
            cooldown_until = NULL,
            request_count = COALESCE(request_count, 0) + 1,
            auth_failure_streak = COALESCE(auth_failure_streak, 0) + 1,
            fail_count = COALESCE(fail_count, 0) + 1,
            last_used = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(ROUTING_STATE_AUTH_INVALID)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_account_refresh_failed(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            routing_state = $2,
            last_routing_error = $3,
            cooldown_until = NULL,
            refresh_failure_streak = COALESCE(refresh_failure_streak, 0) + 1,
            fail_count = COALESCE(fail_count, 0) + 1,
            last_used = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(ROUTING_STATE_REFRESH_FAILED)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_account_proxy_failed(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    mark_account_cooling_down(
        pool,
        id,
        message,
        PROXY_FAILURE_COOLDOWN_SECONDS,
        false,
        true,
    )
    .await
}

pub async fn mark_account_transient_failure(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    mark_account_cooling_down(
        pool,
        id,
        message,
        DEFAULT_FAILURE_COOLDOWN_SECONDS,
        false,
        true,
    )
    .await
}

async fn mark_account_cooling_down(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
    cooldown_seconds: i64,
    increment_rate_limit_streak: bool,
    increment_counts: bool,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            routing_state = $2,
            cooldown_until = NOW() + make_interval(secs => $3::int),
            last_routing_error = $4,
            request_count = CASE
                WHEN $6 THEN COALESCE(request_count, 0) + 1
                ELSE COALESCE(request_count, 0)
            END,
            fail_count = CASE
                WHEN $6 THEN COALESCE(fail_count, 0) + 1
                ELSE COALESCE(fail_count, 0)
            END,
            rate_limit_streak = CASE
                WHEN $5 THEN COALESCE(rate_limit_streak, 0) + 1
                ELSE 0
            END,
            last_used = NOW()
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(ROUTING_STATE_COOLING_DOWN)
    .bind(cooldown_seconds as i32)
    .bind(message)
    .bind(increment_rate_limit_streak)
    .bind(increment_counts)
    .execute(pool)
    .await?;

    Ok(())
}
