use serde_json::Value;

use crate::account::types::CREDENTIAL_TYPE_GROK_COOKIES;
use crate::account::types::{
    ROUTING_STATE_AUTH_INVALID, ROUTING_STATE_CANDIDATE, ROUTING_STATE_HEALTHY,
};
use crate::db::account_credentials;

pub const SESSION_STATUS_UNKNOWN: &str = "unknown";
pub const SESSION_STATUS_HEALTHY: &str = "healthy";
pub const SESSION_STATUS_EXPIRED: &str = "expired";
pub const SESSION_STATUS_SYNC_ERROR: &str = "sync_error";
pub const SESSION_STATUS_NEEDS_LOGIN: &str = "needs_login";

pub async fn mark_profile_sync_success(
    pool: &sqlx::PgPool,
    id: i32,
    cookies: &Value,
    profile_dir: &str,
) -> Result<(), sqlx::Error> {
    account_credentials::upsert_account_credential(pool, id, CREDENTIAL_TYPE_GROK_COOKIES, cookies)
        .await?;

    sqlx::query(
        r#"
        UPDATE accounts
        SET
            cookies = $1,
            profile_dir = $2,
            session_status = $3,
            session_error = NULL,
            session_checked_at = NOW(),
            cookies_synced_at = NOW(),
            active = true,
            fail_count = 0,
            routing_state = $5,
            cooldown_until = NULL,
            last_routing_error = NULL
        WHERE id = $4
        "#,
    )
    .bind(cookies)
    .bind(profile_dir)
    .bind(SESSION_STATUS_HEALTHY)
    .bind(id)
    .bind(ROUTING_STATE_HEALTHY)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_profile_sync_error(
    pool: &sqlx::PgPool,
    id: i32,
    profile_dir: Option<&str>,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            profile_dir = COALESCE($1, profile_dir),
            session_status = $2,
            session_error = $3,
            session_checked_at = NOW()
        WHERE id = $4
        "#,
    )
    .bind(profile_dir)
    .bind(SESSION_STATUS_SYNC_ERROR)
    .bind(message)
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_session_expired(
    pool: &sqlx::PgPool,
    id: i32,
    message: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            active = false,
            session_status = $1,
            session_error = $2,
            session_checked_at = NOW(),
            routing_state = $4,
            cooldown_until = NULL,
            last_routing_error = $2
        WHERE id = $3
        "#,
    )
    .bind(SESSION_STATUS_EXPIRED)
    .bind(message)
    .bind(id)
    .bind(ROUTING_STATE_AUTH_INVALID)
    .execute(pool)
    .await?;

    Ok(())
}

#[allow(dead_code)]
pub async fn mark_session_healthy(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            session_status = $1,
            session_error = NULL,
            session_checked_at = NOW(),
            routing_state = $3,
            cooldown_until = NULL,
            last_routing_error = NULL
        WHERE id = $2
        "#,
    )
    .bind(SESSION_STATUS_HEALTHY)
    .bind(id)
    .bind(ROUTING_STATE_HEALTHY)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn mark_needs_login(
    pool: &sqlx::PgPool,
    id: i32,
    profile_dir: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET
            profile_dir = $1,
            session_status = CASE
                WHEN cookies_synced_at IS NULL THEN $2
                ELSE session_status
            END,
            session_error = CASE
                WHEN cookies_synced_at IS NULL THEN 'Login required before first sync.'
                ELSE session_error
            END,
            routing_state = CASE
                WHEN cookies_synced_at IS NULL THEN $4
                ELSE routing_state
            END
        WHERE id = $3
        "#,
    )
    .bind(profile_dir)
    .bind(SESSION_STATUS_NEEDS_LOGIN)
    .bind(id)
    .bind(ROUTING_STATE_CANDIDATE)
    .execute(pool)
    .await?;

    Ok(())
}
