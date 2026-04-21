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
use crate::api::chat_completions::{
    ApiKey, UsageContext, finish_completion, resolve_usage_context, track_api_key_usage,
};
use crate::api::consumer_api_support::finalize_stream_provider_error;
use crate::api::plan_enforcement::{REQUEST_KIND_CHAT, enforce_plan_access};
use crate::api::request_orchestrator::{
    CollectedChatOutput, UNEXPECTED_STREAM_END_MESSAGE, extract_text_content_strict,
    provider_model_slug, resolve_model_route, start_orchestrated_chat_stream,
    validate_reasoning_effort,
};
use crate::error::AppError;
use crate::providers::{ChatMessage, ChatStreamEvent};

#[derive(Debug, Deserialize)]
pub struct ResponsesRequest {
    pub model: Option<String>,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub stream: bool,
    #[serde(default)]
    pub tools: Vec<Value>,
    #[serde(default)]
    pub reasoning: Option<ResponsesReasoningConfig>,
    #[serde(default)]
    pub reasoning_effort: Option<String>,
}

#[derive(Debug, Deserialize, Default)]
pub struct ResponsesReasoningConfig {
    #[serde(default)]
    pub effort: Option<String>,
}

pub async fn create_response(
    State(state): State<AppState>,
    Extension(api_key): Extension<ApiKey>,
    Json(body): Json<ResponsesRequest>,
) -> Result<Response, AppError> {
    validate_responses_request(&body)?;

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

    let reasoning_effort = resolve_reasoning_effort(&body)?;
    let provider_model =
        provider_model_slug(&route.upstream_model_slug, reasoning_effort.as_deref());
    let (system_prompt, messages) =
        normalize_responses_input(body.instructions.as_deref(), &body.input)?;
    let response_id = format!("resp_{}", uuid::Uuid::new_v4().simple());
    let created_at = chrono::Utc::now().timestamp();

    if body.stream {
        return Ok(stream_response(
            state,
            route,
            provider_model,
            response_id,
            created_at,
            system_prompt,
            messages,
            usage_context,
        )
        .await
        .into_response());
    }

    let output = crate::api::request_orchestrator::collect_orchestrated_chat_completion(
        &state,
        &route,
        &provider_model,
        &messages,
        &system_prompt,
        &usage_context,
    )
    .await?;

    Ok(Json(completed_response_json(
        &response_id,
        created_at,
        &route.public_model_slug,
        &body.instructions,
        &output,
    ))
    .into_response())
}

fn validate_responses_request(body: &ResponsesRequest) -> Result<(), AppError> {
    let _declared_tool_count = body.tools.len();
    Ok(())
}

