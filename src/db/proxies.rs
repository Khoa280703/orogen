use chrono::DateTime;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DbProxy {
    pub id: i32,
    pub url: String,
    pub label: Option<String>,
    pub active: Option<bool>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, FromRow)]
pub struct ActiveProxyRow {
    pub id: i32,
    pub url: String,
}

/// List all proxies
pub async fn list_proxies(pool: &sqlx::PgPool) -> Result<Vec<DbProxy>, sqlx::Error> {
    sqlx::query_as!(
        DbProxy,
        r#"
        SELECT id, url, label, active, created_at
        FROM proxies
        ORDER BY created_at DESC
        "#
    )
    .fetch_all(pool)
    .await
}

/// Get proxy by ID
#[allow(dead_code)]
pub async fn get_proxy(pool: &sqlx::PgPool, id: i32) -> Result<Option<DbProxy>, sqlx::Error> {
    sqlx::query_as!(
        DbProxy,
        r#"
        SELECT id, url, label, active, created_at
        FROM proxies
        WHERE id = $1
        "#,
        id
    )
    .fetch_optional(pool)
    .await
}

/// Create a new proxy
pub async fn create_proxy(
    pool: &sqlx::PgPool,
    url: &str,
    label: Option<&str>,
) -> Result<i32, sqlx::Error> {
    let result = sqlx::query!(
        r#"
        INSERT INTO proxies (url, label)
        VALUES ($1, $2)
        RETURNING id
        "#,
        url,
        label
    )
    .fetch_one(pool)
    .await?;

    Ok(result.id)
}

/// Update proxy
pub async fn update_proxy(
    pool: &sqlx::PgPool,
    id: i32,
    url: Option<&str>,
    label: Option<&str>,
    active: Option<bool>,
) -> Result<bool, sqlx::Error> {
    let id: i32 = id;
    if let Some(u) = url {
        let _: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            UPDATE proxies SET url = $1 WHERE id = $2
            "#,
            u,
            id
        )
        .execute(pool)
        .await?;
    }
    if let Some(l) = label {
        let _: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            UPDATE proxies SET label = $1 WHERE id = $2
            "#,
            l,
            id
        )
        .execute(pool)
        .await?;
    }
    if let Some(a) = active {
        let _: sqlx::postgres::PgQueryResult = sqlx::query!(
            r#"
            UPDATE proxies SET active = $1 WHERE id = $2
            "#,
            a,
            id
        )
        .execute(pool)
        .await?;
    }
    Ok(true)
}

/// Delete proxy by ID
pub async fn delete_proxy(pool: &sqlx::PgPool, id: i32) -> Result<bool, sqlx::Error> {
    let id: i32 = id;
    let result: sqlx::postgres::PgQueryResult =
        sqlx::query!("DELETE FROM proxies WHERE id = $1", id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

/// Check if proxy has assigned accounts
pub async fn has_assigned_accounts(
    pool: &sqlx::PgPool,
    proxy_id: i32,
) -> Result<bool, sqlx::Error> {
    let count: Option<i32> = sqlx::query_scalar!(
        r#"
        SELECT COUNT(*)::INTEGER FROM accounts WHERE proxy_id = $1
        "#,
        proxy_id
    )
    .fetch_one(pool)
    .await?;

    Ok(count.unwrap_or(0) > 0)
}

/// List all active proxy URLs for runtime round-robin assignment.
pub async fn list_active_proxies(pool: &sqlx::PgPool) -> Result<Vec<ActiveProxyRow>, sqlx::Error> {
    let rows = sqlx::query_as::<_, ActiveProxyRow>(
        r#"
        SELECT id, url
        FROM proxies
        WHERE active = true
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows)
}

pub async fn deactivate_proxy(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE proxies
        SET active = false
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn detach_proxy_from_accounts(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE accounts
        SET proxy_id = NULL
        WHERE proxy_id = $1
        "#,
    )
    .bind(id)
    .execute(pool)
    .await?;

    Ok(())
}
