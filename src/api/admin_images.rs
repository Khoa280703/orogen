use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sqlx::Row;

use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct ImageListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct AdminImageListItem {
    pub id: i32,
    pub user_id: i32,
    pub user_email: String,
    pub user_name: Option<String>,
    pub prompt: String,
    pub model_slug: String,
    pub status: String,
    pub image_count: i64,
    pub error_message: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminImageDetail {
    pub id: i32,
    pub user_id: i32,
    pub user_email: String,
    pub user_name: Option<String>,
    pub prompt: String,
    pub model_slug: String,
    pub status: String,
    pub result_urls: Value,
    pub error_message: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AdminImageListResponse {
    pub items: Vec<AdminImageListItem>,
    pub total: i64,
}

pub async fn list_images(
    State(state): State<AppState>,
    Query(query): Query<ImageListQuery>,
) -> Result<Json<AdminImageListResponse>, (StatusCode, String)> {
    let search = query.search.as_deref().unwrap_or("").trim();
    let status = query.status.as_deref().unwrap_or("").trim();
    let search_filter = if search.is_empty() {
        None
    } else {
        Some(format!("%{search}%"))
    };
    let status_filter = if status.is_empty() {
        None
    } else {
        Some(status)
    };

    let items = sqlx::query(
        r#"
        SELECT g.id, g.user_id, u.email AS user_email, u.name AS user_name, g.prompt, g.model_slug,
               g.status, g.result_urls, g.error_message, g.created_at
        FROM image_generations g
        JOIN users u ON u.id = g.user_id
        WHERE ($1::TEXT IS NULL OR u.email ILIKE $1 OR COALESCE(u.name, '') ILIKE $1 OR g.prompt ILIKE $1)
          AND ($2::TEXT IS NULL OR g.status = $2)
        ORDER BY g.created_at DESC, g.id DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(search_filter.as_deref())
    .bind(status_filter)
    .bind(query.limit.clamp(1, 100))
    .bind(query.offset.max(0))
    .fetch_all(&state.db)
    .await
    .map_err(internal_error)?;

    let total: i64 = sqlx::query_scalar::<_, i64>(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM image_generations g
        JOIN users u ON u.id = g.user_id
        WHERE ($1::TEXT IS NULL OR u.email ILIKE $1 OR COALESCE(u.name, '') ILIKE $1 OR g.prompt ILIKE $1)
          AND ($2::TEXT IS NULL OR g.status = $2)
        "#,
    )
    .bind(search_filter.as_deref())
    .bind(status_filter)
    .fetch_one(&state.db)
    .await
    .map_err(internal_error)?;

    Ok(Json(AdminImageListResponse {
        items: items.into_iter().map(map_list_item).collect(),
        total,
    }))
}

pub async fn get_image_detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<AdminImageDetail>, (StatusCode, String)> {
    let row = sqlx::query(
        r#"
        SELECT g.id, g.user_id, u.email AS user_email, u.name AS user_name, g.prompt, g.model_slug,
               g.status, g.result_urls, g.error_message, g.created_at
        FROM image_generations g
        JOIN users u ON u.id = g.user_id
        WHERE g.id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(&state.db)
    .await
    .map_err(internal_error)?
    .ok_or((
        StatusCode::NOT_FOUND,
        "Image generation not found".to_string(),
    ))?;

    Ok(Json(AdminImageDetail {
        id: row.get("id"),
        user_id: row.get("user_id"),
        user_email: row.get("user_email"),
        user_name: row.get("user_name"),
        prompt: row.get("prompt"),
        model_slug: row.get("model_slug"),
        status: row.get("status"),
        result_urls: row.get("result_urls"),
        error_message: row.get("error_message"),
        created_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
            .map(|value| value.to_rfc3339()),
    }))
}

pub async fn delete_image(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let result = sqlx::query(r#"DELETE FROM image_generations WHERE id = $1"#)
        .bind(id)
        .execute(&state.db)
        .await
        .map_err(internal_error)?;

    if result.rows_affected() == 0 {
        return Err((
            StatusCode::NOT_FOUND,
            "Image generation not found".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({ "success": true })))
}

fn default_limit() -> i64 {
    50
}

fn map_list_item(row: sqlx::postgres::PgRow) -> AdminImageListItem {
    let result_urls: Value = row.get("result_urls");
    let image_count = result_urls
        .as_array()
        .map(|items| items.len() as i64)
        .unwrap_or(0);

    AdminImageListItem {
        id: row.get("id"),
        user_id: row.get("user_id"),
        user_email: row.get("user_email"),
        user_name: row.get("user_name"),
        prompt: row.get("prompt"),
        model_slug: row.get("model_slug"),
        status: row.get("status"),
        image_count,
        error_message: row.get("error_message"),
        created_at: row
            .get::<Option<chrono::DateTime<chrono::Utc>>, _>("created_at")
            .map(|value| value.to_rfc3339()),
    }
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {error}"),
    )
}
