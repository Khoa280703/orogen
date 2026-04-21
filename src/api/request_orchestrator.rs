use std::sync::Arc;
use std::time::Instant;

use serde_json::Value;
use tokio::sync::mpsc;

use crate::AppState;
use crate::api::chat_completions::{ChatCompletionMessage, UsageContext, finish_completion};
use crate::api::consumer_api_support::{
    finalize_stream_provider_error, start_chat_stream_with_retry,
};
use crate::db::public_models::PublicModelWithRoute;
use crate::error::AppError;
use crate::providers::ProviderError;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::{ChatMessage as ProviderChatMessage, ChatStreamEvent};

pub const UNEXPECTED_STREAM_END_MESSAGE: &str = "Upstream stream ended unexpectedly";

#[derive(Debug, Clone)]
pub struct ResolvedModelRoute {
    pub public_model_slug: String,
    pub provider_slug: String,
    pub upstream_model_slug: String,
}

#[derive(Debug, Clone, Default)]
pub struct CollectedChatOutput {
    pub content: String,
    pub thinking: String,
}

pub async fn resolve_model_route(
    state: &AppState,
    requested_model: Option<&str>,
) -> Result<ResolvedModelRoute, AppError> {
    let model_slug = requested_model
        .filter(|value| !value.trim().is_empty())
        .unwrap_or(state.config.default_model.as_str());

    let resolved =
        crate::db::public_model_routes::get_public_model_route_by_slug(&state.db, model_slug)
            .await
            .ok_or_else(|| {
                AppError::BadRequest(format!("Unknown or inactive model: {model_slug}"))
            })?;

    if !chat_provider_is_registered(state, &resolved.provider_slug) {
        return Err(AppError::Internal(format!(
            "Chat provider not registered: {}",
            resolved.provider_slug
        )));
    }

    Ok(ResolvedModelRoute {
        public_model_slug: resolved.public_model_slug,
        provider_slug: resolved.provider_slug,
        upstream_model_slug: resolved.upstream_model_slug,
    })
}

pub fn provider_model_slug(
    base_provider_model_slug: &str,
    reasoning_effort: Option<&str>,
) -> String {
    if let Some(effort) = reasoning_effort {
        if !base_provider_model_slug.ends_with(&format!("-{effort}")) {
            return format!("{base_provider_model_slug}-{effort}");
        }
    }
    base_provider_model_slug.to_string()
}

pub fn validate_reasoning_effort(
    reasoning_effort: Option<&str>,
) -> Result<Option<String>, AppError> {
    let Some(effort) = reasoning_effort
        .map(str::trim)
        .filter(|value| !value.is_empty())
    else {
        return Ok(None);
    };

    match effort {
        "none" | "low" | "medium" | "high" | "xhigh" => Ok(Some(effort.to_string())),
        _ => Err(AppError::BadRequest(format!(
            "Unsupported reasoning effort: {effort}"
        ))),
    }
}

pub async fn list_supported_public_models(
    state: &AppState,
    plan_id: Option<i32>,
) -> Vec<PublicModelWithRoute> {
    let models = if let Some(plan_id) = plan_id {
        crate::db::public_models::list_public_models_for_plan(&state.db, plan_id).await
    } else {
        crate::db::public_models::list_public_models(&state.db).await
    };

    models
        .into_iter()
        .filter(|model| provider_is_registered(state, &model.provider_slug))
        .collect()
}

pub fn provider_is_registered(state: &AppState, provider_slug: &str) -> bool {
    state.providers.chat_provider(provider_slug).is_some()
        || state.providers.image_provider(provider_slug).is_some()
}

pub fn chat_provider_is_registered(state: &AppState, provider_slug: &str) -> bool {
    state.providers.chat_provider(provider_slug).is_some()
}

fn is_text_part_type(item_type: &str) -> bool {
    matches!(item_type, "text" | "input_text" | "output_text")
}

