use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

use crate::AppState;
use crate::account::pool::CurrentAccount;
use crate::api::chat_completions::{
    ApiKey, duration_to_latency_ms, record_usage, request_error_status, resolve_usage_context,
    touch_api_key_last_used,
};
use crate::api::plan_enforcement::{REQUEST_KIND_VIDEO, enforce_plan_access};
use crate::error::AppError;
use crate::grok::client::GrokRequestError;
use crate::grok::media_response_parser::parse_video_generation_body;
use crate::grok::types::GrokRequest;
use crate::services::proxy_failover;

const GROK_MEDIA_CREATE_POST_URL: &str = "https://grok.com/rest/media/post/create";
const DEFAULT_VIDEO_MODEL: &str = "grok-3";

#[derive(Debug, Deserialize)]
pub struct VideoGenerationRequest {
    #[serde(default)]
    pub image_url: Option<String>,
    #[serde(default)]
    pub prompt: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub mode: Option<String>,
    #[serde(default)]
    pub aspect_ratio: Option<String>,
    #[serde(default)]
    pub duration_seconds: Option<u32>,
    #[serde(default)]
    pub resolution: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MediaCreatePostRequest {
    media_type: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    media_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    prompt: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct MediaCreatePostResponse {
    post: CreatedMediaPost,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CreatedMediaPost {
    id: String,
}

struct PreparedVideoRequest {
    payload: GrokRequest,
    mode: String,
    duration_seconds: u32,
    resolution: String,
}

pub async fn generate_videos(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(body): Json<VideoGenerationRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let trimmed_image_url = body.image_url.as_deref().map(str::trim).unwrap_or_default();
    let trimmed_prompt = body.prompt.as_deref().map(str::trim).unwrap_or_default();
    if trimmed_image_url.is_empty() && trimmed_prompt.is_empty() {
        return Err(AppError::GrokApi(
            "prompt is required when image_url is empty".into(),
        ));
    }

    let api_key = extract_api_key(&headers);
    track_api_key_usage(&state, &api_key).await;

    let requested_model = body
        .model
        .clone()
        .unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string());
    let usage_context =
        resolve_usage_context(&state, &api_key, &requested_model, REQUEST_KIND_VIDEO).await?;

    enforce_plan_access(
        &state.db,
        usage_context.user_id,
        usage_context.api_key_id,
        usage_context.plan_id,
        usage_context.request_kind,
        &requested_model,
    )
    .await?;

    let account = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
    let start = std::time::Instant::now();

    match run_video_generation_flow(&state, &account, &body, &requested_model).await {
        Ok((raw_body, prepared)) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            state.pool.mark_success().await;
            record_usage(&state, &usage_context, account.id, "success", latency_ms).await;
            touch_api_key_last_used(&state, &usage_context).await;
            build_video_generation_response(raw_body, prepared)
        }
        Err(GrokRequestError::ProxyFailed(message)) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            if let Some(next) = proxy_failover::deactivate_failed_proxy(&state, &account, &message).await {
                match run_video_generation_flow(&state, &next, &body, &requested_model).await {
                    Ok((raw_body, prepared)) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        state.pool.mark_success().await;
                        record_usage(&state, &usage_context, next.id, "success", total_latency_ms)
                            .await;
                        touch_api_key_last_used(&state, &usage_context).await;
                        build_video_generation_response(raw_body, prepared)
                    }
                    Err(retry_error) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        if matches!(retry_error, GrokRequestError::RateLimited) {
                            state.pool.mark_rate_limited().await;
                        }
                        record_usage(
                            &state,
                            &usage_context,
                            next.id,
                            request_error_status(&retry_error),
                            total_latency_ms,
                        )
                        .await;
                        Err(AppError::GrokApi(retry_error.to_string()))
                    }
                }
            } else {
                record_usage(&state, &usage_context, account.id, "proxy_failed", latency_ms).await;
                Err(AppError::GrokApi(message))
            }
        }
        Err(GrokRequestError::CfBlocked) if account.proxy_id.is_some() => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            if let Some(next) = proxy_failover::deactivate_failed_proxy(
                &state,
                &account,
                "Proxy received Cloudflare block from upstream.",
            )
            .await
            {
                match run_video_generation_flow(&state, &next, &body, &requested_model).await {
                    Ok((raw_body, prepared)) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        state.pool.mark_success().await;
                        record_usage(&state, &usage_context, next.id, "success", total_latency_ms)
                            .await;
                        touch_api_key_last_used(&state, &usage_context).await;
                        build_video_generation_response(raw_body, prepared)
                    }
                    Err(retry_error) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        if matches!(retry_error, GrokRequestError::RateLimited) {
                            state.pool.mark_rate_limited().await;
                        }
                        record_usage(
                            &state,
                            &usage_context,
                            next.id,
                            request_error_status(&retry_error),
                            total_latency_ms,
                        )
                        .await;
                        Err(AppError::GrokApi(retry_error.to_string()))
                    }
                }
            } else {
                record_usage(&state, &usage_context, account.id, "cf_blocked", latency_ms).await;
                Err(AppError::GrokApi("Cloudflare blocked".into()))
            }
        }
        Err(GrokRequestError::RateLimited) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            state.pool.mark_rate_limited().await;

            if state.pool.rotate().await {
                let next = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
                match run_video_generation_flow(&state, &next, &body, &requested_model).await {
                    Ok((raw_body, prepared)) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        state.pool.mark_success().await;
                        record_usage(&state, &usage_context, next.id, "success", total_latency_ms)
                            .await;
                        touch_api_key_last_used(&state, &usage_context).await;
                        build_video_generation_response(raw_body, prepared)
                    }
                    Err(retry_error) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        if matches!(retry_error, GrokRequestError::RateLimited) {
                            state.pool.mark_rate_limited().await;
                        } else {
                            state.pool.mark_failure().await;
                        }
                        record_usage(
                            &state,
                            &usage_context,
                            next.id,
                            request_error_status(&retry_error),
                            total_latency_ms,
                        )
                        .await;
                        Err(AppError::GrokApi(retry_error.to_string()))
                    }
                }
            } else {
                record_usage(
                    &state,
                    &usage_context,
                    account.id,
                    "rate_limited",
                    latency_ms,
                )
                .await;
                Err(AppError::GrokApi("Rate limited".into()))
            }
        }
        Err(error @ GrokRequestError::Unauthorized) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            if let Some(id) = account.id {
                let _ = crate::db::account_sessions::mark_session_expired(
                    &state.db,
                    id,
                    "Upstream Grok session expired or cookies are invalid.",
                )
                .await;
            }

            if state.pool.rotate().await {
                let next = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
                match run_video_generation_flow(&state, &next, &body, &requested_model).await {
                    Ok((raw_body, prepared)) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        state.pool.mark_success().await;
                        record_usage(&state, &usage_context, next.id, "success", total_latency_ms)
                            .await;
                        touch_api_key_last_used(&state, &usage_context).await;
                        build_video_generation_response(raw_body, prepared)
                    }
                    Err(retry_error) => {
                        let total_latency_ms = duration_to_latency_ms(start.elapsed());
                        if matches!(retry_error, GrokRequestError::Unauthorized) {
                            if let Some(id) = next.id {
                                let _ = crate::db::account_sessions::mark_session_expired(
                                    &state.db,
                                    id,
                                    "Upstream Grok session expired or cookies are invalid.",
                                )
                                .await;
                            }
                        } else {
                            state.pool.mark_failure().await;
                        }
                        record_usage(
                            &state,
                            &usage_context,
                            next.id,
                            request_error_status(&retry_error),
                            total_latency_ms,
                        )
                        .await;
                        Err(AppError::GrokApi(retry_error.to_string()))
                    }
                }
            } else {
                record_usage(
                    &state,
                    &usage_context,
                    account.id,
                    request_error_status(&error),
                    latency_ms,
                )
                .await;
                Err(AppError::GrokApi("All accounts exhausted".into()))
            }
        }
        Err(GrokRequestError::CfBlocked) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            record_usage(&state, &usage_context, account.id, "cf_blocked", latency_ms).await;
            Err(AppError::GrokApi("Cloudflare blocked".into()))
        }
        Err(error) => {
            let latency_ms = duration_to_latency_ms(start.elapsed());
            record_usage(
                &state,
                &usage_context,
                account.id,
                request_error_status(&error),
                latency_ms,
            )
            .await;
            Err(AppError::GrokApi(error.to_string()))
        }
    }
}