async fn stream_response(
    state: AppState,
    route: crate::api::request_orchestrator::ResolvedModelRoute,
    provider_model: String,
    response_id: String,
    created_at: i64,
    system_prompt: String,
    messages: Vec<ChatMessage>,
    usage_context: UsageContext,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream! {
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
                let mut sequence_number = 0_i64;
                let mut output = CollectedChatOutput::default();
                let mut message_started = false;
                let mut reasoning_started = false;
                let mut message_output_index = 0_usize;
                let mut reasoning_output_index = 0_usize;

                yield Ok(sse_event(
                    "response.created",
                    with_sequence(
                        &mut sequence_number,
                        json!({
                            "type": "response.created",
                            "response": {
                                "id": response_id,
                                "object": "response",
                                "created_at": created_at,
                                "status": "in_progress",
                                "background": false,
                                "error": null,
                                "output": [],
                            }
                        }),
                    ),
                ));
                yield Ok(sse_event(
                    "response.in_progress",
                    with_sequence(
                        &mut sequence_number,
                        json!({
                            "type": "response.in_progress",
                            "response": {
                                "id": response_id,
                                "object": "response",
                                "created_at": created_at,
                                "status": "in_progress",
                                "background": false,
                                "error": null,
                            }
                        }),
                    ),
                ));

                while let Some(event) = rx.recv().await {
                    match event {
                        ChatStreamEvent::Thinking(token) => {
                            if token.is_empty() {
                                continue;
                            }

                            if !reasoning_started {
                                reasoning_started = true;
                                reasoning_output_index = if message_started { 1 } else { 0 };
                                let reasoning_id = reasoning_item_id(&response_id, reasoning_output_index);
                                yield Ok(sse_event(
                                    "response.output_item.added",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.output_item.added",
                                            "output_index": reasoning_output_index,
                                            "item": {
                                                "id": reasoning_id,
                                                "type": "reasoning",
                                                "summary": [],
                                            }
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.reasoning_summary_part.added",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.reasoning_summary_part.added",
                                            "item_id": reasoning_id,
                                            "output_index": reasoning_output_index,
                                            "summary_index": 0,
                                            "part": {
                                                "type": "summary_text",
                                                "text": "",
                                            }
                                        }),
                                    ),
                                ));
                            }

                            output.thinking.push_str(&token);
                            yield Ok(sse_event(
                                "response.reasoning_summary_text.delta",
                                with_sequence(
                                    &mut sequence_number,
                                    json!({
                                        "type": "response.reasoning_summary_text.delta",
                                        "item_id": reasoning_item_id(&response_id, reasoning_output_index),
                                        "output_index": reasoning_output_index,
                                        "summary_index": 0,
                                        "delta": token,
                                    }),
                                ),
                            ));
                        }
                        ChatStreamEvent::Token(token) => {
                            if token.is_empty() {
                                continue;
                            }

                            if !message_started {
                                message_started = true;
                                message_output_index = if reasoning_started { 1 } else { 0 };
                                let message_id = message_item_id(&response_id, message_output_index);
                                yield Ok(sse_event(
                                    "response.output_item.added",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.output_item.added",
                                            "output_index": message_output_index,
                                            "item": {
                                                "id": message_id,
                                                "type": "message",
                                                "content": [],
                                                "role": "assistant",
                                            }
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.content_part.added",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.content_part.added",
                                            "item_id": message_id,
                                            "output_index": message_output_index,
                                            "content_index": 0,
                                            "part": {
                                                "type": "output_text",
                                                "annotations": [],
                                                "logprobs": [],
                                                "text": "",
                                            }
                                        }),
                                    ),
                                ));
                            }

                            output.content.push_str(&token);
                            yield Ok(sse_event(
                                "response.output_text.delta",
                                with_sequence(
                                    &mut sequence_number,
                                    json!({
                                        "type": "response.output_text.delta",
                                        "item_id": message_item_id(&response_id, message_output_index),
                                        "output_index": message_output_index,
                                        "content_index": 0,
                                        "delta": token,
                                        "logprobs": [],
                                    }),
                                ),
                            ));
                        }
                        ChatStreamEvent::Done => {
                            finish_completion(&state, &usage_context, account_id, "success", true, started_at).await;

                            if reasoning_started {
                                let reasoning_id = reasoning_item_id(&response_id, reasoning_output_index);
                                yield Ok(sse_event(
                                    "response.reasoning_summary_text.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.reasoning_summary_text.done",
                                            "item_id": reasoning_id,
                                            "output_index": reasoning_output_index,
                                            "summary_index": 0,
                                            "text": output.thinking,
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.reasoning_summary_part.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.reasoning_summary_part.done",
                                            "item_id": reasoning_id,
                                            "output_index": reasoning_output_index,
                                            "summary_index": 0,
                                            "part": {
                                                "type": "summary_text",
                                                "text": output.thinking,
                                            }
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.output_item.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.output_item.done",
                                            "output_index": reasoning_output_index,
                                            "item": {
                                                "id": reasoning_id,
                                                "type": "reasoning",
                                                "summary": [{
                                                    "type": "summary_text",
                                                    "text": output.thinking,
                                                }],
                                            }
                                        }),
                                    ),
                                ));
                            }

                            if message_started {
                                let message_id = message_item_id(&response_id, message_output_index);
                                yield Ok(sse_event(
                                    "response.output_text.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.output_text.done",
                                            "item_id": message_id,
                                            "output_index": message_output_index,
                                            "content_index": 0,
                                            "text": output.content,
                                            "logprobs": [],
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.content_part.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.content_part.done",
                                            "item_id": message_id,
                                            "output_index": message_output_index,
                                            "content_index": 0,
                                            "part": {
                                                "type": "output_text",
                                                "annotations": [],
                                                "logprobs": [],
                                                "text": output.content,
                                            }
                                        }),
                                    ),
                                ));
                                yield Ok(sse_event(
                                    "response.output_item.done",
                                    with_sequence(
                                        &mut sequence_number,
                                        json!({
                                            "type": "response.output_item.done",
                                            "output_index": message_output_index,
                                            "item": {
                                                "id": message_id,
                                                "type": "message",
                                                "role": "assistant",
                                                "content": [{
                                                    "type": "output_text",
                                                    "annotations": [],
                                                    "logprobs": [],
                                                    "text": output.content,
                                                }],
                                            }
                                        }),
                                    ),
                                ));
                            }

                            yield Ok(sse_event(
                                "response.completed",
                                with_sequence(
                                    &mut sequence_number,
                                    json!({
                                        "type": "response.completed",
                                        "response": {
                                            "id": response_id,
                                            "object": "response",
                                            "created_at": created_at,
                                            "status": "completed",
                                            "background": false,
                                            "error": null,
                                        }
                                    }),
                                ),
                            ));
                            yield Ok(Event::default().data("[DONE]"));
                            return;
                        }
                        ChatStreamEvent::Error(error) => {
                            finalize_stream_provider_error(&state, &usage_context, account_id, &error, started_at).await;
                            yield Ok(sse_event(
                                "response.failed",
                                with_sequence(
                                    &mut sequence_number,
                                    json!({
                                        "type": "response.failed",
                                        "response": {
                                            "id": response_id,
                                            "object": "response",
                                            "created_at": created_at,
                                            "status": "failed",
                                            "background": false,
                                            "error": {
                                                "message": error.to_string(),
                                            },
                                        }
                                    }),
                                ),
                            ));
                            yield Ok(Event::default().data("[DONE]"));
                            return;
                        }
                    }
                }

                finish_completion(&state, &usage_context, account_id, "stream_interrupted", false, started_at).await;
                yield Ok(sse_event(
                    "response.failed",
                    with_sequence(
                        &mut sequence_number,
                        json!({
                            "type": "response.failed",
                            "response": {
                                "id": response_id,
                                "object": "response",
                                "created_at": created_at,
                                "status": "failed",
                                "background": false,
                                "error": {
                                    "message": UNEXPECTED_STREAM_END_MESSAGE,
                                },
                            }
                        }),
                    ),
                ));
                yield Ok(Event::default().data("[DONE]"));
            }
            Err(error) => {
                let mut sequence_number = 0_i64;
                yield Ok(sse_event(
                    "response.failed",
                    with_sequence(
                        &mut sequence_number,
                        json!({
                            "type": "response.failed",
                            "response": {
                                "id": response_id,
                                "object": "response",
                                "created_at": created_at,
                                "status": "failed",
                                "background": false,
                                "error": {
                                    "message": error.to_string(),
                                },
                            }
                        }),
                    ),
                ));
                yield Ok(Event::default().data("[DONE]"));
            }
        }
    };

    Sse::new(stream).keep_alive(KeepAlive::default())
}

