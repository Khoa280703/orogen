use std::collections::HashSet;
use std::sync::Arc;

use tokio::sync::mpsc;

use crate::AppState;
use crate::account::pool::CurrentAccount;
use crate::db::{account_sessions, accounts};
use crate::error::AppError;
use crate::providers::chat_provider::ChatProvider;
use crate::providers::image_provider::ImageProvider;
use crate::providers::types::ProviderRoutingDisposition;
use crate::providers::{ChatMessage, ChatStreamEvent, GeneratedAsset, ProviderError};
use crate::services::proxy_failover;

pub(crate) fn build_user_usage_context(
    user_id: i32,
    provider_slug: String,
    model: String,
    request_kind: &'static str,
) -> crate::api::chat_completions::UsageContext {
    crate::api::chat_completions::UsageContext {
        api_key_id: None,
        user_id: Some(user_id),
        plan_id: None,
        provider_slug,
        model,
        request_kind,
    }
}

pub(crate) async fn start_chat_stream_with_retry(
    state: &AppState,
    provider: Arc<dyn ChatProvider>,
    provider_slug: &str,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
) -> Result<
    (
        mpsc::UnboundedReceiver<ChatStreamEvent>,
        Option<i32>,
        String,
    ),
    AppError,
> {
    let account = state
        .pool
        .get_current_for_provider(provider_slug)
        .await
        .ok_or(AppError::NoAccounts)?;
    let started_at = std::time::Instant::now();

    match open_chat_stream(&provider, &account, model, messages, system_prompt).await {
        Ok(rx) => Ok((rx, account.id, account.name)),
        Err(error) => {
            recover_chat_stream_after_error(
                state,
                provider,
                provider_slug,
                model,
                messages,
                system_prompt,
                usage_context,
                started_at,
                account,
                &error,
            )
            .await
        }
    }
}

pub(crate) async fn generate_images_with_retry(
    state: &AppState,
    provider: Arc<dyn ImageProvider>,
    provider_slug: &str,
    model: &str,
    prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
) -> Result<(Vec<GeneratedAsset>, Option<i32>, String), AppError> {
    let account = state
        .pool
        .get_current_for_provider(provider_slug)
        .await
        .ok_or(AppError::NoAccounts)?;
    let started_at = std::time::Instant::now();

    match provider.generate_images(&account, prompt, model).await {
        Ok(assets) => Ok((assets, account.id, account.name)),
        Err(error) => {
            recover_image_generation_after_error(
                state,
                provider,
                provider_slug,
                model,
                prompt,
                usage_context,
                started_at,
                account,
                &error,
            )
            .await
        }
    }
}

pub(crate) async fn mark_account_success(db: &sqlx::PgPool, account_id: Option<i32>) {
    if let Some(id) = account_id {
        let _ = accounts::update_health_counts(db, id, true).await;
    }
}

pub(crate) async fn finalize_stream_provider_error(
    state: &AppState,
    usage_context: &crate::api::chat_completions::UsageContext,
    account_id: Option<i32>,
    error: &ProviderError,
    started_at: std::time::Instant,
) {
    apply_provider_error_state(state, account_id, error).await;
    crate::api::chat_completions::record_usage(
        state,
        usage_context,
        account_id,
        request_error_status(error),
        crate::api::chat_completions::duration_to_latency_ms(started_at.elapsed()),
    )
    .await;
}

async fn record_provider_error_usage(
    state: &AppState,
    usage_context: &crate::api::chat_completions::UsageContext,
    account_id: Option<i32>,
    error: &ProviderError,
    started_at: std::time::Instant,
) {
    crate::api::chat_completions::record_usage(
        state,
        usage_context,
        account_id,
        request_error_status(error),
        crate::api::chat_completions::duration_to_latency_ms(started_at.elapsed()),
    )
    .await;
}

async fn open_chat_stream(
    provider: &Arc<dyn ChatProvider>,
    account: &CurrentAccount,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: &str,
) -> Result<mpsc::UnboundedReceiver<ChatStreamEvent>, ProviderError> {
    provider
        .chat_stream(account, model, messages, system_prompt)
        .await
}

async fn recover_chat_stream_after_error(
    state: &AppState,
    provider: Arc<dyn ChatProvider>,
    provider_slug: &str,
    model: &str,
    messages: &[ChatMessage],
    system_prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
    started_at: std::time::Instant,
    failed_account: CurrentAccount,
    initial_error: &ProviderError,
) -> Result<
    (
        mpsc::UnboundedReceiver<ChatStreamEvent>,
        Option<i32>,
        String,
    ),
    AppError,
