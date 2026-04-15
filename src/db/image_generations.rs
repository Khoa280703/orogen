use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value;
use sqlx::Row;

#[derive(Debug, Clone, Serialize)]
pub struct ImageGeneration {
    pub id: i32,
    pub user_id: i32,
    pub prompt: String,
    pub model_slug: String,
    pub status: String,
    pub result_urls: Value,
    pub error_message: Option<String>,
    pub created_at: Option<DateTime<Utc>>,
}

pub async fn create_generation(
    pool: &sqlx::PgPool,
    user_id: i32,
    prompt: &str,
    model_slug: &str,
) -> Result<ImageGeneration, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO image_generations (user_id, prompt, model_slug)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, prompt, model_slug, status, result_urls, error_message, created_at
        "#,
    )
    .bind(user_id)
    .bind(prompt)
    .bind(model_slug)
    .fetch_one(pool)
    .await?;

    Ok(map_generation(&row))
}

pub async fn update_generation_result(
    pool: &sqlx::PgPool,
    id: i32,
    urls: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE image_generations
        SET status = 'completed', result_urls = $1, error_message = NULL
        WHERE id = $2
        "#,
    )
    .bind(urls)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn update_generation_error(
    pool: &sqlx::PgPool,
    id: i32,
    error: &str,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        UPDATE image_generations
        SET status = 'failed', error_message = $1
        WHERE id = $2
        "#,
    )
    .bind(error)
    .bind(id)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn list_generations(
    pool: &sqlx::PgPool,
    user_id: i32,
    limit: i64,
    offset: i64,
) -> Result<Vec<ImageGeneration>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, prompt, model_slug, status, result_urls, error_message, created_at
        FROM image_generations
        WHERE user_id = $1
        ORDER BY created_at DESC, id DESC
        LIMIT $2 OFFSET $3
        "#,
    )
    .bind(user_id)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await?;

    Ok(rows.iter().map(map_generation).collect())
}

pub async fn get_generation(
    pool: &sqlx::PgPool,
    id: i32,
    user_id: i32,
) -> Result<Option<ImageGeneration>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, prompt, model_slug, status, result_urls, error_message, created_at
        FROM image_generations
        WHERE id = $1 AND user_id = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.as_ref().map(map_generation))
}

fn map_generation(row: &sqlx::postgres::PgRow) -> ImageGeneration {
    ImageGeneration {
        id: row.get("id"),
        user_id: row.get("user_id"),
        prompt: row.get("prompt"),
        model_slug: row.get("model_slug"),
        status: row.get("status"),
        result_urls: row.get("result_urls"),
        error_message: row.get("error_message"),
        created_at: row.get("created_at"),
    }
}
