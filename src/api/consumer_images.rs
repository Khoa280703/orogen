use axum::{
    Json,
    extract::{Extension, Path, Query, State},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::AppState;
use crate::api::chat_completions::{duration_to_latency_ms, record_usage};
use crate::api::consumer_api_support::{
    build_user_usage_context, generate_images_with_retry, mark_account_success,
};
use crate::api::plan_enforcement::{REQUEST_KIND_IMAGE, enforce_user_plan_access};
use crate::db::image_generations;
use crate::error::AppError;
use crate::middleware::jwt_auth::JwtUser;
use crate::providers::GeneratedAsset;

const DEFAULT_IMAGE_MODEL: &str = "imagine-x-1";

#[derive(Debug, Deserialize)]
pub struct GenerateImagesRequest {
    pub prompt: String,
    #[serde(default)]
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
pub struct GeneratedImageResponse {
    pub id: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ImageGenerationResponse {
    pub id: i32,
    pub prompt: String,
    pub model_slug: String,
    pub status: String,
    pub images: Vec<GeneratedImageResponse>,
    pub error_message: Option<String>,
    pub created_at: Option<String>,
}

pub async fn generate_images(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Json(payload): Json<GenerateImagesRequest>,
) -> Result<Json<ImageGenerationResponse>, AppError> {
    let prompt = payload.prompt.trim().to_string();
    if prompt.is_empty() {
        return Err(AppError::BadRequest("prompt is required".into()));
    }

    let model = payload
        .model
        .unwrap_or_else(|| DEFAULT_IMAGE_MODEL.to_string());
    let resolved_model = if let Some(model) =
        crate::db::models::get_model_with_provider_by_slug(&state.db, &model).await
    {
        model
    } else if let Some(model) =
        crate::db::models::get_model_with_provider_by_slug(&state.db, DEFAULT_IMAGE_MODEL).await
    {
        model
    } else {
        return Err(AppError::Internal("No active image model is available".into()));
    };
    let model = resolved_model.slug.clone();
    let plan_id =
        enforce_user_plan_access(&state.db, user.user_id, REQUEST_KIND_IMAGE, &model).await?;
    let generation = image_generations::create_generation(&state.db, user.user_id, &prompt, &model)
        .await
        .map_err(db_error)?;

    let provider = state
        .providers
        .get_image_provider(&resolved_model.provider_slug)
        .ok_or_else(|| {
            AppError::Internal(format!(
                "Image provider not registered: {}",
                resolved_model.provider_slug
            ))
        })?;
    let usage = {
        let mut usage =
            build_user_usage_context(user.user_id, model.clone(), REQUEST_KIND_IMAGE);
        usage.plan_id = Some(plan_id);
        usage
    };
    let started_at = std::time::Instant::now();

    match generate_images_with_retry(&state, provider, &model, &prompt, &usage).await {
        Ok((assets, account_id, _account_name)) => {
            let image_values = assets
                .iter()
                .map(|asset| json!({ "id": asset.id, "url": asset.url }))
                .collect::<Vec<_>>();
            let result_urls = serde_json::to_value(&image_values)
                .map_err(|error| AppError::Internal(format!("Serialize image result failed: {error}")))?;

            image_generations::update_generation_result(&state.db, generation.id, &result_urls)
                .await
                .map_err(db_error)?;
            mark_account_success(&state.db, account_id).await;
            record_usage(
                &state,
                &usage,
                account_id,
                "success",
                duration_to_latency_ms(started_at.elapsed()),
            )
            .await;

            Ok(Json(ImageGenerationResponse {
                id: generation.id,
                prompt: generation.prompt,
                model_slug: generation.model_slug,
                status: "completed".into(),
                images: map_assets(assets),
                error_message: None,
                created_at: generation.created_at.map(|value| value.to_rfc3339()),
            }))
        }
        Err(error) => {
            let _ = image_generations::update_generation_error(&state.db, generation.id, &error.to_string()).await;
            Err(error)
        }
    }
}

pub async fn list_history(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<Vec<ImageGenerationResponse>>, AppError> {
    let items = image_generations::list_generations(
        &state.db,
        user.user_id,
        query.limit.clamp(1, 100),
        query.offset.max(0),
    )
    .await
    .map_err(db_error)?;

    Ok(Json(items.into_iter().map(map_generation).collect()))
}

pub async fn get_generation(
    State(state): State<AppState>,
    Extension(user): Extension<JwtUser>,
    Path(id): Path<i32>,
) -> Result<Json<ImageGenerationResponse>, AppError> {
    let item = image_generations::get_generation(&state.db, id, user.user_id)
        .await
        .map_err(db_error)?
        .ok_or_else(|| AppError::NotFound("Image generation not found".into()))?;

    Ok(Json(map_generation(item)))
}

fn map_generation(item: image_generations::ImageGeneration) -> ImageGenerationResponse {
    let images = item
        .result_urls
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|value| {
            Some(GeneratedImageResponse {
                id: value.get("id")?.as_str()?.to_string(),
                url: value.get("url")?.as_str()?.to_string(),
            })
        })
        .collect();

    ImageGenerationResponse {
        id: item.id,
        prompt: item.prompt,
        model_slug: item.model_slug,
        status: item.status,
        images,
        error_message: item.error_message,
        created_at: item.created_at.map(|value| value.to_rfc3339()),
    }
}

fn map_assets(assets: Vec<GeneratedAsset>) -> Vec<GeneratedImageResponse> {
    assets
        .into_iter()
        .map(|asset| GeneratedImageResponse {
            id: asset.id,
            url: asset.url,
        })
        .collect()
}

fn default_limit() -> i64 {
    20
}

fn db_error(error: sqlx::Error) -> AppError {
    AppError::Internal(format!("Database error: {error}"))
}