fn resolve_reasoning_effort(body: &ResponsesRequest) -> Result<Option<String>, AppError> {
    let nested = body
        .reasoning
        .as_ref()
        .and_then(|reasoning| reasoning.effort.as_deref());
    validate_reasoning_effort(body.reasoning_effort.as_deref().or(nested))
}

fn normalize_responses_input(
    instructions: Option<&str>,
    input: &Value,
) -> Result<(String, Vec<ChatMessage>), AppError> {
    let mut system_parts = Vec::new();
    let mut messages = Vec::new();

    if let Some(instructions) = instructions
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        system_parts.push(instructions.to_string());
    }

    match input {
        Value::Null => {}
        Value::String(text) => push_message("user", text, &mut system_parts, &mut messages),
        Value::Array(items) => {
            for item in items {
                append_input_item(item, &mut system_parts, &mut messages)?;
            }
        }
        Value::Object(_) => append_input_item(input, &mut system_parts, &mut messages)?,
        _ => {
            return Err(AppError::BadRequest(
                "input must be a string, object, or array".into(),
            ));
        }
    }

    if messages.is_empty() {
        return Err(AppError::BadRequest(
            "input must contain at least one text message".into(),
        ));
    }

    Ok((system_parts.join("\n\n"), messages))
}

fn append_input_item(
    item: &Value,
    system_parts: &mut Vec<String>,
    messages: &mut Vec<ChatMessage>,
) -> Result<(), AppError> {
    match item {
        Value::String(text) => {
            push_message("user", text, system_parts, messages);
            Ok(())
        }
        Value::Object(map) => {
            let item_type = map.get("type").and_then(Value::as_str);
            match item_type {
                Some("input_image") | Some("image") | Some("image_url") => {
                    Err(AppError::BadRequest(
                        "image input is not supported on /v1/responses yet".into(),
                    ))
                }
                Some("function_call")
                | Some("function_call_output")
                | Some("computer_call")
                | Some("computer_call_output") => Err(AppError::BadRequest(
                    "tool and function payloads are not supported on /v1/responses yet".into(),
                )),
                Some("input_text") | Some("text") => {
                    let text = map
                        .get("text")
                        .and_then(Value::as_str)
                        .or_else(|| map.get("content").and_then(Value::as_str))
                        .unwrap_or("");
                    push_message("user", text, system_parts, messages);
                    Ok(())
                }
                Some("message") => append_message_object(
                    map.get("role"),
                    map.get("content"),
                    system_parts,
                    messages,
                ),
                Some(other) => Err(AppError::BadRequest(format!(
                    "Unsupported input item type on /v1/responses: {other}"
                ))),
                None if map.get("role").is_some() || map.get("content").is_some() => {
                    append_message_object(
                        map.get("role"),
                        map.get("content"),
                        system_parts,
                        messages,
                    )
                }
                None => Err(AppError::BadRequest(
                    "input object must contain role/content or a supported type".into(),
                )),
            }
        }
        _ => Err(AppError::BadRequest(
            "input items must be strings or objects".into(),
        )),
    }
}

