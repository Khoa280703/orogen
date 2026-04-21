use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;

#[derive(Debug, Clone, Serialize)]
pub struct Message {
    pub id: i32,
    pub conversation_id: i32,
    pub role: String,
    pub content: String,
    pub model_slug: Option<String>,
    pub provider_slug: Option<String>,
    pub tokens_used: i32,
    pub created_at: Option<DateTime<Utc>>,
}

pub async fn create_message(
    pool: &sqlx::PgPool,
    conversation_id: i32,
    role: &str,
    content: &str,
    model_slug: Option<&str>,
    provider_slug: Option<&str>,
    tokens_used: i32,
) -> Result<Message, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO messages (conversation_id, role, content, model_slug, provider_slug, tokens_used)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, conversation_id, role, content, model_slug, provider_slug, tokens_used, created_at
        "#,
    )
    .bind(conversation_id)
    .bind(role)
    .bind(content)
    .bind(model_slug)
    .bind(provider_slug)
    .bind(tokens_used)
    .fetch_one(pool)
    .await?;

    Ok(map_message(&row))
}

pub async fn list_messages(
    pool: &sqlx::PgPool,
    conversation_id: i32,
) -> Result<Vec<Message>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, conversation_id, role, content, model_slug, provider_slug, tokens_used, created_at
        FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(conversation_id)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(map_message).collect())
}

pub async fn count_messages(pool: &sqlx::PgPool, conversation_id: i32) -> Result<i64, sqlx::Error> {
    let count: Option<i64> =
        sqlx::query_scalar(r#"SELECT COUNT(*)::BIGINT FROM messages WHERE conversation_id = $1"#)
            .bind(conversation_id)
            .fetch_one(pool)
            .await?;

    Ok(count.unwrap_or(0))
}

fn map_message(row: &sqlx::postgres::PgRow) -> Message {
    Message {
        id: row.get("id"),
        conversation_id: row.get("conversation_id"),
        role: row.get("role"),
        content: row.get("content"),
        model_slug: row.get("model_slug"),
        provider_slug: row.get("provider_slug"),
        tokens_used: row.get("tokens_used"),
        created_at: row.get("created_at"),
    }
}
