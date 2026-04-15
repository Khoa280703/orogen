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
        cookies: account.cookies.clone(),
        proxy_id: Some(replacement.id),
        proxy_url: Some(replacement.url.clone()),
    })
}
