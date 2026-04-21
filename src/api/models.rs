use axum::Json;
use axum::extract::State;
use axum::http::HeaderMap;
use serde_json::{Value, json};
use sqlx::query_scalar;

use crate::AppState;
use crate::api::request_orchestrator::list_supported_public_models;
use crate::error::AppError;
use crate::middleware::jwt_auth::validate_token;

pub async fn list_models(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, AppError> {
    let bearer_token = extract_bearer_token(&headers);
    let (user_id, api_key_plan_id) = if let Some(token) = bearer_token {
        let api_key = crate::db::api_keys::get_key_by_value(&state.db, &token)
            .await
            .map_err(|error| AppError::Internal(format!("Failed to resolve API key: {error}")))?;

        if let Some(key) = api_key {
            (key.user_id, key.plan_id)
        } else if let Ok(jwt_user) = validate_token(&token) {
            (Some(jwt_user.user_id), None)
        } else {
            // Ignore unrelated bearer tokens on this public endpoint instead of returning 401.
            (None, None)
        }
    } else {
        (None, None)
    };

    // Get models based on user's plan or all models if anonymous
    let models = if let Some(plan_id) = api_key_plan_id {
        list_supported_public_models(&state, Some(plan_id)).await
    } else if let Some(uid) = user_id {
        // Get active plan_id for user
        let plan_id_result: Option<i32> = query_scalar(
            r#"SELECT plan_id FROM user_plans WHERE user_id = $1 AND active = true LIMIT 1"#,
        )
        .bind(uid)
        .fetch_optional(&state.db)
        .await
        .map_err(|e| AppError::Internal(format!("Database error: {}", e)))?
        .flatten();

        let plan_id = plan_id_result.ok_or(AppError::PlanRequired)?;
        list_supported_public_models(&state, Some(plan_id)).await
    } else {
        list_supported_public_models(&state, None).await
    };

    Ok(Json(model_catalog_response(&models)))
}

fn model_catalog_response(models: &[crate::db::public_models::PublicModelWithRoute]) -> Value {
    let data: Vec<Value> = models
        .iter()
        .map(|m| {
            json!({
                "id": m.slug,
                "type": "model",
                "object": "model",
                "display_name": m.display_name,
                "description": m.description,
                "created": m.created_at.timestamp(),
                "created_at": m.created_at.to_rfc3339(),
                "owned_by": m.provider_slug,
            })
        })
        .collect();

    json!({ "object": "list", "data": data })
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    headers
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
}

#[cfg(test)]
mod tests {
    use axum::http::{HeaderMap, HeaderValue};
    use chrono::Utc;

    use crate::db::public_models::PublicModelWithRoute;

    #[test]
    fn serializes_public_model_catalog_entries() {
        let payload = super::model_catalog_response(&[PublicModelWithRoute {
            public_model_id: 1,
            slug: "gpt-5.1".to_string(),
            display_name: "GPT-5.1".to_string(),
            description: Some("Public Codex route".to_string()),
            created_at: Utc::now(),
            provider_slug: "codex".to_string(),
            upstream_model_slug: "gpt-5-codex".to_string(),
        }]);

        assert_eq!(payload["object"], "list");
        assert_eq!(payload["data"][0]["id"], "gpt-5.1");
        assert_eq!(payload["data"][0]["owned_by"], "codex");
        assert_eq!(payload["data"][0]["display_name"], "GPT-5.1");
        assert_eq!(payload["data"][0]["description"], "Public Codex route");
    }

    #[test]
    fn preserves_filtered_plan_scoped_catalog_input() {
        let payload = super::model_catalog_response(&[PublicModelWithRoute {
            public_model_id: 2,
            slug: "gpt-5-mini".to_string(),
            display_name: "GPT-5 Mini".to_string(),
            description: None,
            created_at: Utc::now(),
            provider_slug: "codex".to_string(),
            upstream_model_slug: "gpt-5-codex-mini".to_string(),
        }]);

        let models = payload["data"].as_array().unwrap();
        assert_eq!(models.len(), 1);
        assert_eq!(models[0]["id"], "gpt-5-mini");
    }

    #[test]
    fn prefers_authorization_bearer_over_x_api_key() {
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_static("Bearer token-from-auth"),
        );
        headers.insert("x-api-key", HeaderValue::from_static("token-from-header"));

        assert_eq!(
            super::extract_bearer_token(&headers).as_deref(),
            Some("token-from-auth")
        );
    }
}
