use axum::{Json, extract::{Extension, State}};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::api::chat_completions::{duration_to_latency_ms, record_usage};
use crate::api::consumer_api_support::{build_user_usage_context, mark_account_success};
use crate::api::plan_enforcement::{REQUEST_KIND_VIDEO, enforce_user_plan_access};
use crate::error::AppError;
use crate::grok::media_response_parser::parse_video_generation_body;
use crate::grok::client::GrokRequestError;
use crate::grok::types::GrokRequest;
use crate::middleware::jwt_auth::JwtUser;
use crate::services::proxy_failover;

const GROK_MEDIA_CREATE_POST_URL: &str = "https://grok.com/rest/media/post/create";
const DEFAULT_VIDEO_MODEL: &str = "grok-3";

#[derive(Debug, Deserialize)]
pub struct GenerateVideoRequest {
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
    Extension(user): Extension<JwtUser>,
    Json(body): Json<GenerateVideoRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let trimmed_image_url = body.image_url.as_deref().map(str::trim).unwrap_or_default();
    let trimmed_prompt = body.prompt.as_deref().map(str::trim).unwrap_or_default();
    if trimmed_image_url.is_empty() && trimmed_prompt.is_empty() {
        return Err(AppError::BadRequest("prompt is required when image_url is empty".into()));
    }

    let requested_model = body.model.clone().unwrap_or_else(|| DEFAULT_VIDEO_MODEL.to_string());
    let plan_id =
        enforce_user_plan_access(&state.db, user.user_id, REQUEST_KIND_VIDEO, &requested_model).await?;
    let usage = {
        let mut usage =
            build_user_usage_context(user.user_id, requested_model.clone(), REQUEST_KIND_VIDEO);
        usage.plan_id = Some(plan_id);
        usage
    };

    let account = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
    let start = std::time::Instant::now();
    match run_video_generation_flow(&state, &account, &body, &requested_model).await {
        Ok((raw_body, prepared)) => {
            state.pool.mark_success().await;
            mark_account_success(&state.db, account.id).await;
            record_usage(
                &state,
                &usage,
                account.id,
                "success",
                duration_to_latency_ms(start.elapsed()),
            )
            .await;

            Ok(Json(build_video_generation_response(raw_body, prepared)))
        }
        Err(GrokRequestError::ProxyFailed(message)) => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(&state, &account, &message).await {
                let (raw_body, prepared) = run_video_generation_flow(&state, &next, &body, &requested_model)
                    .await
                    .map_err(|error| AppError::GrokApi(error.to_string()))?;
                state.pool.mark_success().await;
                mark_account_success(&state.db, next.id).await;
                record_usage(
                    &state,
                    &usage,
                    next.id,
                    "success",
                    duration_to_latency_ms(start.elapsed()),
                )
                .await;
                Ok(Json(build_video_generation_response(raw_body, prepared)))
            } else {
                record_usage(
                    &state,
                    &usage,
                    account.id,
                    "proxy_failed",
                    duration_to_latency_ms(start.elapsed()),
                )
                .await;
                Err(AppError::GrokApi(message))
            }
        }
        Err(GrokRequestError::CfBlocked) if account.proxy_id.is_some() => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(
                &state,
                &account,
                "Proxy received Cloudflare block from upstream.",
            )
            .await
            {
                let (raw_body, prepared) = run_video_generation_flow(&state, &next, &body, &requested_model)
                    .await
                    .map_err(|error| AppError::GrokApi(error.to_string()))?;
                state.pool.mark_success().await;
                mark_account_success(&state.db, next.id).await;
                record_usage(
                    &state,
                    &usage,
                    next.id,
                    "success",
                    duration_to_latency_ms(start.elapsed()),
                )
                .await;
                Ok(Json(build_video_generation_response(raw_body, prepared)))
            } else {
                record_usage(
                    &state,
                    &usage,
                    account.id,
                    "cf_blocked",
                    duration_to_latency_ms(start.elapsed()),
                )
                .await;
                Err(AppError::GrokApi("Cloudflare blocked".into()))
            }
        }
        Err(GrokRequestError::Unauthorized) => {
            if let Some(id) = account.id {
                let _ = crate::db::account_sessions::mark_session_expired(
                    &state.db,
                    id,
                    "Upstream Grok session expired or cookies are invalid.",
                )
                .await;
            }
            record_usage(
                &state,
                &usage,
                account.id,
                "unauthorized",
                duration_to_latency_ms(start.elapsed()),
            )
            .await;
            Err(AppError::GrokApi("Unauthorized".into()))
        }
        Err(error) => {
            record_usage(
                &state,
                &usage,
                account.id,
                crate::api::chat_completions::request_error_status(&error),
                duration_to_latency_ms(start.elapsed()),
            )
            .await;
            Err(AppError::GrokApi(error.to_string()))
        }
    }
}

async fn run_video_generation_flow(
    state: &AppState,
    account: &crate::account::pool::CurrentAccount,
    body: &GenerateVideoRequest,
    requested_model: &str,
) -> Result<(String, PreparedVideoRequest), GrokRequestError> {
    let proxy_ref = account.proxy_url.as_ref();
    let parent_post_id = create_parent_post(state, &account.cookies, proxy_ref, body).await?;
    let prepared = prepare_video_request(body, requested_model.to_string(), parent_post_id);
    let raw_body = state
        .grok
        .send_request(&account.cookies, &prepared.payload, proxy_ref)
        .await?;
    Ok((raw_body, prepared))
}

async fn create_parent_post(
    state: &AppState,
    cookies: &crate::account::types::GrokCookies,
    proxy_url: Option<&String>,
    body: &GenerateVideoRequest,
) -> Result<String, GrokRequestError> {
    let trimmed_image_url = body.image_url.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let trimmed_prompt = body.prompt.as_deref().map(str::trim).filter(|value| !value.is_empty());

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

    let raw_body = state
        .grok
        .send_json_request(GROK_MEDIA_CREATE_POST_URL, cookies, &create_payload, proxy_url)
        .await
        ?;
    let response: MediaCreatePostResponse = serde_json::from_str(&raw_body)
        .map_err(|error| GrokRequestError::Network(format!("Invalid media post response: {error}")))?;
    Ok(response.post.id)
}

fn prepare_video_request(
    body: &GenerateVideoRequest,
    model_name: String,
    parent_post_id: String,
) -> PreparedVideoRequest {
    let trimmed_image_url = body.image_url.as_deref().map(str::trim).filter(|value| !value.is_empty());
    let trimmed_prompt = body.prompt.as_deref().map(str::trim).filter(|value| !value.is_empty()).unwrap_or("");
    let mode = body.mode.clone().unwrap_or_else(|| {
        if trimmed_prompt.is_empty() { "normal".to_string() } else { "custom".to_string() }
    });
    let aspect_ratio = body.aspect_ratio.clone().unwrap_or_else(|| "2:3".to_string());
    let duration_seconds = body.duration_seconds.unwrap_or(6);
    let resolution = body.resolution.clone().unwrap_or_else(|| "480p".to_string());

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
) -> serde_json::Value {
    let parsed = parse_video_generation_body(&raw_body);

    json!({
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
    })
}
