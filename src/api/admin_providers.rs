// Admin endpoints for provider management
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::db::{create_provider, list_providers, update_provider};
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct ProviderResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub active: bool,
    pub created_at: String,
}

impl From<crate::db::providers::Provider> for ProviderResponse {
    fn from(p: crate::db::providers::Provider) -> Self {
        Self {
            id: p.id,
            name: p.name,
            slug: p.slug,
            active: p.active,
            created_at: p.created_at.to_rfc3339(),
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateProviderRequest {
    pub name: String,
    pub slug: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProviderRequest {
    pub name: Option<String>,
    pub active: Option<bool>,
}

/// GET /admin/providers - List all providers
pub async fn list_all_providers(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let providers = list_providers(&state.db).await;
    let responses: Vec<ProviderResponse> = providers.into_iter().map(Into::into).collect();

    Ok(Json(json!({ "data": responses })))
}

/// POST /admin/providers - Create new provider
pub async fn create_new_provider(
    State(state): State<AppState>,
    Json(req): Json<CreateProviderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = create_provider(&state.db, &req.name, &req.slug)
        .await
        .ok_or(AppError::Internal("Failed to create provider".into()))?;

    Ok(Json(json!({ "data": ProviderResponse::from(provider) })))
}

/// PUT /admin/providers/:id - Update provider
pub async fn update_existing_provider(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateProviderRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let provider = update_provider(&state.db, id, req.name.as_deref(), req.active)
        .await
        .ok_or(AppError::Internal("Failed to update provider".into()))?;

    Ok(Json(json!({ "data": ProviderResponse::from(provider) })))
}
