use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPlan {
    pub id: i32,
    pub user_id: i32,
    pub plan_id: i32,
    pub starts_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub active: bool,
    pub created_at: Option<DateTime<Utc>>,
}

/// Assign a plan to a user
pub async fn assign_plan(
    pool: &sqlx::PgPool,
    user_id: i32,
    plan_id: i32,
    expires_at: Option<DateTime<Utc>>,
) -> Result<UserPlan, sqlx::Error> {
    // Deactivate existing active plans for this user
    sqlx::query(
        r#"
        UPDATE user_plans SET active = false WHERE user_id = $1 AND active = true
        "#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    // Insert new plan assignment
    let row = sqlx::query(
        r#"
        INSERT INTO user_plans (user_id, plan_id, expires_at)
        VALUES ($1, $2, $3)
        RETURNING id, user_id, plan_id, starts_at, expires_at, active, created_at
        "#,
    )
    .bind(user_id)
    .bind(plan_id)
    .bind(expires_at)
    .fetch_one(pool)
    .await?;

    Ok(UserPlan {
        id: row.get("id"),
        user_id: row.get("user_id"),
        plan_id: row.get("plan_id"),
        starts_at: row.get("starts_at"),
        expires_at: row.get("expires_at"),
        active: row.get("active"),
        created_at: row.get("created_at"),
    })
}

/// Get user's active plan
pub async fn get_active_plan(
    pool: &sqlx::PgPool,
    user_id: i32,
) -> Result<Option<UserPlan>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, plan_id, starts_at, expires_at, active, created_at
        FROM user_plans
        WHERE user_id = $1 AND active = true
        ORDER BY starts_at DESC
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| UserPlan {
        id: r.get("id"),
        user_id: r.get("user_id"),
        plan_id: r.get("plan_id"),
        starts_at: r.get("starts_at"),
        expires_at: r.get("expires_at"),
        active: r.get("active"),
        created_at: r.get("created_at"),
    }))
}