> {
    let mut seen_account_ids = HashSet::new();
    let mut current_account = failed_account;
    let mut current_error = initial_error.clone();

    loop {
        if let Some(id) = current_account.id {
            seen_account_ids.insert(id);
        }

        match current_error.routing_disposition() {
            ProviderRoutingDisposition::RetryNextAccount
            | ProviderRoutingDisposition::ExpireAccount => {
                apply_provider_error_state(state, current_account.id, &current_error).await;

                let next = state
                    .pool
                    .get_next_for_provider(provider_slug, current_account.id, &seen_account_ids)
                    .await;

                let Some(next) = next else {
                    record_provider_error_usage(
                        state,
                        usage_context,
                        current_account.id,
                        &current_error,
                        started_at,
                    )
                    .await;
                    return Err(current_error.into());
                };

                match open_chat_stream(&provider, &next, model, messages, system_prompt).await {
                    Ok(rx) => return Ok((rx, next.id, next.name)),
                    Err(next_error) => {
                        current_account = next;
                        current_error = next_error;
                    }
                }
            }
            ProviderRoutingDisposition::DeactivateProxy => {
                let message = current_error.to_string();
                let Some(next) =
                    proxy_failover::deactivate_failed_proxy(state, &current_account, &message)
                        .await
                else {
                    finalize_failed_start(
                        state,
                        usage_context,
                        current_account.id,
                        &current_error,
                        started_at,
                    )
                    .await;
                    return Err(current_error.into());
                };

                match open_chat_stream(&provider, &next, model, messages, system_prompt).await {
                    Ok(rx) => return Ok((rx, next.id, next.name)),
                    Err(next_error) => {
                        current_account = next;
                        current_error = next_error;
                    }
                }
            }
            _ => {
                finalize_failed_start(
                    state,
                    usage_context,
                    current_account.id,
                    &current_error,
                    started_at,
                )
                .await;
                return Err(current_error.into());
            }
        }
    }
}

async fn recover_image_generation_after_error(
    state: &AppState,
    provider: Arc<dyn ImageProvider>,
    provider_slug: &str,
    model: &str,
    prompt: &str,
    usage_context: &crate::api::chat_completions::UsageContext,
    started_at: std::time::Instant,
    failed_account: CurrentAccount,
    initial_error: &ProviderError,
) -> Result<(Vec<GeneratedAsset>, Option<i32>, String), AppError> {
    let mut seen_account_ids = HashSet::new();
    let mut current_account = failed_account;
    let mut current_error = initial_error.clone();

    loop {
        if let Some(id) = current_account.id {
            seen_account_ids.insert(id);
        }

        match current_error.routing_disposition() {
            ProviderRoutingDisposition::RetryNextAccount
            | ProviderRoutingDisposition::ExpireAccount => {
                apply_provider_error_state(state, current_account.id, &current_error).await;

                let next = state
                    .pool
                    .get_next_for_provider(provider_slug, current_account.id, &seen_account_ids)
                    .await;

                let Some(next) = next else {
                    record_provider_error_usage(
                        state,
                        usage_context,
                        current_account.id,
                        &current_error,
                        started_at,
                    )
                    .await;
                    return Err(current_error.into());
                };

                match provider.generate_images(&next, prompt, model).await {
                    Ok(assets) => return Ok((assets, next.id, next.name)),
                    Err(next_error) => {
                        current_account = next;
                        current_error = next_error;
                    }
                }
            }
            ProviderRoutingDisposition::DeactivateProxy => {
                let message = current_error.to_string();
                let Some(next) =
                    proxy_failover::deactivate_failed_proxy(state, &current_account, &message)
                        .await
                else {
                    finalize_failed_start(
                        state,
                        usage_context,
                        current_account.id,
                        &current_error,
                        started_at,
                    )
                    .await;
                    return Err(current_error.into());
                };

                match provider.generate_images(&next, prompt, model).await {
                    Ok(assets) => return Ok((assets, next.id, next.name)),
                    Err(next_error) => {
                        current_account = next;
                        current_error = next_error;
                    }
                }
            }
            _ => {
                finalize_failed_start(
                    state,
                    usage_context,
                    current_account.id,
                    &current_error,
                    started_at,
                )
                .await;
                return Err(current_error.into());
            }
        }
    }
}

async fn finalize_failed_start(
    state: &AppState,
    usage_context: &crate::api::chat_completions::UsageContext,
    account_id: Option<i32>,
    error: &ProviderError,
    started_at: std::time::Instant,
) {
    finalize_stream_provider_error(state, usage_context, account_id, error, started_at).await;
}

fn request_error_status(error: &ProviderError) -> &'static str {
    error.usage_status()
}

async fn apply_provider_error_state(
    state: &AppState,
    account_id: Option<i32>,
    error: &ProviderError,
) {
    if let Some(id) = account_id {
        match error.routing_disposition() {
            ProviderRoutingDisposition::RetryNextAccount => match error {
                ProviderError::RateLimited => {
                    let _ = accounts::record_rate_limited_attempt(&state.db, id).await;
                }
                ProviderError::UpstreamTransient(_) => {
                    let _ =
                        accounts::mark_account_transient_failure(&state.db, id, &error.to_string())
                            .await;
                }
                _ => {}
            },
            ProviderRoutingDisposition::ExpireAccount => {
                let _ = accounts::mark_account_auth_invalid(
                    &state.db,
                    id,
                    "Upstream account credentials expired or are invalid.",
                )
                .await;
                let _ = account_sessions::mark_session_expired(
                    &state.db,
                    id,
                    "Upstream account credentials expired or are invalid.",
                )
                .await;
            }
            ProviderRoutingDisposition::DeactivateProxy => {
                let _ =
                    accounts::mark_account_proxy_failed(&state.db, id, &error.to_string()).await;
                proxy_failover::deactivate_proxy_for_account_id(state, id, &error.to_string())
                    .await;
            }
            _ if error.should_mark_account_unhealthy() => {
                let _ = accounts::mark_account_transient_failure(&state.db, id, &error.to_string())
                    .await;
            }
            _ => {}
        }
    }
}