fn append_message_object(
    role: Option<&Value>,
    content: Option<&Value>,
    system_parts: &mut Vec<String>,
    messages: &mut Vec<ChatMessage>,
) -> Result<(), AppError> {
    let role = role.and_then(Value::as_str).unwrap_or("user");
    if matches!(role, "tool" | "function") {
        return Err(AppError::BadRequest(
            "tool and function message roles are not supported on /v1/responses yet".into(),
        ));
    }
    let content = extract_text_content_strict(content.unwrap_or(&Value::Null), "/v1/responses")?;
    push_message(role, &content, system_parts, messages);
    Ok(())
}

fn push_message(
    role: &str,
    text: &str,
    system_parts: &mut Vec<String>,
    messages: &mut Vec<ChatMessage>,
) {
    let text = text.trim();
    if text.is_empty() {
        return;
    }

    if role == "system" {
        system_parts.push(text.to_string());
        return;
    }

    messages.push(ChatMessage {
        role: role.to_string(),
        content: text.to_string(),
    });
}

fn completed_response_json(
    response_id: &str,
    created_at: i64,
    model: &str,
    instructions: &Option<String>,
    output: &CollectedChatOutput,
) -> Value {
    let mut items = Vec::new();

    if !output.thinking.is_empty() {
        items.push(json!({
            "id": reasoning_item_id(response_id, 0),
            "type": "reasoning",
            "summary": [{
                "type": "summary_text",
                "text": output.thinking,
            }],
        }));
    }

    let message_index = if output.thinking.is_empty() { 0 } else { 1 };
    items.push(json!({
        "id": message_item_id(response_id, message_index),
        "type": "message",
        "status": "completed",
        "role": "assistant",
        "content": [{
            "type": "output_text",
            "annotations": [],
            "logprobs": [],
            "text": output.content,
        }],
    }));

    json!({
        "id": response_id,
        "object": "response",
        "created_at": created_at,
        "status": "completed",
        "background": false,
        "error": null,
        "instructions": instructions,
        "model": model,
        "output": items,
        "output_text": output.content,
        "parallel_tool_calls": false,
        "usage": {
            "input_tokens": 0,
            "output_tokens": 0,
            "total_tokens": 0,
        },
    })
}

