use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::Row;

#[derive(Debug, Clone, Serialize)]
pub struct Conversation {
    pub id: i32,
    pub user_id: i32,
    pub title: Option<String>,
    pub model_slug: Option<String>,
    pub active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ConversationListItem {
    pub id: i32,
    pub user_id: i32,
    pub title: Option<String>,
    pub model_slug: Option<String>,
    pub active: bool,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub message_count: i64,
}

pub async fn create_conversation(
    pool: &sqlx::PgPool,
    user_id: i32,
    title: Option<&str>,
    model_slug: Option<&str>,
) -> Result<Conversation, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO conversations (user_id, title, model_slug)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, title, model_slug, active, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(title)
    .bind(model_slug)
    .fetch_one(pool)
    .await?;

    Ok(map_conversation(&row))
}

pub async fn list_conversations(
    pool: &sqlx::PgPool,
    user_id: i32,
    limit: i64,
    offset: i64,
) -> Result<Vec<ConversationListItem>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT c.id, c.user_id, c.title, c.model_slug, c.active, c.created_at, c.updated_at,
               COUNT(m.id)::BIGINT AS message_count
        FROM conversations c
        LEFT JOIN messages m ON m.conversation_id = c.id
        WHERE c.user_id = $1 AND c.active = true
        GROUP BY c.id
        ORDER BY c.updated_at DESC NULLS LAST, c.created_at DESC NULLS LAST
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ConversationListItem {
            id: row.get("id"),
            user_id: row.get("user_id"),
            title: row.get("title"),
            model_slug: row.get("model_slug"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
            message_count: row.get("message_count"),
        })
        .collect())
}

pub async fn get_conversation(
    pool: &sqlx::PgPool,
    id: i32,
    user_id: i32,
) -> Result<Option<Conversation>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, title, model_slug, active, created_at, updated_at
        FROM conversations
        WHERE id = $1 AND user_id = $2 AND active = true
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.as_ref().map(map_conversation))
}

pub async fn update_title(
    pool: &sqlx::PgPool,
    id: i32,
    user_id: i32,
    title: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE conversations SET title = $1, updated_at = NOW() WHERE id = $2 AND user_id = $3"#,
    )
        .bind(title)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn update_model_slug(
    pool: &sqlx::PgPool,
    id: i32,
    user_id: i32,
    model_slug: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"UPDATE conversations SET model_slug = $1, updated_at = NOW() WHERE id = $2 AND user_id = $3"#,
    )
    .bind(model_slug)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn touch_conversation(pool: &sqlx::PgPool, id: i32) -> Result<(), sqlx::Error> {
    sqlx::query(r#"UPDATE conversations SET updated_at = NOW() WHERE id = $1"#)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete_conversation(
    pool: &sqlx::PgPool,
    id: i32,
    user_id: i32,
) -> Result<bool, sqlx::Error> {
    let result =
        sqlx::query(r#"UPDATE conversations SET active = false, updated_at = NOW() WHERE id = $1 AND user_id = $2"#)
            .bind(id)
            .bind(user_id)
            .execute(pool)
            .await?;
    Ok(result.rows_affected() > 0)
}

fn map_conversation(row: &sqlx::postgres::PgRow) -> Conversation {
    Conversation {
        id: row.get("id"),
        user_id: row.get("user_id"),
        title: row.get("title"),
        model_slug: row.get("model_slug"),
        active: row.get("active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    }
}
