use std::sync::Arc;

use tokio::sync::mpsc;

use crate::AppState;
use crate::account::pool::CurrentAccount;
use crate::db::{account_sessions, accounts};
use crate::error::AppError;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::image_provider::ImageProvider;
use crate::providers::{ChatMessage, ChatStreamEvent, GeneratedAsset, ProviderError};
use crate::services::proxy_failover;

pub(crate) fn build_user_usage_context(
    user_id: i32,
    model: String,
    request_kind: &'static str,
) -> crate::api::chat_completions::UsageContext {
    crate::api::chat_completions::UsageContext {
        api_key_id: None,
        user_id: Some(user_id),
        plan_id: None,
        model,
        request_kind,
    }
}

pub(crate) async fn start_chat_stream_with_retry(
    state: &AppState,
    provider: Arc<dyn ChatProvider>,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
) -> Result<(mpsc::UnboundedReceiver<ChatStreamEvent>, Option<i32>, String), AppError> {
    let account = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
    let started_at = std::time::Instant::now();

    match open_chat_stream(&provider, &account, model, messages, system_prompt).await {
        Ok(rx) => Ok((rx, account.id, account.name)),
        Err(ProviderError::ProxyFailed(message)) => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(state, &account, &message).await {
                match open_chat_stream(&provider, &next, model, messages, system_prompt).await {
                    Ok(rx) => Ok((rx, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(
                    state,
                    usage_context,
                    account.id,
                    &ProviderError::ProxyFailed(message.clone()),
                    started_at,
                )
                .await;
                Err(AppError::GrokApi(message))
            }
        }
        Err(ProviderError::CfBlocked) if account.proxy_id.is_some() => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(
                state,
                &account,
                "Proxy received Cloudflare block from upstream.",
            )
            .await
            {
                match open_chat_stream(&provider, &next, model, messages, system_prompt).await {
                    Ok(rx) => Ok((rx, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(state, usage_context, account.id, &ProviderError::CfBlocked, started_at).await;
                Err(AppError::GrokApi("Cloudflare blocked".into()))
            }
        }
        Err(ProviderError::RateLimited) => {
            mark_rate_limited(&state.db, account.id).await;
            if state.pool.rotate().await {
                let next = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
                match open_chat_stream(&provider, &next, model, messages, system_prompt).await {
                    Ok(rx) => Ok((rx, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(state, usage_context, account.id, &ProviderError::RateLimited, started_at).await;
                Err(AppError::GrokApi("Rate limited".into()))
            }
        }
        Err(error) => {
            finalize_failed_start(state, usage_context, account.id, &error, started_at).await;
            Err(error.into())
        }
    }
}

pub(crate) async fn generate_images_with_retry(
    state: &AppState,
    provider: Arc<dyn ImageProvider>,
    model: &str,
    prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
) -> Result<(Vec<GeneratedAsset>, Option<i32>, String), AppError> {
    let account = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
    let started_at = std::time::Instant::now();

    match provider
        .generate_images(&account.cookies, account.proxy_url.as_ref(), prompt, model)
        .await
    {
        Ok(assets) => Ok((assets, account.id, account.name)),
        Err(ProviderError::ProxyFailed(message)) => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(state, &account, &message).await {
                match provider
                    .generate_images(&next.cookies, next.proxy_url.as_ref(), prompt, model)
                    .await
                {
                    Ok(assets) => Ok((assets, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(
                    state,
                    usage_context,
                    account.id,
                    &ProviderError::ProxyFailed(message.clone()),
                    started_at,
                )
                .await;
                Err(AppError::GrokApi(message))
            }
        }
        Err(ProviderError::CfBlocked) if account.proxy_id.is_some() => {
            if let Some(next) = proxy_failover::deactivate_failed_proxy(
                state,
                &account,
                "Proxy received Cloudflare block from upstream.",
            )
            .await
            {
                match provider
                    .generate_images(&next.cookies, next.proxy_url.as_ref(), prompt, model)
                    .await
                {
                    Ok(assets) => Ok((assets, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(state, usage_context, account.id, &ProviderError::CfBlocked, started_at).await;
                Err(AppError::GrokApi("Cloudflare blocked".into()))
            }
        }
        Err(ProviderError::RateLimited) => {
            mark_rate_limited(&state.db, account.id).await;
            if state.pool.rotate().await {
                let next = state.pool.get_current().await.ok_or(AppError::NoAccounts)?;
                match provider
                    .generate_images(&next.cookies, next.proxy_url.as_ref(), prompt, model)
                    .await
                {
                    Ok(assets) => Ok((assets, next.id, next.name)),
                    Err(error) => {
                        finalize_failed_start(state, usage_context, next.id, &error, started_at).await;
                        Err(error.into())
                    }
                }
            } else {
                finalize_failed_start(state, usage_context, account.id, &ProviderError::RateLimited, started_at).await;
                Err(AppError::GrokApi("Rate limited".into()))
            }
        }
        Err(error) => {
            finalize_failed_start(state, usage_context, account.id, &error, started_at).await;
            Err(error.into())
        }
    }
}

pub(crate) async fn mark_account_success(db: &sqlx::PgPool, account_id: Option<i32>) {
    if let Some(id) = account_id {
        let _ = accounts::update_health_counts(db, id, true).await;
    }
}

async fn open_chat_stream(
    provider: &Arc<dyn ChatProvider>,
    account: &CurrentAccount,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: &str,
) -> Result<mpsc::UnboundedReceiver<ChatStreamEvent>, ProviderError> {
    provider
        .chat_stream(
            &account.cookies,
            account.proxy_url.as_ref(),
            model,
            messages,
            system_prompt,
        )
        .await
}

async fn finalize_failed_start(
    state: &AppState,
    usage_context: &crate::api::chat_completions::UsageContext,
    account_id: Option<i32>,
    error: &ProviderError,
    started_at: std::time::Instant,
) {
    if let Some(id) = account_id {
        match error {
            ProviderError::Unauthorized => {
                let _ = account_sessions::mark_session_expired(
                    &state.db,
                    id,
                    "Upstream Grok session expired or cookies are invalid.",
                )
                .await;
            }
            ProviderError::ProxyFailed(_) => {}
            _ => {
                let _ = accounts::update_health_counts(&state.db, id, false).await;
            }
        }
    }
    crate::api::chat_completions::record_usage(
        state,
        usage_context,
        account_id,
        request_error_status(error),
        crate::api::chat_completions::duration_to_latency_ms(started_at.elapsed()),
    )
    .await;
}

async fn mark_rate_limited(db: &sqlx::PgPool, account_id: Option<i32>) {
    if let Some(id) = account_id {
        let _ = accounts::record_rate_limited_attempt(db, id).await;
    }
}

fn request_error_status(error: &ProviderError) -> &'static str {
    match error {
        ProviderError::RateLimited => "rate_limited",
        ProviderError::Unauthorized => "unauthorized",
        ProviderError::CfBlocked => "cf_blocked",
        ProviderError::ProxyFailed(_) => "proxy_failed",
        ProviderError::Network(_) => "error",
    }
}