fn with_sequence(sequence_number: &mut i64, payload: Value) -> Value {
    *sequence_number += 1;

    match payload {
        Value::Object(mut map) => {
            map.insert("sequence_number".to_string(), json!(*sequence_number));
            Value::Object(map)
        }
        other => other,
    }
}

fn sse_event(event_name: &str, payload: Value) -> Event {
    Event::default()
        .event(event_name)
        .data(serde_json::to_string(&payload).unwrap())
}

fn message_item_id(response_id: &str, index: usize) -> String {
    format!("msg_{response_id}_{index}")
}

fn reasoning_item_id(response_id: &str, index: usize) -> String {
    format!("rs_{response_id}_{index}")
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    #[test]
    fn accepts_tools_array_for_client_compatibility() {
        super::validate_responses_request(&super::ResponsesRequest {
            model: Some("gpt-5.1".to_string()),
            input: json!("hello"),
            instructions: None,
            stream: false,
            tools: vec![json!({ "type": "function", "name": "lookup_weather" })],
            reasoning: None,
            reasoning_effort: None,
        })
        .expect("tools array should be accepted and ignored for client compatibility");
    }

    #[test]
    fn normalizes_string_input_into_user_message() {
        let (system_prompt, messages) =
            super::normalize_responses_input(Some("be precise"), &json!("hello")).unwrap();

        assert_eq!(system_prompt, "be precise");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "hello");
    }

    #[test]
    fn rejects_image_input_parts() {
        let error = super::normalize_responses_input(
            None,
            &json!([{
                "role": "user",
                "content": [{
                    "type": "input_image",
                    "image_url": "https://example.com/image.png"
                }]
            }]),
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("image input is not supported on /v1/responses yet")
        );
    }

    #[test]
    fn accepts_input_text_content_arrays() {
        let (_, messages) = super::normalize_responses_input(
            None,
            &json!([{
                "role": "user",
                "content": [
                    { "type": "input_text", "text": "hello" },
                    { "type": "input_text", "text": "world" }
                ]
            }]),
        )
        .unwrap();

        assert_eq!(messages[0].content, "hello\nworld");
    }

    #[test]
    fn rejects_tool_role_messages() {
        let error = super::normalize_responses_input(
            None,
            &json!([{
                "role": "tool",
                "content": "not allowed"
            }]),
        )
        .unwrap_err();

        assert!(
            error
                .to_string()
                .contains("tool and function message roles are not supported on /v1/responses yet")
        );
    }

    #[test]
    fn completed_response_uses_public_model_slug() {
        let payload = super::completed_response_json(
            "resp_123",
            1_713_000_000,
            "gpt-5.1",
            &Some("be precise".to_string()),
            &super::CollectedChatOutput {
                content: "final answer".to_string(),
                thinking: String::new(),
            },
        );

        assert_eq!(payload["model"], "gpt-5.1");
        assert_eq!(payload["output_text"], "final answer");
        assert_eq!(payload["output"][0]["type"], "message");
        assert_eq!(payload["output"][0]["content"][0]["text"], "final answer");
    }
}
