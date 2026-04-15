use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Plan {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<serde_json::Value>,
    pub active: bool,
    pub sort_order: i32,
    pub created_at: Option<DateTime<Utc>>,
}

/// List public plans
pub async fn list_plans(pool: &sqlx::PgPool) -> Result<Vec<Plan>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, slug, requests_per_day, requests_per_month,
               price_usd::text as price_usd, price_vnd, features, active, sort_order, created_at
        FROM plans
        WHERE active = true
        ORDER BY sort_order ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Plan {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            requests_per_day: r.get("requests_per_day"),
            requests_per_month: r.get("requests_per_month"),
            price_usd: r.get("price_usd"),
            price_vnd: r.get("price_vnd"),
            features: r.get("features"),
            active: r.get("active"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
        })
        .collect::<Vec<_>>())
}

/// List all plans for admin
pub async fn list_all_plans(pool: &sqlx::PgPool) -> Result<Vec<Plan>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, slug, requests_per_day, requests_per_month,
               price_usd::text as price_usd, price_vnd, features, active, sort_order, created_at
        FROM plans
        ORDER BY sort_order ASC, id ASC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Plan {
            id: r.get("id"),
            name: r.get("name"),
            slug: r.get("slug"),
            requests_per_day: r.get("requests_per_day"),
            requests_per_month: r.get("requests_per_month"),
            price_usd: r.get("price_usd"),
            price_vnd: r.get("price_vnd"),
            features: r.get("features"),
            active: r.get("active"),
            sort_order: r.get("sort_order"),
            created_at: r.get("created_at"),
        })
        .collect::<Vec<_>>())
}

/// Get plan by ID
pub async fn get_plan(pool: &sqlx::PgPool, id: i32) -> Result<Option<Plan>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, name, slug, requests_per_day, requests_per_month,
               price_usd::text as price_usd, price_vnd, features, active, sort_order, created_at
        FROM plans
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Plan {
        id: r.get("id"),
        name: r.get("name"),
        slug: r.get("slug"),
        requests_per_day: r.get("requests_per_day"),
        requests_per_month: r.get("requests_per_month"),
        price_usd: r.get("price_usd"),
        price_vnd: r.get("price_vnd"),
        features: r.get("features"),
        active: r.get("active"),
        sort_order: r.get("sort_order"),
        created_at: r.get("created_at"),
    }))
}

/// Get plan by slug
pub async fn get_plan_by_slug(
    pool: &sqlx::PgPool,
    slug: &str,
) -> Result<Option<Plan>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, name, slug, requests_per_day, requests_per_month,
               price_usd::text as price_usd, price_vnd, features, active, sort_order, created_at
        FROM plans
        WHERE slug = $1 AND active = true
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Plan {
        id: r.get("id"),
        name: r.get("name"),
        slug: r.get("slug"),
        requests_per_day: r.get("requests_per_day"),
        requests_per_month: r.get("requests_per_month"),
        price_usd: r.get("price_usd"),
        price_vnd: r.get("price_vnd"),
        features: r.get("features"),
        active: r.get("active"),
        sort_order: r.get("sort_order"),
        created_at: r.get("created_at"),
    }))
}
