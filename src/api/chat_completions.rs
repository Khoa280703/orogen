use std::convert::Infallible;

use async_stream::stream;
use axum::Json;
use axum::extract::{Extension, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use futures::stream::Stream;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::AppState;
use crate::api::consumer_api_support::{finalize_stream_provider_error, mark_account_success};
use crate::api::plan_enforcement::{REQUEST_KIND_CHAT, enforce_plan_access};
use crate::api::request_orchestrator::{
    UNEXPECTED_STREAM_END_MESSAGE, collect_orchestrated_chat_completion,
    normalize_chat_completion_messages, resolve_model_route, start_orchestrated_chat_stream,
};
use crate::error::AppError;
use crate::grok::client::GrokRequestError;
use crate::providers::ChatStreamEvent;

#[derive(Debug, Clone, Default)]
pub struct ApiKey(pub String);

#[derive(Debug, Clone)]
pub(crate) struct UsageContext {
    pub(crate) api_key_id: Option<i32>,
    pub(crate) user_id: Option<i32>,
    pub(crate) plan_id: Option<i32>,
    pub(crate) provider_slug: String,
    pub(crate) model: String,
    pub(crate) request_kind: &'static str,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionRequest {
    pub model: Option<String>,
    pub messages: Vec<ChatCompletionMessage>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub tools: Vec<Value>,
}

#[derive(Debug, Deserialize)]
pub struct ChatCompletionMessage {
    pub role: String,
    #[serde(default)]
    pub content: Value,
    #[serde(default)]
    pub tool_calls: Vec<Value>,
    #[serde(default)]
    pub function_call: Option<Value>,
}

pub async fn chat_completions(
    State(state): State<AppState>,
    Extension(api_key): Extension<ApiKey>,
    Json(body): Json<ChatCompletionRequest>,
) -> Result<Response, AppError> {
    if body.messages.is_empty() {
        return Err(AppError::BadRequest("messages is required".into()));
    }
    if !body.tools.is_empty() {
        return Err(AppError::BadRequest(
            "tools are no longer supported on /v1/chat/completions".into(),
        ));
    }

    track_api_key_usage(&state, &api_key).await;

    let route = resolve_model_route(&state, body.model.as_deref()).await?;
    let usage_context = resolve_usage_context(
        &state,
        &api_key,
        &route.provider_slug,
        &route.public_model_slug,
        REQUEST_KIND_CHAT,
    )
    .await?;

    enforce_plan_access(
        &state.db,
        usage_context.user_id,
        usage_context.api_key_id,
        usage_context.plan_id,
        usage_context.request_kind,
        &route.public_model_slug,
    )
    .await?;

    let (system_prompt, messages) = normalize_chat_completion_messages(&body.messages)?;
    let completion_id = format!("chatcmpl-{}", &uuid::Uuid::new_v4().to_string()[..12]);

    if body.stream {
        return Ok(stream_response(
            state,
            route,
            completion_id,
            system_prompt,
            messages,
            usage_context,
        )
        .await
        .into_response());
    }

    let output = collect_orchestrated_chat_completion(
        &state,
        &route,
        &route.upstream_model_slug,
        &messages,
        &system_prompt,
        &usage_context,
    )
    .await?;

    Ok(Json(json!({
        "id": completion_id,
        "object": "chat.completion",
        "created": chrono::Utc::now().timestamp(),
        "model": route.public_model_slug,
        "choices": [{
            "index": 0,
            "message": {
                "role": "assistant",
                "content": output.content,
            },
            "finish_reason": "stop",
        }],
        "usage": {
            "prompt_tokens": 0,
            "completion_tokens": 0,
            "total_tokens": 0,
        },
    }))
    .into_response())
}

async fn stream_response(
    state: AppState,
    route: crate::api::request_orchestrator::ResolvedModelRoute,
    completion_id: String,
    system_prompt: String,
    messages: Vec<crate::providers::ChatMessage>,
    usage_context: UsageContext,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
        let model = route.public_model_slug.clone();
        let provider_model = route.upstream_model_slug.clone();
        match start_orchestrated_chat_stream(
            &state,
            &route,
            &provider_model,
            &messages,
            &system_prompt,
            &usage_context,
        )
        .await {
            Ok((mut rx, account_id, _account_name, _provider)) => {
                let started_at = std::time::Instant::now();
                yield Ok(Event::default().data(serde_json::to_string(&role_chunk(&completion_id, &model)).unwrap()));

                while let Some(event) = rx.recv().await {
                    match event {
                        ChatStreamEvent::Token(token) => {
                            if token.is_empty() {
                                continue;
                            }
                            yield Ok(Event::default().data(serde_json::to_string(&content_chunk(&completion_id, &model, &token)).unwrap()));
                        }
                        ChatStreamEvent::Thinking(_) => {}
                        ChatStreamEvent::Done => {
                            finish_completion(&state, &usage_context, account_id, "success", true, started_at).await;
                            yield Ok(Event::default().data(serde_json::to_string(&finish_chunk(&completion_id, &model, "stop")).unwrap()));
                            yield Ok(Event::default().data("[DONE]".to_string()));
                            return;
                        }
                        ChatStreamEvent::Error(error) => {
                            finalize_stream_provider_error(&state, &usage_context, account_id, &error, started_at).await;
                            yield Ok(Event::default().data(serde_json::to_string(&json!({
                                "error": { "message": error.to_string() }
                            })).unwrap()));
                            return;
                        }
                    }
                }

                finish_completion(&state, &usage_context, account_id, "stream_interrupted", false, started_at).await;
                yield Ok(Event::default().data(serde_json::to_string(&json!({
                    "error": { "message": UNEXPECTED_STREAM_END_MESSAGE }
                })).unwrap()));
            }
            Err(error) => {
                yield Ok(Event::default().data(serde_json::to_string(&json!({
                    "error": { "message": error.to_string() }
                })).unwrap()));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

pub(crate) async fn finish_completion(
    state: &AppState,
    usage_context: &UsageContext,
    account_id: Option<i32>,
    status: &str,
    success: bool,
    started_at: std::time::Instant,
) {
    if success {
        mark_account_success(&state.db, account_id).await;
        touch_api_key_last_used(state, usage_context).await;
    } else {
        if let Some(id) = account_id {
            let _ = crate::db::accounts::update_health_counts(&state.db, id, false).await;
        }
    }

    record_usage(
        state,
        usage_context,
        account_id,
        status,
        duration_to_latency_ms(started_at.elapsed()),
    )
    .await;
}

fn role_chunk(completion_id: &str, model: &str) -> Value {
    json!({
        "id": completion_id,
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "delta": { "role": "assistant" },
            "finish_reason": Value::Null,
        }],
    })
}

fn content_chunk(completion_id: &str, model: &str, token: &str) -> Value {
    json!({
        "id": completion_id,
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "delta": { "content": token },
            "finish_reason": Value::Null,
        }],
    })
}