async fn run_video_generation_flow(
    state: &AppState,
    account: &CurrentAccount,
    body: &VideoGenerationRequest,
    effective_model: &str,
) -> Result<(String, PreparedVideoRequest), GrokRequestError> {
    let parent_post_id = create_parent_post(state, account, body).await?;
    let prepared = prepare_video_request(body, effective_model.to_string(), parent_post_id);
    let proxy_ref = account.proxy_url.as_ref();
    let raw_body = state
        .grok
        .send_request(&account.cookies, &prepared.payload, proxy_ref)
        .await?;
    Ok((raw_body, prepared))
}

async fn create_parent_post(
    state: &AppState,
    account: &CurrentAccount,
    body: &VideoGenerationRequest,
) -> Result<String, GrokRequestError> {
    let trimmed_image_url = body
        .image_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let trimmed_prompt = body
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());

    let create_payload = if let Some(image_url) = trimmed_image_url {
        MediaCreatePostRequest {
            media_type: "MEDIA_POST_TYPE_IMAGE",
            media_url: Some(image_url.to_string()),
            prompt: None,
        }
    } else {
        MediaCreatePostRequest {
            media_type: "MEDIA_POST_TYPE_VIDEO",
            media_url: None,
            prompt: trimmed_prompt.map(str::to_string),
        }
    };

    let proxy_ref = account.proxy_url.as_ref();
    let raw_body = state
        .grok
        .send_json_request(
            GROK_MEDIA_CREATE_POST_URL,
            &account.cookies,
            &create_payload,
            proxy_ref,
        )
        .await?;
    let response: MediaCreatePostResponse = serde_json::from_str(&raw_body).map_err(|error| {
        GrokRequestError::Network(format!("Invalid media post response: {error}"))
    })?;
    Ok(response.post.id)
}

