// DB module for plan_models CRUD operations
use crate::db::models::{self, Model};
use sqlx::PgPool;

/// Add model to plan
pub async fn add_model_to_plan(
    pool: &PgPool,
    plan_id: i32,
    model_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"INSERT INTO plan_models (plan_id, model_id)
           VALUES ($1, $2)
           ON CONFLICT (plan_id, model_id) DO NOTHING"#,
    )
    .bind(plan_id)
    .bind(model_id)
    .execute(pool)
    .await?;

    sync_public_model_for_raw_model(pool, plan_id, model_id).await?;

    Ok(())
}

/// Remove model from plan
pub async fn remove_model_from_plan(
    pool: &PgPool,
    plan_id: i32,
    model_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"DELETE FROM plan_models
           WHERE plan_id = $1 AND model_id = $2"#,
    )
    .bind(plan_id)
    .bind(model_id)
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        DELETE FROM plan_public_models ppm
        USING models m, public_models pm
        WHERE ppm.plan_id = $1
          AND m.id = $2
          AND pm.slug = m.slug
          AND ppm.public_model_id = pm.id
        "#,
    )
    .bind(plan_id)
    .bind(model_id)
    .execute(pool)
    .await?;

    Ok(())
}

/// List all models for a plan
#[allow(dead_code)]
pub async fn list_plan_models(pool: &PgPool, plan_id: i32) -> Vec<Model> {
    models::list_models_for_plan(pool, plan_id).await
}

/// Set all models for a plan (replace existing associations)
pub async fn set_plan_models(
    pool: &PgPool,
    plan_id: i32,
    model_ids: Vec<i32>,
) -> Result<(), sqlx::Error> {
    // Delete existing associations
    sqlx::query("DELETE FROM plan_models WHERE plan_id = $1")
        .bind(plan_id)
        .execute(pool)
        .await?;

    sqlx::query("DELETE FROM plan_public_models WHERE plan_id = $1")
        .bind(plan_id)
        .execute(pool)
        .await?;

    // Insert new associations
    for model_id in model_ids {
        sqlx::query(
            r#"INSERT INTO plan_models (plan_id, model_id)
               VALUES ($1, $2)"#,
        )
        .bind(plan_id)
        .bind(model_id)
        .execute(pool)
        .await?;

        sync_public_model_for_raw_model(pool, plan_id, model_id).await?;
    }

    Ok(())
}

async fn sync_public_model_for_raw_model(
    pool: &PgPool,
    plan_id: i32,
    model_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO plan_public_models (plan_id, public_model_id)
        SELECT $1, pm.id
        FROM models m
        JOIN public_models pm ON pm.slug = m.slug
        WHERE m.id = $2
        ON CONFLICT (plan_id, public_model_id) DO NOTHING
        "#,
    )
    .bind(plan_id)
    .bind(model_id)
    .execute(pool)
    .await?;

    Ok(())
}
