use std::collections::HashMap;

use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;

use crate::api::chat_completions::UsageContext;
use crate::db::{
    count_today_by_api_key, count_today_by_api_key_scope, count_today_by_user,
    count_today_by_user_scope, sum_daily_credits_by_api_key, sum_daily_credits_by_user,
    sum_monthly_credits_by_api_key, sum_monthly_credits_by_user,
};
use crate::error::AppError;
use crate::services::usage_metering::{UsageSnapshot, calculate_credits, parse_pricing_policy};

pub const REQUEST_KIND_CHAT: &str = "chat";
pub const REQUEST_KIND_IMAGE: &str = "image";
pub const REQUEST_KIND_VIDEO: &str = "video";

#[derive(Debug, Clone, Default, Deserialize)]
struct PlanFeatures {
    #[serde(default)]
    quota: PlanQuotaConfig,
    #[serde(default)]
    model_limits: HashMap<String, ModelLimitConfig>,
}

#[derive(Debug, Clone, Default, Deserialize)]
struct PlanQuotaConfig {
    #[serde(default)]
    daily_credits: Option<i64>,
    #[serde(default)]
    monthly_credits: Option<i64>,
    #[serde(default)]
    max_input_tokens_per_request: Option<i64>,
    #[serde(default)]
    max_output_tokens_per_request: Option<i64>,
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

fn scoped_limit_for_model(features: &PlanFeatures, model_slug: &str, request_kind: &str) -> Option<i32> {
    features
        .model_limits
        .get(model_slug)
        .and_then(|config| match request_kind {
            REQUEST_KIND_CHAT => config.chat_per_day,
            REQUEST_KIND_IMAGE => config.image_per_day,
            REQUEST_KIND_VIDEO => config.video_per_day,
            _ => None,
        })
}

fn is_limit_exceeded(limit: Option<i32>, count: i64) -> bool {
    matches!(limit, Some(value) if value >= 0 && count >= value as i64)
}

fn credit_quota_message(window: &str, used: i64, requested: i64, limit: i64, model_slug: &str) -> String {
    format!(
        "{window} credit limit exceeded for model {model_slug} ({used} used + {requested} pending > {limit})"
    )
}

async fn ensure_model_exists(pool: &PgPool, model_slug: &str) -> Result<(), AppError> {
    if crate::db::public_model_routes::get_public_model_route_by_slug(pool, model_slug).await.is_none() {
        return Err(AppError::BadRequest(format!("Unknown model slug: {model_slug}")));
    }
    Ok(())
}

async fn get_plan_rule_by_id(pool: &PgPool, plan_id: i32) -> Result<PlanAccessRule, AppError> {
    let row: Option<(Option<i32>, Option<Value>)> = sqlx::query_as(
        r#"SELECT requests_per_day, features FROM plans WHERE id = $1 AND active = true"#,
    )
    .bind(plan_id)
    .fetch_optional(pool)
    .await
    .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
    let (requests_per_day, features) = row.ok_or(AppError::PlanRequired)?;
    Ok(PlanAccessRule { plan_id, requests_per_day, features })
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
    Ok(PlanAccessRule { plan_id, requests_per_day, features })
}

async fn enforce_credit_limits(
    pool: &PgPool,
    usage_context: &UsageContext,
    plan: &PlanAccessRule,
    features: &PlanFeatures,
    model_slug: &str,
) -> Result<(), AppError> {
    if usage_context.request_kind != REQUEST_KIND_CHAT {
        return Ok(());
    }
    let max_input_limit = usage_context
        .api_key_max_input_tokens
        .map(i64::from)
        .or(features.quota.max_input_tokens_per_request);
    if let Some(limit) = max_input_limit.filter(|limit| *limit >= 0) {
        if usage_context.estimated_input_tokens > limit {
            return Err(AppError::BadRequest(format!(
                "Estimated input tokens exceed limit for model {model_slug} ({}) > {limit}",
                usage_context.estimated_input_tokens
            )));
        }
    }
    let max_output_limit = usage_context
        .api_key_max_output_tokens
        .map(i64::from)
        .or(features.quota.max_output_tokens_per_request);
    if let Some(limit) = max_output_limit.filter(|limit| *limit >= 0) {
        if let Some(requested) = usage_context
            .requested_output_tokens
            .filter(|requested| *requested > 0)
        {
            if requested > limit {
                return Err(AppError::BadRequest(format!(
                    "Requested output tokens exceed limit for model {model_slug} ({requested}) > {limit}",
                )));
            }
        }
    }
    let preflight = UsageSnapshot {
        input_tokens: usage_context.estimated_input_tokens.max(0),
        output_tokens: 0,
        cached_input_tokens: 0,
        estimated: true,
    };
    let pricing = parse_pricing_policy(plan.features.as_ref());
    let requested = calculate_credits(preflight, pricing.rates_for_model(model_slug));
    if requested <= 0 {
        return Ok(());
    }
    if let Some(key_id) = usage_context.api_key_id {
        if let Some(limit) = usage_context.api_key_daily_credit_limit {
            let used = sum_daily_credits_by_api_key(pool, key_id, Some(usage_context.request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
            if used + requested > limit {
                return Err(AppError::QuotaExceeded(credit_quota_message("Daily", used, requested, limit, model_slug)));
            }
        }
        if let Some(limit) = usage_context.api_key_monthly_credit_limit {
            let used = sum_monthly_credits_by_api_key(pool, key_id, Some(usage_context.request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
            if used + requested > limit {
                return Err(AppError::QuotaExceeded(credit_quota_message("Monthly", used, requested, limit, model_slug)));
            }
        }
    }
    if let Some(user_id) = usage_context.user_id {
        if let Some(limit) = features.quota.daily_credits {
            let used = sum_daily_credits_by_user(pool, user_id, Some(usage_context.request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
            if used + requested > limit {
                return Err(AppError::QuotaExceeded(credit_quota_message("Daily", used, requested, limit, model_slug)));
            }
        }
        if let Some(limit) = features.quota.monthly_credits {
            let used = sum_monthly_credits_by_user(pool, user_id, Some(usage_context.request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
            if used + requested > limit {
                return Err(AppError::QuotaExceeded(credit_quota_message("Monthly", used, requested, limit, model_slug)));
            }
        }
    }
    Ok(())
}

async fn enforce_request_limits(
    pool: &PgPool,
    usage_context: &UsageContext,
    plan: &PlanAccessRule,
    features: &PlanFeatures,
    model_slug: &str,
) -> Result<(), AppError> {
    let scoped_limit = scoped_limit_for_model(features, model_slug, usage_context.request_kind);
    if let Some(key_id) = usage_context.api_key_id {
        if let Some(limit) = scoped_limit {
            let today_count = count_today_by_api_key_scope(pool, key_id, Some(usage_context.request_kind), Some(model_slug))
                .await
                .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
            if is_limit_exceeded(Some(limit), today_count) {
                return Err(AppError::QuotaExceeded(format!(
                    "Daily {} limit exceeded for model {model_slug}",
                    usage_context.request_kind
                )));
            }
            return Ok(());
        }
        let request_limit = usage_context.api_key_quota_per_day.or(plan.requests_per_day);
        let today_count = count_today_by_api_key(pool, key_id)
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
        if is_limit_exceeded(request_limit, today_count) {
            return Err(AppError::QuotaExceeded(format!("Daily request limit exceeded for plan {}", plan.plan_id)));
        }
        return Ok(());
    }
    let Some(user_id) = usage_context.user_id else {
        return Ok(());
    };
    if let Some(limit) = scoped_limit {
        let today_count = count_today_by_user_scope(pool, user_id, Some(usage_context.request_kind), Some(model_slug))
            .await
            .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
        if is_limit_exceeded(Some(limit), today_count) {
            return Err(AppError::QuotaExceeded(format!(
                "Daily {} limit exceeded for model {model_slug}",
                usage_context.request_kind
            )));
        }
        return Ok(());
    }
    let today_count = count_today_by_user(pool, user_id)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {e}")))?;
    if is_limit_exceeded(plan.requests_per_day, today_count) {
        return Err(AppError::QuotaExceeded(format!("Daily request limit exceeded for plan {}", plan.plan_id)));
    }
    Ok(())
}

async fn enforce_for_plan(pool: &PgPool, usage_context: &UsageContext, plan: &PlanAccessRule, model_slug: &str) -> Result<(), AppError> {
    if !crate::db::public_models::is_public_model_allowed_for_plan(pool, plan.plan_id, model_slug).await {
        return Err(AppError::ModelNotAllowed);
    }
    let features = parse_plan_features(plan.features.as_ref());
    enforce_credit_limits(pool, usage_context, plan, &features, model_slug).await?;
    enforce_request_limits(pool, usage_context, plan, &features, model_slug).await
}

pub async fn enforce_plan_access(
    pool: &PgPool,
    user_id: Option<i32>,
    api_key_id: Option<i32>,
    api_key_plan_id: Option<i32>,
    request_kind: &str,
    model_slug: &str,
) -> Result<(), AppError> {
    let usage_context = UsageContext {
        api_key_id,
        user_id,
        plan_id: api_key_plan_id,
        provider_slug: String::new(),
        model: model_slug.to_string(),
        request_kind: if request_kind == REQUEST_KIND_CHAT { REQUEST_KIND_CHAT } else if request_kind == REQUEST_KIND_IMAGE { REQUEST_KIND_IMAGE } else { REQUEST_KIND_VIDEO },
        estimated_input_tokens: 0,
        requested_output_tokens: None,
        api_key_quota_per_day: None,
        api_key_daily_credit_limit: None,
        api_key_monthly_credit_limit: None,
        api_key_max_input_tokens: None,
        api_key_max_output_tokens: None,
    };
    enforce_usage_context(pool, &usage_context, model_slug).await.map(|_| ())
}

pub async fn enforce_usage_context(
    pool: &PgPool,
    usage_context: &UsageContext,
    model_slug: &str,
) -> Result<i32, AppError> {
    ensure_model_exists(pool, model_slug).await?;
    if let Some(plan_id) = usage_context.plan_id {
        let plan = get_plan_rule_by_id(pool, plan_id).await?;
        enforce_for_plan(pool, usage_context, &plan, model_slug).await?;
        return Ok(plan.plan_id);
    }
    let user_id = usage_context.user_id.ok_or(AppError::PlanRequired)?;
    let plan = get_user_plan_rule(pool, user_id).await?;
    enforce_for_plan(pool, usage_context, &plan, model_slug).await?;
    Ok(plan.plan_id)
}

pub async fn enforce_user_plan_access(
    pool: &PgPool,
    user_id: i32,
    request_kind: &str,
    model_slug: &str,
) -> Result<i32, AppError> {
    let usage_context = UsageContext {
        api_key_id: None,
        user_id: Some(user_id),
        plan_id: None,
        provider_slug: String::new(),
        model: model_slug.to_string(),
        request_kind: if request_kind == REQUEST_KIND_CHAT { REQUEST_KIND_CHAT } else if request_kind == REQUEST_KIND_IMAGE { REQUEST_KIND_IMAGE } else { REQUEST_KIND_VIDEO },
        estimated_input_tokens: 0,
        requested_output_tokens: None,
        api_key_quota_per_day: None,
        api_key_daily_credit_limit: None,
        api_key_monthly_credit_limit: None,
        api_key_max_input_tokens: None,
        api_key_max_output_tokens: None,
    };
    enforce_usage_context(pool, &usage_context, model_slug).await
}
