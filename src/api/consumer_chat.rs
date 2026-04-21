use std::convert::Infallible;

use async_stream::stream;
use axum::Json;
use axum::extract::{Extension, Path, Query, State};
use axum::response::sse::{Event, KeepAlive, Sse};
use futures::stream::Stream;
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::api::chat_completions::{duration_to_latency_ms, record_usage};
use crate::api::consumer_api_support::{
    build_user_usage_context, finalize_stream_provider_error, mark_account_success,
    start_chat_stream_with_retry,
};
use crate::api::plan_enforcement::{REQUEST_KIND_CHAT, enforce_user_plan_access};
use crate::db::{conversations, messages};
use crate::error::AppError;
use crate::middleware::jwt_auth::JwtUser;
use crate::providers::{ChatMessage, ChatStreamEvent};

const DEFAULT_CHAT_MODEL: &str = "grok-3";

struct ResolvedChatModel {
    slug: String,
    provider_slug: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateConversationRequest {
    pub title: Option<String>,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SendMessageRequest {
    pub content: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    #[serde(default = "default_limit")]
    pub limit: i64,
    #[serde(default)]
    pub offset: i64,
}

#[derive(Debug, Serialize)]
pub struct ConversationResponse {
    pub conversation: conversations::Conversation,
    pub messages: Vec<messages::Message>,
}

pub async fn create_conversation(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Json(body): Json<CreateConversationRequest>,
) -> Result<Json<conversations::Conversation>, AppError> {
    let model = body.model.unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string());
    enforce_user_plan_access(&state.db, user.user_id, REQUEST_KIND_CHAT, &model).await?;
    let conversation = conversations::create_conversation(
        &state.db,
        user.user_id,
        body.title.as_deref(),
        Some(&model),
    )
    .await
    .map_err(db_error)?;
    Ok(Json(conversation))
}

pub async fn list_conversations(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<conversations::ConversationListItem>>, AppError> {
    let items = conversations::list_conversations(
        &state.db,
        user.user_id,
        query.limit.clamp(1, 100),
        query.offset.max(0),
    )
    .await
    .map_err(db_error)?;
    Ok(Json(items))
}

pub async fn get_conversation(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Path(id): Path<i32>,
) -> Result<Json<ConversationResponse>, AppError> {
    let conversation = require_conversation(&state, user.user_id, id).await?;
    let items = messages::list_messages(&state.db, id)
        .await
        .map_err(db_error)?;
    Ok(Json(ConversationResponse {
        conversation,
        messages: items,
    }))
}

pub async fn delete_conversation(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Path(id): Path<i32>,
) -> Result<axum::http::StatusCode, AppError> {
    if !conversations::delete_conversation(&state.db, id, user.user_id)
        .await
        .map_err(db_error)?
    {
        return Err(AppError::Unauthorized);
    }
    Ok(axum::http::StatusCode::NO_CONTENT)
}

pub async fn send_message(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Path(id): Path<i32>,
    Json(body): Json<SendMessageRequest>,
) -> Result<Sse<impl Stream<Item = Result<Event, Infallible>>>, AppError> {
    let conversation = require_conversation(&state, user.user_id, id).await?;
    let content = body.content.trim().to_string();
    if content.is_empty() {
        return Err(AppError::GrokApi("content is required".into()));
    }

    let requested_model = body
        .model
        .or(conversation.model_slug.clone())
        .unwrap_or_else(|| DEFAULT_CHAT_MODEL.to_string());
    let resolved_model =
        if let Some(model) = select_active_chat_model(&state.db, &requested_model).await {
            model
        } else if let Some(model) = select_active_chat_model(&state.db, DEFAULT_CHAT_MODEL).await {
            model
        } else {
            return Err(AppError::Internal(
                "No active chat model is available".into(),
            ));
        };
    let model = resolved_model.slug.clone();
    let provider_slug = resolved_model.provider_slug.clone();
    let plan_id =
        enforce_user_plan_access(&state.db, user.user_id, REQUEST_KIND_CHAT, &model).await?;

    if messages::count_messages(&state.db, id)
        .await
        .map_err(db_error)?
        == 0
    {
        let title = content.chars().take(80).collect::<String>();
        conversations::update_title(&state.db, id, user.user_id, &title)
            .await
            .map_err(db_error)?;
    }
    conversations::update_model_slug(&state.db, id, user.user_id, &model)
        .await
        .map_err(db_error)?;
    messages::create_message(
        &state.db,
        id,
        "user",
        &content,
        Some(&model),
        Some(&provider_slug),
        0,
    )
    .await
    .map_err(db_error)?;
    conversations::touch_conversation(&state.db, id)
        .await
        .map_err(db_error)?;

    let history = messages::list_messages(&state.db, id)
        .await
        .map_err(db_error)?
        .into_iter()
        .map(|item| ChatMessage {
            role: item.role,
            content: item.content,
        })
        .collect::<Vec<_>>();

    let provider = state
        .providers
        .chat_provider(&provider_slug)
        .ok_or_else(|| {
            AppError::Internal(format!("Chat provider not registered: {provider_slug}"))
        })?;
    let usage = {
        let mut usage = build_user_usage_context(
            user.user_id,
            provider_slug.clone(),
            model.clone(),
            REQUEST_KIND_CHAT,
        );
        usage.plan_id = Some(plan_id);
        usage
    };
    let (mut rx, account_id, _account_name) = start_chat_stream_with_retry(
        &state,
        provider,
        &provider_slug,
        &usage.model,
        &history,
        "",
        &usage,
    )
    .await?;
    let stream_state = state.clone();

    let event_stream = stream! {
        let started = std::time::Instant::now();
        let mut assistant = String::new();

        while let Some(event) = rx.recv().await {
            match event {
                ChatStreamEvent::Token(token) => {
                    assistant.push_str(&token);
                    yield Ok(Event::default().event("token").data(json!({ "content": token }).to_string()));
                }
                ChatStreamEvent::Thinking(thinking) => {
                    yield Ok(Event::default().event("thinking").data(json!({ "content": thinking }).to_string()));
                }
                ChatStreamEvent::Done => {
                    finish_chat(
                        &stream_state,
                        &usage,
                        id,
                        account_id,
                        &assistant,
                        &model,
                        &provider_slug,
                        "success",
                        true,
                        started,
                    ).await;
                    yield Ok(Event::default().event("done").data(json!({}).to_string()));
                    return;
                }
                ChatStreamEvent::Error(error) => {
                    finish_chat_with_provider_error(
                        &stream_state,
                        &usage,
                        id,
                        account_id,
                        &assistant,
                        &model,
                        &provider_slug,
                        &error,
                        started,
                    ).await;
                    yield Ok(Event::default().event("error").data(json!({ "message": error.to_string() }).to_string()));
                    return;
                }
            }
        }

        finish_chat(
            &stream_state,
            &usage,
            id,
            account_id,
            &assistant,
            &model,
            &provider_slug,
            "stream_interrupted",
            false,
            started,
        ).await;
        yield Ok(Event::default().event("error").data(json!({ "message": "Upstream stream ended unexpectedly" }).to_string()));
    };

    Ok(Sse::new(event_stream).keep_alive(KeepAlive::default()))
}

async fn require_conversation(
    state: &AppState,
    user_id: i32,
    id: i32,
) -> Result<conversations::Conversation, AppError> {
    conversations::get_conversation(&state.db, id, user_id)
        .await
        .map_err(db_error)?
        .ok_or(AppError::Unauthorized)
}

async fn finish_chat(
    state: &AppState,
    usage: &crate::api::chat_completions::UsageContext,
    conversation_id: i32,
    account_id: Option<i32>,
    assistant: &str,
    model_slug: &str,
    provider_slug: &str,
    status: &str,
    success: bool,
    started: std::time::Instant,
) {
    if !assistant.trim().is_empty() {
        let _ = messages::create_message(
            &state.db,
            conversation_id,
            "assistant",
            assistant,
            Some(model_slug),
            Some(provider_slug),
            0,
        )
        .await;
        let _ = conversations::touch_conversation(&state.db, conversation_id).await;
    }
    if success {
        mark_account_success(&state.db, account_id).await;
    } else {
        if let Some(id) = account_id {
            let _ = crate::db::accounts::update_health_counts(&state.db, id, false).await;
        }
    }
    record_usage(
        state,
        usage,
        account_id,
        status,
        duration_to_latency_ms(started.elapsed()),
    )
    .await;
}

async fn finish_chat_with_provider_error(
    state: &AppState,
    usage: &crate::api::chat_completions::UsageContext,
    conversation_id: i32,
    account_id: Option<i32>,
    assistant: &str,
    model_slug: &str,
    provider_slug: &str,
    error: &crate::providers::types::ProviderError,
    started: std::time::Instant,
) {
    if !assistant.trim().is_empty() {
        let _ = messages::create_message(
            &state.db,
            conversation_id,
            "assistant",
            assistant,
            Some(model_slug),
            Some(provider_slug),
            0,
        )
        .await;
        let _ = conversations::touch_conversation(&state.db, conversation_id).await;
    }

    finalize_stream_provider_error(state, usage, account_id, error, started).await;
}

fn default_limit() -> i64 {
    20
}
fn db_error(error: sqlx::Error) -> AppError {
    AppError::Internal(format!("Database error: {error}"))
}

async fn select_active_chat_model(
    pool: &sqlx::PgPool,
    requested_model: &str,
) -> Option<ResolvedChatModel> {
    crate::db::models::get_model_with_provider_by_slug(pool, requested_model)
        .await
        .map(|model| ResolvedChatModel {
            slug: model.slug,
            provider_slug: model.provider_slug,
        })
}
