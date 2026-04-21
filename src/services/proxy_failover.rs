use crate::AppState;
use crate::account::pool::CurrentAccount;
use crate::db::{accounts, proxies};

pub async fn deactivate_failed_proxy(
    state: &AppState,
    account: &CurrentAccount,
    error_message: &str,
) -> Option<CurrentAccount> {
    let account_id = account.id?;
    let proxy_id = account.proxy_id?;
    let affected_account_ids = accounts::list_account_ids_by_proxy(&state.db, proxy_id)
        .await
        .unwrap_or_default();

    tracing::warn!(
        account_id = account.id,
        account = account.name,
        proxy_id,
        proxy = account.proxy_url,
        error = error_message,
        "Proxy failed for account, disabling proxy and detaching accounts"
    );

    if let Err(error) = proxies::deactivate_proxy(&state.db, proxy_id).await {
        tracing::warn!(proxy_id, error = %error, "Failed to deactivate proxy");
        return None;
    }

    if let Err(error) = proxies::detach_proxy_from_accounts(&state.db, proxy_id).await {
        tracing::warn!(proxy_id, error = %error, "Failed to detach proxy from accounts");
        return None;
    }

    let active_proxies = match proxies::list_active_proxies(&state.db).await {
        Ok(rows) if !rows.is_empty() => rows,
        Ok(_) => return None,
        Err(error) => {
            tracing::warn!(error = %error, "Failed to load active proxies after deactivation");
            return None;
        }
    };

    for (index, affected_account_id) in affected_account_ids.iter().enumerate() {
        let replacement = &active_proxies[index % active_proxies.len()];
        if let Err(error) =
            accounts::assign_proxy_to_account(&state.db, *affected_account_id, Some(replacement.id))
                .await
        {
            tracing::warn!(
                account_id = affected_account_id,
                proxy_id = replacement.id,
                error = %error,
                "Failed to reassign replacement proxy to account"
            );
        }
    }

    let replacement_index = affected_account_ids
        .iter()
        .position(|id| *id == account_id)
        .unwrap_or(0);
    let replacement = &active_proxies[replacement_index % active_proxies.len()];

    Some(CurrentAccount {
        id: Some(account_id),
        name: account.name.clone(),
        provider_slug: account.provider_slug.clone(),
        account_label: account.account_label.clone(),
        external_account_id: account.external_account_id.clone(),
        auth_mode: account.auth_mode.clone(),
        credential: account.credential.clone(),
        proxy_id: Some(replacement.id),
        proxy_url: Some(replacement.url.clone()),
    })
}

pub async fn deactivate_proxy_for_account_id(
    state: &AppState,
    account_id: i32,
    error_message: &str,
) {
    let account = match accounts::get_account(&state.db, account_id).await {
        Ok(Some(account)) => account,
        Ok(None) => return,
        Err(error) => {
            tracing::warn!(account_id, error = %error, "Failed to load account for proxy cleanup");
            return;
        }
    };

    let Some(proxy_id) = account.proxy_id else {
        return;
    };

    let affected_account_ids = accounts::list_account_ids_by_proxy(&state.db, proxy_id)
        .await
        .unwrap_or_default();

    tracing::warn!(
        account_id,
        proxy_id,
        proxy = ?account.proxy_id,
        error = error_message,
        "Proxy failed for account during stream, disabling proxy for future requests"
    );

    if let Err(error) = proxies::deactivate_proxy(&state.db, proxy_id).await {
        tracing::warn!(proxy_id, error = %error, "Failed to deactivate proxy");
        return;
    }

    if let Err(error) = proxies::detach_proxy_from_accounts(&state.db, proxy_id).await {
        tracing::warn!(proxy_id, error = %error, "Failed to detach proxy from accounts");
        return;
    }

    let active_proxies = match proxies::list_active_proxies(&state.db).await {
        Ok(rows) if !rows.is_empty() => rows,
        Ok(_) => return,
        Err(error) => {
            tracing::warn!(error = %error, "Failed to load active proxies after stream proxy failure");
            return;
        }
    };

    for (index, affected_account_id) in affected_account_ids.iter().enumerate() {
        let replacement = &active_proxies[index % active_proxies.len()];
        if let Err(error) =
            accounts::assign_proxy_to_account(&state.db, *affected_account_id, Some(replacement.id))
                .await
        {
            tracing::warn!(
                account_id = affected_account_id,
                proxy_id = replacement.id,
                error = %error,
                "Failed to reassign replacement proxy after stream proxy failure"
            );
        }
    }
}
