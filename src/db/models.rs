// DB module for models CRUD operations
use sqlx::FromRow;
use sqlx::PgPool;

#[derive(Debug, Clone, FromRow)]
pub struct Model {
    pub id: i32,
    pub provider_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub active: bool,
    pub sort_order: i32,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct ModelWithProvider {
    pub provider_slug: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

/// List all active models (ordered by provider, sort_order)
pub async fn list_models(pool: &PgPool) -> Vec<Model> {
    sqlx::query_as(
        r#"SELECT m.id, m.provider_id, m.name, m.slug, m.description, m.active, m.sort_order, m.created_at
           FROM models m
           JOIN providers p ON m.provider_id = p.id
           WHERE m.active = true AND p.active = true
           ORDER BY p.name, m.sort_order"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

#[allow(dead_code)]
pub async fn list_models_with_provider(pool: &PgPool) -> Vec<ModelWithProvider> {
    sqlx::query_as(
        r#"SELECT p.slug AS provider_slug, m.name, m.slug, m.description, m.created_at
           FROM models m
           JOIN providers p ON m.provider_id = p.id
           WHERE m.active = true AND p.active = true
           ORDER BY p.name, m.sort_order"#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// List models by provider ID
#[allow(dead_code)]
pub async fn list_models_by_provider(pool: &PgPool, provider_id: i32) -> Vec<Model> {
    sqlx::query_as(
        r#"SELECT id, provider_id, name, slug, description, active, sort_order, created_at
           FROM models
           WHERE provider_id = $1 AND active = true
           ORDER BY sort_order"#,
    )
    .bind(provider_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// Get model by slug
#[allow(dead_code)]
pub async fn get_model_by_slug(pool: &PgPool, slug: &str) -> Option<Model> {
    sqlx::query_as(
        r#"SELECT m.id, m.provider_id, m.name, m.slug, m.description, m.active, m.sort_order, m.created_at
           FROM models m
           JOIN providers p ON m.provider_id = p.id
           WHERE m.slug = $1 AND m.active = true AND p.active = true"#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn get_model_with_provider_by_slug(
    pool: &PgPool,
    slug: &str,
) -> Option<ModelWithProvider> {
    sqlx::query_as(
        r#"SELECT p.slug AS provider_slug, m.name, m.slug, m.description, m.created_at
           FROM models m
           JOIN providers p ON m.provider_id = p.id
           WHERE m.slug = $1 AND m.active = true AND p.active = true"#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// List models allowed for a specific plan (JOIN plan_models)
pub async fn list_models_for_plan(pool: &PgPool, plan_id: i32) -> Vec<Model> {
    sqlx::query_as(
        r#"SELECT m.id, m.provider_id, m.name, m.slug, m.description, m.active, m.sort_order, m.created_at
           FROM models m
           JOIN plan_models pm ON m.id = pm.model_id
           JOIN providers p ON m.provider_id = p.id
           WHERE pm.plan_id = $1 AND m.active = true AND p.active = true
           ORDER BY p.name, m.sort_order"#,
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

#[allow(dead_code)]
pub async fn list_models_with_provider_for_plan(
    pool: &PgPool,
    plan_id: i32,
) -> Vec<ModelWithProvider> {
    sqlx::query_as(
        r#"SELECT p.slug AS provider_slug, m.name, m.slug, m.description, m.created_at
           FROM models m
           JOIN plan_models pm ON m.id = pm.model_id
           JOIN providers p ON m.provider_id = p.id
           WHERE pm.plan_id = $1 AND m.active = true AND p.active = true
           ORDER BY p.name, m.sort_order"#,
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

/// Check if model is allowed for a plan
#[allow(dead_code)]
pub async fn is_model_allowed_for_plan(pool: &PgPool, plan_id: i32, model_slug: &str) -> bool {
    let result: Option<i32> = sqlx::query_scalar(
        r#"SELECT m.id
           FROM models m
           JOIN plan_models pm ON m.id = pm.model_id
           JOIN providers p ON m.provider_id = p.id
           WHERE pm.plan_id = $1 AND m.slug = $2 AND m.active = true AND p.active = true
           LIMIT 1"#,
    )
    .bind(plan_id)
    .bind(model_slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten();

    result.is_some()
}

/// Create a new model
pub async fn create_model(
    pool: &PgPool,
    provider_id: i32,
    name: &str,
    slug: &str,
    description: Option<&str>,
    sort_order: i32,
) -> Option<Model> {
    sqlx::query_as(
        r#"INSERT INTO models (provider_id, name, slug, description, active, sort_order)
           VALUES ($1, $2, $3, $4, true, $5)
           RETURNING id, provider_id, name, slug, description, active, sort_order, created_at"#,
    )
    .bind(provider_id)
    .bind(name)
    .bind(slug)
    .bind(description)
    .bind(sort_order)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

/// Update model
pub async fn update_model(
    pool: &PgPool,
    id: i32,
    name: Option<&str>,
    description: Option<&str>,
    active: Option<bool>,
    sort_order: Option<i32>,
) -> Option<Model> {
    let mut query = String::from("UPDATE models SET");
    let mut updates = Vec::new();

    if let Some(n) = name {
        updates.push(format!("name = '{}'", n));
    }
    if let Some(d) = description {
        updates.push(format!("description = '{}'", d.replace('\'', "''")));
    }
    if let Some(a) = active {
        updates.push(format!("active = {}", a));
    }
    if let Some(s) = sort_order {
        updates.push(format!("sort_order = {}", s));
    }

    if updates.is_empty() {
        return get_model_by_id(pool, id).await;
    }

    query.push_str(&updates.join(", "));
    query.push_str(
        " WHERE id = $1 RETURNING id, provider_id, name, slug, description, active, sort_order, created_at",
    );

    sqlx::query_as(&query)
        .bind(id)
        .fetch_optional(pool)
        .await
        .ok()
        .flatten()
}

/// Get model by ID (helper for update)
async fn get_model_by_id(pool: &PgPool, id: i32) -> Option<Model> {
    sqlx::query_as(
        r#"SELECT id, provider_id, name, slug, description, active, sort_order, created_at
           FROM models WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .ok()?
}
