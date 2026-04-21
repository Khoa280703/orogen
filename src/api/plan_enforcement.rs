use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;

use crate::db::{
    count_today_by_api_key, count_today_by_api_key_scope, count_today_by_user,
    count_today_by_user_scope,
};
use crate::error::AppError;

pub const REQUEST_KIND_CHAT: &str = "chat";
pub const REQUEST_KIND_IMAGE: &str = "image";
pub const REQUEST_KIND_VIDEO: &str = "video";

#[derive(Debug, Clone, Default, Deserialize)]
struct PlanFeatures {
    #[serde(default)]
    model_limits: HashMap<String, ModelLimitConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct ModelLimitConfig {
    #[serde(default)]
    chat_per_day: Option<i32>,
    #[serde(default)]
    image_per_day: Option<i32>,
    #[serde(default)]
    video_per_day: Option<i32>,
}

impl ModelLimitConfig {
    fn limit_for_kind(&self, request_kind: &str) -> Option<i32> {
        match request_kind {
            REQUEST_KIND_CHAT => self.chat_per_day,
            REQUEST_KIND_IMAGE => self.image_per_day,
            REQUEST_KIND_VIDEO => self.video_per_day,
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
struct PlanAccessRule {
    plan_id: i32,
    requests_per_day: Option<i32>,
    features: Option<Value>,
}

fn parse_plan_features(features: Option<&Value>) -> PlanFeatures {
    features
        .cloned()
        .and_then(|value| serde_json::from_value::<PlanFeatures>(value).ok())
        .unwrap_or_default()
}

fn scoped_limit_for_model(
    features: Option<&Value>,
    model_slug: &str,
    request_kind: &str,
) -> Option<i32> {
    parse_plan_features(features)
        .model_limits
        .get(model_slug)
        .and_then(|config| config.limit_for_kind(request_kind))
}

fn is_limit_exceeded(limit: Option<i32>, count: i64) -> bool {
    match limit {
        Some(value) if value >= 0 => count >= value as i64,
        _ => false,
    }
}

fn quota_message(request_kind: &str, model_slug: &str) -> String {
    format!("Daily {request_kind} limit exceeded for model {model_slug}")
}

async fn ensure_model_exists(pool: &PgPool, model_slug: &str) -> Result<(), AppError> {
    if crate::db::public_model_routes::get_public_model_route_by_slug(pool, model_slug)
        .await
        .is_none()
    {
        return Err(AppError::BadRequest(format!(
            "Unknown model slug: {model_slug}"
        )));
    }

    Ok(())
}

async fn get_plan_rule_by_id(pool: &PgPool, plan_id: i32) -> Result<PlanAccessRule, AppError> {
    let row: Option<(Option<i32>, Option<Value>)> = sqlx::query_as(
        r#"
        SELECT requests_per_day, features
        FROM plans
        WHERE id = $1 AND active = true
        "#,
    )
    .bind(plan_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    let (requests_per_day, features) = row.ok_or(AppError::PlanRequired)?;

    Ok(PlanAccessRule {
        plan_id,
        requests_per_day,
        features,
    })
}

async fn get_user_plan_rule(pool: &PgPool, user_id: i32) -> Result<PlanAccessRule, AppError> {
    let row: Option<(i32, Option<i32>, Option<Value>)> = sqlx::query_as(
        r#"
        SELECT up.plan_id, p.requests_per_day, p.features
        FROM user_plans up
        JOIN plans p ON up.plan_id = p.id
        WHERE up.user_id = $1 AND up.active = true AND p.active = true
        LIMIT 1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    let (plan_id, requests_per_day, features) = row.ok_or(AppError::PlanRequired)?;

    Ok(PlanAccessRule {
        plan_id,
        requests_per_day,
        features,
    })
}

async fn enforce_api_key_limit(
    pool: &PgPool,
    api_key_id: i32,
    plan: &PlanAccessRule,
    request_kind: &str,
    model_slug: &str,
) -> Result<(), AppError> {
    let scoped_limit = scoped_limit_for_model(plan.features.as_ref(), model_slug, request_kind);

    if let Some(limit) = scoped_limit {
        let today_count =
            count_today_by_api_key_scope(pool, api_key_id, Some(request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

        if is_limit_exceeded(Some(limit), today_count) {
            return Err(AppError::QuotaExceeded(quota_message(
                request_kind,
                model_slug,
            )));
        }

        return Ok(());
    }

    let today_count = count_today_by_api_key(pool, api_key_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    if is_limit_exceeded(plan.requests_per_day, today_count) {
        return Err(AppError::QuotaExceeded(format!(
            "Daily request limit exceeded for plan {}",
            plan.plan_id
        )));
    }

    Ok(())
}

async fn enforce_user_limit(
    pool: &PgPool,
    user_id: i32,
    plan: &PlanAccessRule,
    request_kind: &str,
    model_slug: &str,
) -> Result<(), AppError> {
    let scoped_limit = scoped_limit_for_model(plan.features.as_ref(), model_slug, request_kind);

    if let Some(limit) = scoped_limit {
        let today_count =
            count_today_by_user_scope(pool, user_id, Some(request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

        if is_limit_exceeded(Some(limit), today_count) {
            return Err(AppError::QuotaExceeded(quota_message(
                request_kind,
                model_slug,
            )));
        }

        return Ok(());
    }

    let today_count = count_today_by_user(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;

    if is_limit_exceeded(plan.requests_per_day, today_count) {
        return Err(AppError::QuotaExceeded(format!(
            "Daily request limit exceeded for plan {}",
            plan.plan_id
        )));
    }

    Ok(())
}

pub async fn enforce_plan_access(
    pool: &PgPool,
    user_id: Option<i32>,
    api_key_id: Option<i32>,
    api_key_plan_id: Option<i32>,
    request_kind: &str,
    model_slug: &str,
) -> Result<(), AppError> {
    ensure_model_exists(pool, model_slug).await?;

    if let Some(plan_id) = api_key_plan_id {
        let plan = get_plan_rule_by_id(pool, plan_id).await?;

        if !crate::db::public_models::is_public_model_allowed_for_plan(
            pool,
            plan.plan_id,
            model_slug,
        )
        .await
        {
            return Err(AppError::ModelNotAllowed);
        }

        if let Some(key_id) = api_key_id {
            enforce_api_key_limit(pool, key_id, &plan, request_kind, model_slug).await?;
        }

        return Ok(());
    }

    let user_id = match user_id {
        Some(id) => id,
        None => return Ok(()),
    };

    let plan = get_user_plan_rule(pool, user_id).await?;

    if !crate::db::public_models::is_public_model_allowed_for_plan(pool, plan.plan_id, model_slug)
        .await
    {
        return Err(AppError::ModelNotAllowed);
    }

    enforce_user_limit(pool, user_id, &plan, request_kind, model_slug).await
}

pub async fn enforce_user_plan_access(
    pool: &PgPool,
    user_id: i32,
    request_kind: &str,
    model_slug: &str,
) -> Result<i32, AppError> {
    ensure_model_exists(pool, model_slug).await?;

    let plan = get_user_plan_rule(pool, user_id).await?;

    if !crate::db::public_models::is_public_model_allowed_for_plan(pool, plan.plan_id, model_slug)
        .await
    {
        return Err(AppError::ModelNotAllowed);
    }

    enforce_user_limit(pool, user_id, &plan, request_kind, model_slug).await?;

    Ok(plan.plan_id)
}
