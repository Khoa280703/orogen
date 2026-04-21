use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Deserialize;
use serde_json::json;

use crate::AppState;
use crate::api::chat_completions::{
    ApiKey, duration_to_latency_ms, record_usage, resolve_usage_context, touch_api_key_last_used,
};
use crate::api::consumer_api_support::{generate_images_with_retry, mark_account_success};
use crate::api::plan_enforcement::{REQUEST_KIND_IMAGE, enforce_plan_access};
use crate::error::AppError;
use crate::providers::GeneratedAsset;

const DEFAULT_IMAGE_MODEL: &str = "imagine-x-1";

#[derive(Debug, Deserialize)]
pub struct ImageGenerationRequest {
    pub prompt: String,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub n: Option<u32>,
}

pub async fn generate_images(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<ImageGenerationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let prompt = body.prompt.trim();
    if prompt.is_empty() {
        return Err(AppError::GrokApi("prompt is required".into()));
    }

    let api_key = extract_api_key(&headers);
    track_api_key_usage(&state, &api_key).await;

    let requested_model = body
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_IMAGE_MODEL.to_string());
    let resolved_model =
        crate::db::models::get_model_with_provider_by_slug(&state.db, &requested_model)
            .await
            .ok_or_else(|| {
                AppError::BadRequest(format!("Unknown or inactive model: {requested_model}"))
            })?;
    let usage_context = resolve_usage_context(
        &state,
        &api_key,
        &resolved_model.provider_slug,
        &resolved_model.slug,
        REQUEST_KIND_IMAGE,
    )
    .await?;

    enforce_plan_access(
        &state.db,
        usage_context.user_id,
        usage_context.api_key_id,
        usage_context.plan_id,
        usage_context.request_kind,
        &resolved_model.slug,
    )
    .await?;

    let provider = state
        .providers
        .get_image_provider(&resolved_model.provider_slug)
        .ok_or_else(|| {
            AppError::Internal(format!(
                "Image provider not registered: {}",
                resolved_model.provider_slug
            ))
        })?;
    let started_at = std::time::Instant::now();

    match generate_images_with_retry(
        &state,
        provider,
        &resolved_model.provider_slug,
        &resolved_model.slug,
        prompt,
        &usage_context,
    )
    .await
    {
        Ok((assets, account_id, _account_name)) => {
            mark_account_success(&state.db, account_id).await;
            record_usage(
                &state,
                &usage_context,
                account_id,
                "success",
                duration_to_latency_ms(started_at.elapsed()),
            )
            .await;
            touch_api_key_last_used(&state, &usage_context).await;
            Ok(build_image_response(assets, body.n))
        }
        Err(error) => Err(error),
    }
}

fn build_image_response(
    mut assets: Vec<GeneratedAsset>,
    requested_n: Option<u32>,
) -> Json<serde_json::Value> {
    if let Some(limit) = requested_n {
        assets.truncate(limit as usize);
    }

    Json(json!({
        "created": chrono::Utc::now().timestamp(),
        "data": assets.into_iter().map(|asset| {
            json!({
                "id": asset.id,
                "url": asset.url,
                "revised_prompt": serde_json::Value::Null,
            })
        }).collect::<Vec<_>>(),
        "requested_n": requested_n.unwrap_or(0),
        "errors": Vec::<String>::new(),
    }))
}

async fn track_api_key_usage(state: &AppState, api_key: &ApiKey) {
    if api_key.0.is_empty() {
        return;
    }

    let mut counts = state.key_request_counts.write().await;
    *counts.entry(api_key.0.clone()).or_insert(0) += 1;
}

fn extract_api_key(headers: &HeaderMap) -> ApiKey {
    let value = headers
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .map(str::to_string)
        .or_else(|| {
            headers
                .get("x-api-key")
                .and_then(|v| v.to_str().ok())
                .map(str::to_string)
        })
        .unwrap_or_default();
    ApiKey(value)
}
