// Admin endpoints for model management
use axum::Json;
use axum::extract::{Path, State};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::db::{create_model, get_provider, list_models, update_model};
use crate::error::AppError;

#[derive(Debug, Serialize)]
pub struct ModelResponse {
    pub id: i32,
    pub provider_id: i32,
    pub provider_name: String,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    pub active: bool,
    pub sort_order: i32,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateModelRequest {
    pub provider_id: i32,
    pub name: String,
    pub slug: String,
    pub description: Option<String>,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct UpdateModelRequest {
    pub name: Option<String>,
    pub description: Option<String>,
    pub active: Option<bool>,
    pub sort_order: Option<i32>,
}

/// GET /admin/models - List all models with provider info
pub async fn list_all_models(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, AppError> {
    let models = list_models(&state.db).await;

    let mut responses = Vec::new();
    for m in models {
        let provider_name = get_provider(&state.db, m.provider_id)
            .await
            .map(|p| p.name)
            .unwrap_or_else(|| "Unknown".to_string());

        responses.push(ModelResponse {
            id: m.id,
            provider_id: m.provider_id,
            provider_name,
            name: m.name,
            slug: m.slug,
            description: m.description,
            active: m.active,
            sort_order: m.sort_order,
            created_at: m.created_at.to_rfc3339(),
        });
    }

    Ok(Json(json!({ "data": responses })))
}

/// POST /admin/models - Create new model
pub async fn create_new_model(
    State(state): State<AppState>,
    Json(req): Json<CreateModelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    // Verify provider exists
    if get_provider(&state.db, req.provider_id).await.is_none() {
        return Err(AppError::Internal("Provider not found".into()));
    }

    let model = create_model(
        &state.db,
        req.provider_id,
        &req.name,
        &req.slug,
        req.description.as_deref(),
        req.sort_order,
    )
    .await
    .ok_or(AppError::Internal("Failed to create model".into()))?;

    crate::db::public_model_routes::sync_public_catalog_for_model(&state.db, model.id)
        .await
        .map_err(|error| AppError::Internal(format!("Failed to sync public catalog: {error}")))?;

    let provider_name = get_provider(&state.db, model.provider_id)
        .await
        .map(|p| p.name)
        .unwrap_or_else(|| "Unknown".to_string());

    let response = ModelResponse {
        id: model.id,
        provider_id: model.provider_id,
        provider_name,
        name: model.name,
        slug: model.slug,
        description: model.description,
        active: model.active,
        sort_order: model.sort_order,
        created_at: model.created_at.to_rfc3339(),
    };

    Ok(Json(json!({ "data": response })))
}

/// PUT /admin/models/:id - Update model
pub async fn update_existing_model(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UpdateModelRequest>,
) -> Result<Json<serde_json::Value>, AppError> {
    let model = update_model(
        &state.db,
        id,
        req.name.as_deref(),
        req.description.as_deref(),
        req.active,
        req.sort_order,
    )
    .await
    .ok_or(AppError::Internal("Failed to update model".into()))?;

    crate::db::public_model_routes::sync_public_catalog_for_model(&state.db, model.id)
        .await
        .map_err(|error| AppError::Internal(format!("Failed to sync public catalog: {error}")))?;

    let provider_name = get_provider(&state.db, model.provider_id)
        .await
        .map(|p| p.name)
        .unwrap_or_else(|| "Unknown".to_string());

    let response = ModelResponse {
        id: model.id,
        provider_id: model.provider_id,
        provider_name,
        name: model.name,
        slug: model.slug,
        description: model.description,
        active: model.active,
        sort_order: model.sort_order,
        created_at: model.created_at.to_rfc3339(),
    };

    Ok(Json(json!({ "data": response })))
}

/// DELETE /admin/models/:id - Deactivate model (soft delete)
pub async fn delete_model(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, AppError> {
    if let Some(model) = update_model(&state.db, id, None, None, Some(false), None).await {
        crate::db::public_model_routes::sync_public_catalog_for_model(&state.db, model.id)
            .await
            .map_err(|error| {
                AppError::Internal(format!("Failed to sync public catalog: {error}"))
            })?;
    }

    Ok(Json(
        json!({ "success": true, "message": "Model deactivated" }),
    ))
}
