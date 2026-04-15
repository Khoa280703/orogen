// DB module for providers CRUD operations
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, FromRow)]
pub struct Provider {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub active: bool,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List all active providers
pub async fn list_providers(pool: &PgPool) -> Vec<Provider> {
    sqlx::query_as(
        r#"SELECT id, name, slug, active, created_at
           FROM providers WHERE active = true
           ORDER BY name"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// Get provider by ID
pub async fn get_provider(pool: &PgPool, id: i32) -> Option<Provider> {
    sqlx::query_as(
        r#"SELECT id, name, slug, active, created_at
           FROM providers WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Create a new provider
pub async fn create_provider(pool: &PgPool, name: &str, slug: &str) -> Option<Provider> {
    sqlx::query_as(
        r#"INSERT INTO providers (name, slug, active)
           VALUES ($1, $2, true)
           RETURNING id, name, slug, active, created_at"#,
    )
    .bind(name)
    .bind(slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Update provider
pub async fn update_provider(
    pool: &PgPool,
    id: i32,
    name: Option<&str>,
    active: Option<bool>,
) -> Option<Provider> {
    let mut query = String::from("UPDATE providers SET");
    let mut updates = Vec::new();

    if let Some(n) = name {
        updates.push(format!("name = '{}'", n));
    }
    if let Some(a) = active {
        updates.push(format!("active = {}", a));
    }

    if updates.is_empty() {
        return get_provider(pool, id).await;
    }

    query.push_str(&updates.join(", "));
    query.push_str(" WHERE id = $1 RETURNING id, name, slug, active, created_at");

    sqlx::query_as(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}
