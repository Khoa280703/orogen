use chrono::{DateTime, Utc};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
#[allow(dead_code)]
pub struct PublicModelRoute {
    pub public_model_id: i32,
    pub public_model_slug: String,
    pub public_model_display_name: String,
    pub public_model_description: Option<String>,
    pub public_model_created_at: DateTime<Utc>,
    pub provider_slug: String,
    pub upstream_model_slug: String,
}

pub async fn get_public_model_route_by_slug(
    pool: &sqlx::PgPool,
    slug: &str,
) -> Option<PublicModelRoute> {
    sqlx::query_as::<_, PublicModelRoute>(
        r#"
        SELECT
            pm.id AS public_model_id,
            pm.slug AS public_model_slug,
            pm.display_name AS public_model_display_name,
            pm.description AS public_model_description,
            pm.created_at AS public_model_created_at,
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
          AND pm.slug = $1
        ORDER BY r.route_priority ASC, r.id ASC
        LIMIT 1
        "#,
    )
    .bind(slug)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}

pub async fn sync_public_catalog_for_model(
    pool: &sqlx::PgPool,
    model_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO public_models (slug, display_name, description, active, created_at)
        SELECT m.slug, m.name, m.description, m.active, m.created_at
        FROM models m
        WHERE m.id = $1
        ON CONFLICT (slug) DO UPDATE
        SET
            display_name = EXCLUDED.display_name,
            description = EXCLUDED.description,
            active = EXCLUDED.active
        "#,
    )
    .bind(model_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        INSERT INTO public_model_routes (public_model_id, provider_slug, upstream_model_slug, route_priority, active)
        SELECT pm.id, p.slug, m.slug, 0, m.active
        FROM models m
        JOIN providers p ON p.id = m.provider_id
        JOIN public_models pm ON pm.slug = m.slug
        WHERE m.id = $1
        ON CONFLICT (public_model_id, provider_slug, upstream_model_slug) DO UPDATE
        SET
            route_priority = EXCLUDED.route_priority,
            active = EXCLUDED.active
        "#,
    )
    .bind(model_id)
    .execute(pool)
    .await?;

    Ok(())
}