fn finish_chunk(completion_id: &str, model: &str, finish_reason: &str) -> Value {
    json!({
        "id": completion_id,
        "object": "chat.completion.chunk",
        "created": chrono::Utc::now().timestamp(),
        "model": model,
        "choices": [{
            "index": 0,
            "delta": {},
            "finish_reason": finish_reason,
        }],
    })
}

pub(crate) async fn track_api_key_usage(state: &AppState, api_key: &ApiKey) {
    if api_key.0.is_empty() {
        return;
    }

    let mut counts = state.key_request_counts.write().await;
    *counts.entry(api_key.0.clone()).or_insert(0) += 1;
}

pub(crate) async fn resolve_usage_context(
    state: &AppState,
    api_key: &ApiKey,
    provider_slug: &str,
    model: &str,
    request_kind: &'static str,
) -> Result<UsageContext, AppError> {
    if api_key.0.is_empty() {
        return Ok(UsageContext {
            api_key_id: None,
            user_id: None,
            plan_id: None,
            provider_slug: provider_slug.to_string(),
            model: model.to_string(),
            request_kind,
        });
    }

    let key = crate::db::api_keys::get_key_by_value(&state.db, &api_key.0)
        .await
        .map_err(|error| AppError::Internal(format!("Failed to resolve API key: {error}")))?;

    Ok(UsageContext {
        api_key_id: key.as_ref().map(|row| row.id),
        user_id: key.as_ref().and_then(|row| row.user_id),
        plan_id: key.and_then(|row| row.plan_id),
        provider_slug: provider_slug.to_string(),
        model: model.to_string(),
        request_kind,
    })
}