fn prepare_video_request(
    body: &VideoGenerationRequest,
    model_name: String,
    parent_post_id: String,
) -> PreparedVideoRequest {
    let trimmed_image_url = body
        .image_url
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let trimmed_prompt = body
        .prompt
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("");
    let mode = body.mode.clone().unwrap_or_else(|| {
        if trimmed_prompt.is_empty() {
            "normal".to_string()
        } else {
            "custom".to_string()
        }
    });
    let aspect_ratio = body
        .aspect_ratio
        .clone()
        .unwrap_or_else(|| "2:3".to_string());
    let duration_seconds = body.duration_seconds.unwrap_or(6);
    let resolution = body
        .resolution
        .clone()
        .unwrap_or_else(|| "480p".to_string());

    let message = if let Some(image_url) = trimmed_image_url {
        format!("{image_url} {trimmed_prompt} --mode={mode}")
    } else {
        format!("{trimmed_prompt} --mode={mode}")
    }
    .trim()
    .to_string();

    PreparedVideoRequest {
        payload: GrokRequest::new_video_generation(
            message,
            model_name,
            parent_post_id,
            aspect_ratio,
            duration_seconds,
            resolution.clone(),
        ),
        mode,
        duration_seconds,
        resolution,
    }
}

fn build_video_generation_response(
    raw_body: String,
    prepared: PreparedVideoRequest,
) -> Result<Json<serde_json::Value>, AppError> {
    let parsed = parse_video_generation_body(&raw_body);

    if parsed.assets.is_empty() && !parsed.errors.is_empty() {
        return Err(AppError::GrokApi(parsed.errors.join(" | ")));
    }

    Ok(Json(json!({
        "created": chrono::Utc::now().timestamp(),
        "data": parsed.assets.into_iter().map(|asset| {
            json!({
                "id": asset.id,
                "url": asset.url,
                "model_name": asset.model_name,
                "resolution_name": asset.resolution_name,
            })
        }).collect::<Vec<_>>(),
        "mode": prepared.mode,
        "duration_seconds": prepared.duration_seconds,
        "resolution": prepared.resolution,
        "errors": parsed.errors,
    })))
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
