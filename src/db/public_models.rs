use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct PublicModelWithRoute {
    pub public_model_id: i32,
    pub slug: String,
    pub display_name: String,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub provider_slug: String,
    pub upstream_model_slug: String,
}

pub async fn list_public_models(pool: &sqlx::PgPool) -> Vec<PublicModelWithRoute> {
    sqlx::query_as::<_, PublicModelWithRoute>(
        r#"
        SELECT DISTINCT ON (pm.slug)
            pm.id AS public_model_id,
            pm.slug,
            pm.display_name,
            pm.description,
            pm.created_at,
            r.provider_slug,
            r.upstream_model_slug
        FROM public_models pm
        JOIN public_model_routes r
          ON r.public_model_id = pm.id
         AND r.active = true
        JOIN providers p
          ON p.slug = r.provider_slug
         AND p.active = true
        WHERE pm.active = true
        ORDER BY pm.slug ASC, r.route_priority ASC, r.id ASC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn list_public_models_for_plan(
    pool: &sqlx::PgPool,
    plan_id: i32,
) -> Vec<PublicModelWithRoute> {
    sqlx::query_as::<_, PublicModelWithRoute>(
        r#"
        SELECT DISTINCT ON (pm.slug)
            pm.id AS public_model_id,
            pm.slug,
            pm.display_name,
            pm.description,
            pm.created_at,
            r.provider_slug,
            r.upstream_model_slug
        FROM public_models pm
        JOIN public_model_routes r
          ON r.public_model_id = pm.id
         AND r.active = true
        JOIN providers p
          ON p.slug = r.provider_slug
         AND p.active = true
        JOIN plan_public_models ppm
          ON ppm.public_model_id = pm.id
        WHERE pm.active = true
          AND ppm.plan_id = $1
        ORDER BY pm.slug ASC, r.route_priority ASC, r.id ASC
        "#,
    )
    .bind(plan_id)
    .fetch_all(pool)
    .await
    .unwrap_or_default()
}

pub async fn is_public_model_allowed_for_plan(
    pool: &sqlx::PgPool,
    plan_id: i32,
    public_model_slug: &str,
) -> bool {
    sqlx::query_scalar::<_, i32>(
        r#"
        SELECT pm.id
        FROM public_models pm
        JOIN plan_public_models ppm
          ON ppm.public_model_id = pm.id
        WHERE ppm.plan_id = $1
          AND pm.slug = $2
          AND pm.active = true
        LIMIT 1
        "#,
    )
    .bind(plan_id)
    .bind(public_model_slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
    .is_some()
}