pub(crate) async fn record_usage(
    state: &AppState,
    usage_context: &UsageContext,
    account_id: Option<i32>,
    status: &str,
    latency_ms: i32,
) {
    if let Err(error) = crate::db::usage_logs::log_request(
        &state.db,
        usage_context.api_key_id,
        usage_context.user_id,
        account_id,
        Some(usage_context.provider_slug.as_str()),
        Some(usage_context.model.as_str()),
        Some(usage_context.request_kind),
        status,
        latency_ms,
    )
    .await
    {
        tracing::error!(
            api_key_id = usage_context.api_key_id,
            account_id,
            status,
            error = %error,
            "Failed to persist usage log"
        );
    }
}

pub(crate) async fn touch_api_key_last_used(state: &AppState, usage_context: &UsageContext) {
    if let Some(api_key_id) = usage_context.api_key_id {
        if let Err(error) = crate::db::api_keys::touch_last_used(&state.db, api_key_id).await {
            tracing::warn!(api_key_id, error = %error, "Failed to update api key last_used_at");
        }
    }
}

pub(crate) fn duration_to_latency_ms(duration: std::time::Duration) -> i32 {
    duration.as_millis().min(i32::MAX as u128) as i32
}

pub(crate) fn request_error_status(error: &GrokRequestError) -> &'static str {
    match error {
        GrokRequestError::RateLimited => "rate_limited",
        GrokRequestError::Unauthorized => "unauthorized",
        GrokRequestError::CfBlocked => "cf_blocked",
        GrokRequestError::ProxyFailed(_) => "proxy_failed",
        _ => "error",
    }
}

#[cfg(test)]
mod tests {
    use crate::api::request_orchestrator::{
        extract_text_content, normalize_chat_completion_messages,
    };
    use serde_json::json;

    #[test]
    fn extracts_text_from_openai_content_array() {
        let content = json!([
            { "type": "text", "text": "hello" },
            { "type": "text", "text": "world" }
        ]);

        assert_eq!(extract_text_content(&content), "hello\nworld");
    }

    #[test]
    fn rejects_legacy_tool_call_payloads() {
        let parsed: super::ChatCompletionRequest = serde_json::from_value(json!({
            "model": "grok-3",
            "messages": [{
                "role": "assistant",
                "content": null,
                "tool_calls": [{
                    "id": "call_1",
                    "type": "function",
                    "function": {
                        "name": "lookup_weather",
                        "arguments": "{}"
                    }
                }]
            }]
        }))
        .unwrap();

        let error = normalize_chat_completion_messages(&parsed.messages).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("tool and function calling payloads")
        );
    }

    #[test]
    fn rejects_image_content_parts() {
        let parsed: super::ChatCompletionRequest = serde_json::from_value(json!({
            "model": "grok-3",
            "messages": [{
                "role": "user",
                "content": [{
                    "type": "image_url",
                    "image_url": { "url": "https://example.com/image.png" }
                }]
            }]
        }))
        .unwrap();

        let error = normalize_chat_completion_messages(&parsed.messages).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("image input is not supported on /v1/chat/completions yet")
        );
    }
}
