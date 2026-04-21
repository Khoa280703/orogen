use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::Row;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ConversationListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminConversationListItem {
    pub id: i32,
    pub user_id: i32,
    pub user_email: String,
    pub user_name: Option<String>,
    pub title: Option<String>,
    pub model_slug: Option<String>,
    pub active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub message_count: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminConversationMessage {
    pub id: i32,
    pub role: String,
    pub content: String,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminConversationDetail {
    pub id: i32,
    pub user_id: i32,
    pub user_email: String,
    pub user_name: Option<String>,
    pub title: Option<String>,
    pub model_slug: Option<String>,
    pub active: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
    pub messages: Vec<AdminConversationMessage>,
}

#[derive(Debug, Serialize)]
pub struct AdminConversationListResponse {
    pub items: Vec<AdminConversationListItem>,
    pub total: i64,
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Query(query): Query<ConversationListQuery>,
) -> Result<Json<AdminConversationListResponse>, (StatusCode, String)> {
    let search = query.search.as_deref().unwrap_or("").trim();
    let model = query.model.as_deref().unwrap_or("").trim();
    let search_filter = if search.is_empty() {
        None
    } else {
        Some(format!("%{search}%"))
    };
    let model_filter = if model.is_empty() { None } else { Some(model) };

    let items = sqlx::query(
        r#"
        SELECT c.id, c.user_id, u.email AS user_email, u.name AS user_name, c.title, c.model_slug,
               c.active, c.created_at, c.updated_at, COUNT(m.id)::BIGINT AS message_count
        FROM conversations c
        JOIN users u ON u.id = c.user_id
        LEFT JOIN messages m ON m.conversation_id = c.id
        WHERE c.active = true
          AND ($1::TEXT IS NULL OR u.email ILIKE $1 OR COALESCE(u.name, '') ILIKE $1 OR COALESCE(c.title, '') ILIKE $1)
          AND ($2::TEXT IS NULL OR c.model_slug = $2)
        GROUP BY c.id, u.id
        ORDER BY c.updated_at DESC NULLS LAST, c.created_at DESC NULLS LAST
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(search_filter.as_deref())
    .bind(model_filter)
    .bind(query.limit.clamp(1, 100))
    .bind(query.offset.max(0))
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let total: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM conversations c
        JOIN users u ON u.id = c.user_id
        WHERE c.active = true
          AND ($1::TEXT IS NULL OR u.email ILIKE $1 OR COALESCE(u.name, '') ILIKE $1 OR COALESCE(c.title, '') ILIKE $1)
          AND ($2::TEXT IS NULL OR c.model_slug = $2)
        "#,
    )
    .bind(search_filter.as_deref())
    .bind(model_filter)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(AdminConversationListResponse {
        items: items.into_iter().map(map_list_item).collect(),
        total,
    }))
}

pub async fn get_conversation_detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AdminConversationDetail>, (StatusCode, String)> {
    let row = sqlx::query(
        r#"
        SELECT c.id, c.user_id, u.email AS user_email, u.name AS user_name, c.title, c.model_slug,
               c.active, c.created_at, c.updated_at
        FROM conversations c
        JOIN users u ON u.id = c.user_id
        WHERE c.id = $1 AND c.active = true
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or((StatusCode::NOT_FOUND, "Conversation not found".to_string()))?;

    let messages = sqlx::query(
        r#"
        SELECT id, role, content, created_at
        FROM messages
        WHERE conversation_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
    )
    .bind(id)
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(AdminConversationDetail {
        id: row.get("id"),
        user_id: row.get("user_id"),
        user_email: row.get("user_email"),
        user_name: row.get("user_name"),
        title: row.get("title"),
        model_slug: row.get("model_slug"),
        active: row.get("active"),
        created_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
            .map(|value| value.to_rfc3339()),
        updated_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
            .map(|value| value.to_rfc3339()),
        messages: messages
            .into_iter()
            .map(|message| AdminConversationMessage {
                id: message.get("id"),
                role: message.get("role"),
                content: message.get("content"),
                created_at: message
                    .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
                    .map(|value| value.to_rfc3339()),
            })
            .collect(),
    }))
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = sqlx::query(
        r#"UPDATE conversations SET active = false, updated_at = NOW() WHERE id = $1 AND active = true"#,
    )
    .bind(id)
    .execute(&state.db)
    .await
    .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((StatusCode::NOT_FOUND, "Conversation not found".to_string()));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

fn default_limit() -> i64 {
    50
}

fn map_list_item(row: sqlx::postgres::PgRow) -> AdminConversationListItem {
    AdminConversationListItem {
        id: row.get("id"),
        user_id: row.get("user_id"),
        user_email: row.get("user_email"),
        user_name: row.get("user_name"),
        title: row.get("title"),
        model_slug: row.get("model_slug"),
        active: row.get("active"),
        created_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
            .map(|value| value.to_rfc3339()),
        updated_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("updated_at")
            .map(|value| value.to_rfc3339()),
        message_count: row.get("message_count"),
    }
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {error}"),
    )
}