pub fn extract_text_content(content: &Value) -> String {
    match content {
        Value::String(text) => text.trim().to_string(),
        Value::Array(items) => items
            .iter()
            .filter_map(|item| match item {
                Value::String(text) => Some(text.trim().to_string()),
                Value::Object(map) => {
                    let item_type = map.get("type").and_then(Value::as_str).unwrap_or("text");
                    if !is_text_part_type(item_type) {
                        return None;
                    }
                    map.get("text")
                        .and_then(Value::as_str)
                        .map(str::trim)
                        .filter(|text| !text.is_empty())
                        .map(str::to_string)
                }
                _ => None,
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(map) => {
            let item_type = map.get("type").and_then(Value::as_str).unwrap_or("text");
            if !is_text_part_type(item_type) {
                return String::new();
            }

            map.get("text")
                .and_then(Value::as_str)
                .or_else(|| map.get("content").and_then(Value::as_str))
                .map(str::trim)
                .filter(|text| !text.is_empty())
                .map(str::to_string)
                .unwrap_or_default()
        }
        _ => String::new(),
    }
}

pub fn extract_text_content_strict(content: &Value, endpoint: &str) -> Result<String, AppError> {
    match content {
        Value::Null => Ok(String::new()),
        Value::String(text) => Ok(text.trim().to_string()),
        Value::Array(items) => {
            let mut parts = Vec::new();
            for item in items {
                match item {
                    Value::String(text) => {
                        let text = text.trim();
                        if !text.is_empty() {
                            parts.push(text.to_string());
                        }
                    }
                    Value::Object(map) => {
                        let item_type = map.get("type").and_then(Value::as_str).unwrap_or("text");
                        match item_type {
                            "input_image" | "image" | "image_url" => {
                                return Err(AppError::BadRequest(format!(
                                    "image input is not supported on {endpoint} yet"
                                )));
                            }
                            "function_call"
                            | "function_call_output"
                            | "computer_call"
                            | "computer_call_output" => {
                                return Err(AppError::BadRequest(format!(
                                    "tool and function payloads are not supported on {endpoint} yet"
                                )));
                            }
                            other if !is_text_part_type(other) => {
                                return Err(AppError::BadRequest(format!(
                                    "Unsupported content part type on {endpoint}: {other}"
                                )));
                            }
                            _ => {
                                let text = map
                                    .get("text")
                                    .and_then(Value::as_str)
                                    .or_else(|| map.get("content").and_then(Value::as_str))
                                    .map(str::trim)
                                    .filter(|text| !text.is_empty())
                                    .unwrap_or("");
                                if !text.is_empty() {
                                    parts.push(text.to_string());
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(AppError::BadRequest(format!(
                            "Unsupported content payload on {endpoint}"
                        )));
                    }
                }
            }
            Ok(parts.join("\n"))
        }
        Value::Object(map) => {
            let item_type = map.get("type").and_then(Value::as_str);
            match item_type {
                Some("input_image") | Some("image") | Some("image_url") => Err(
                    AppError::BadRequest(format!("image input is not supported on {endpoint} yet")),
                ),
                Some("function_call")
                | Some("function_call_output")
                | Some("computer_call")
                | Some("computer_call_output") => Err(AppError::BadRequest(format!(
                    "tool and function payloads are not supported on {endpoint} yet"
                ))),
                Some(other) if !is_text_part_type(other) => Err(AppError::BadRequest(format!(
                    "Unsupported content part type on {endpoint}: {other}"
                ))),
                _ => Ok(extract_text_content(content)),
            }
        }
        _ => Err(AppError::BadRequest(format!(
            "Unsupported content payload on {endpoint}"
        ))),
    }
}

pub fn normalize_chat_completion_messages(
    messages: &[ChatCompletionMessage],
) -> Result<(String, Vec<ProviderChatMessage>), AppError> {
    let mut system_parts = Vec::new();
    let mut chat_messages = Vec::new();

    for message in messages {
        if matches!(message.role.as_str(), "tool" | "function")
            || !message.tool_calls.is_empty()
            || message.function_call.is_some()
        {
            return Err(AppError::BadRequest(
                "tool and function calling payloads are no longer supported on /v1/chat/completions"
                    .into(),
            ));
        }

        let content = extract_text_content_strict(&message.content, "/v1/chat/completions")?;
        if message.role == "system" {
            if !content.is_empty() {
                system_parts.push(content);
            }
            continue;
        }

        if content.is_empty() {
            continue;
        }

        chat_messages.push(ProviderChatMessage {
            role: message.role.clone(),
            content,
        });
    }

    if chat_messages.is_empty() {
        return Err(AppError::BadRequest(
            "at least one non-system message with text content is required".into(),
        ));
    }

    Ok((system_parts.join("\n\n"), chat_messages))
}

pub async fn chat_provider_for_route(
    state: &AppState,
    route: &ResolvedModelRoute,
) -> Result<Arc<dyn ChatProvider>, AppError> {
    state
        .providers
        .get_chat_provider(&route.provider_slug)
        .ok_or_else(|| {
            AppError::Internal(format!(
                "Chat provider not registered: {}",
                route.provider_slug
            ))
        })
}

pub async fn start_orchestrated_chat_stream(
    state: &AppState,
    route: &ResolvedModelRoute,
    provider_model: &str,
    messages: &[ProviderChatMessage],
    system_prompt: &str,
    usage_context: &UsageContext,
) -> Result<
    (
        mpsc::UnboundedReceiver<ChatStreamEvent>,
        Option<i32>,
        String,
        Arc<dyn ChatProvider>,
    ),
    AppError,
> {
    let provider = chat_provider_for_route(state, route).await?;
    let (rx, account_id, account_name) = start_chat_stream_with_retry(
        state,
        provider.clone(),
        &route.provider_slug,
        provider_model,
        messages,
        system_prompt,
        usage_context,
    )
    .await?;

    Ok((rx, account_id, account_name, provider))
}

pub async fn collect_orchestrated_chat_completion(
    state: &AppState,
    route: &ResolvedModelRoute,
    provider_model: &str,
    messages: &[ProviderChatMessage],
    system_prompt: &str,
    usage_context: &UsageContext,
) -> Result<CollectedChatOutput, AppError> {
    let (mut rx, account_id, _account_name, _provider) = start_orchestrated_chat_stream(
        state,
        route,
        provider_model,
        messages,
        system_prompt,
        usage_context,
    )
    .await?;
    let started_at = Instant::now();
    let mut output = CollectedChatOutput::default();

    while let Some(event) = rx.recv().await {
        match event {
            ChatStreamEvent::Token(token) => output.content.push_str(&token),
            ChatStreamEvent::Thinking(token) => output.thinking.push_str(&token),
            ChatStreamEvent::Done => {
                finish_completion(
                    state,
                    usage_context,
                    account_id,
                    "success",
                    true,
                    started_at,
                )
                .await;
                return Ok(output);
            }
            ChatStreamEvent::Error(error) => {
                finalize_stream_provider_error(
                    state,
                    usage_context,
                    account_id,
                    &error,
                    started_at,
                )
                .await;
                return Err(provider_error_to_app_error(error));
            }
        }
    }

    finish_completion(
        state,
        usage_context,
        account_id,
        "stream_interrupted",
        false,
        started_at,
    )
    .await;
    Err(AppError::GrokApi(UNEXPECTED_STREAM_END_MESSAGE.into()))
}

fn provider_error_to_app_error(error: ProviderError) -> AppError {
    error.into()
}
