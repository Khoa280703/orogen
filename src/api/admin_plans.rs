use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sqlx::Row;

use crate::AppState;
use crate::db::plans;
use crate::db::{
    add_model_to_plan, get_plan, list_models_for_plan, remove_model_from_plan, set_plan_models,
};

#[derive(Debug, Serialize)]
pub struct PlanResponse {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub requests_per_day: Option<i32>,
    pub requests_per_month: Option<i32>,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
    pub features: Option<Value>,
    pub active: bool,
    pub sort_order: i32,
    pub created_at: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PlanCreateRequest {
    pub name: String,
    pub slug: String,
    #[serde(default)]
    pub requests_per_day: Option<i32>,
    #[serde(default)]
    pub requests_per_month: Option<i32>,
    #[serde(default)]
    pub price_usd: Option<String>,
    #[serde(default)]
    pub price_vnd: Option<i32>,
    #[serde(default)]
    pub features: Option<Value>,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub sort_order: i32,
}

#[derive(Debug, Deserialize)]
pub struct PlanUpdateRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub requests_per_day: Option<i32>,
    #[serde(default)]
    pub requests_per_month: Option<i32>,
    #[serde(default)]
    pub price_usd: Option<String>,
    #[serde(default)]
    pub price_vnd: Option<i32>,
    #[serde(default)]
    pub features: Option<Value>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub sort_order: Option<i32>,
}

/// GET /admin/plans - List all plans
pub async fn list_plans(
    State(state): State<AppState>,
) -> Result<Json<Vec<PlanResponse>>, (StatusCode, String)> {
    let db = &state.db;

    let plans_list = plans::list_all_plans(db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let response: Vec<PlanResponse> = plans_list
        .into_iter()
        .map(|p| PlanResponse {
            id: p.id,
            name: p.name,
            slug: p.slug,
            requests_per_day: p.requests_per_day,
            requests_per_month: p.requests_per_month,
            price_usd: p.price_usd,
            price_vnd: p.price_vnd,
            features: p.features,
            active: p.active,
            sort_order: p.sort_order,
            created_at: p.created_at.map(|d| d.to_rfc3339()),
        })
        .collect();

    Ok(Json(response))
}

/// POST /admin/plans - Create a new plan
pub async fn create_plan(
    State(state): State<AppState>,
    Json(req): Json<PlanCreateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Check if slug already exists
    let existing =
        sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM plans WHERE slug = $1)")
            .bind(&req.slug)
            .fetch_one(db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    if existing {
        return Err((
            StatusCode::CONFLICT,
            "Plan with this slug already exists".to_string(),
        ));
    }

    let price_usd_str = req.price_usd.clone();
    let features = req.features.unwrap_or_else(|| serde_json::json!({}));

    let row = sqlx::query(
        r#"INSERT INTO plans (name, slug, requests_per_day, requests_per_month, price_usd, price_vnd, features, active, sort_order)
           VALUES ($1, $2, $3, $4, $5::numeric, $6, $7, $8, $9)
           RETURNING id"#,
    )
    .bind(&req.name)
    .bind(&req.slug)
    .bind(req.requests_per_day)
    .bind(req.requests_per_month)
    .bind(&price_usd_str)
    .bind(req.price_vnd)
    .bind(&features)
    .bind(req.active)
    .bind(req.sort_order)
    .fetch_one(db)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, format!("Database error: {}", e)))?;

    let id: i32 = row.get("id");

    Ok(Json(serde_json::json!({ "id": id })))
}

/// PUT /admin/plans/:id - Update a plan
pub async fn update_plan(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<PlanUpdateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify plan exists
    let _existing = plans::get_plan(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    if req.name.is_none()
        && req.requests_per_day.is_none()
        && req.requests_per_month.is_none()
        && req.price_usd.is_none()
        && req.price_vnd.is_none()
        && req.features.is_none()
        && req.active.is_none()
        && req.sort_order.is_none()
    {
        return Ok(Json(serde_json::json!({ "success": true })));
    }

    sqlx::query(
        r#"
        UPDATE plans
        SET
            name = COALESCE($1, name),
            requests_per_day = COALESCE($2, requests_per_day),
            requests_per_month = COALESCE($3, requests_per_month),
            price_usd = COALESCE($4::numeric, price_usd),
            price_vnd = COALESCE($5, price_vnd),
            features = COALESCE($6, features),
            active = COALESCE($7, active),
            sort_order = COALESCE($8, sort_order)
        WHERE id = $9
        "#,
    )
    .bind(req.name)
    .bind(req.requests_per_day)
    .bind(req.requests_per_month)
    .bind(req.price_usd)
    .bind(req.price_vnd)
    .bind(req.features)
    .bind(req.active)
    .bind(req.sort_order)
    .bind(id)
    .execute(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(serde_json::json!({ "success": true })))
}

/// DELETE /admin/plans/:id - Delete a plan
pub async fn delete_plan(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    let existing = plans::get_plan(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    let assigned_count =
        sqlx::query_scalar::<_, i64>(r#"SELECT COUNT(*) FROM user_plans WHERE plan_id = $1"#)
            .bind(id)
            .fetch_one(db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;

    if assigned_count > 0 {
        return Err((
            StatusCode::CONFLICT,
            format!(
                "Cannot delete plan '{}': currently referenced by {} user plan record(s)",
                existing.name, assigned_count
            ),
        ));
    }

    sqlx::query(r#"DELETE FROM plans WHERE id = $1"#)
        .bind(id)
        .execute(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(json!({ "success": true })))
}

// Plan-Model Association Endpoints

#[derive(Debug, Deserialize)]
pub struct PlanModelsRequest {
    pub model_ids: Vec<i32>,
}

/// GET /admin/plans/:id/models - List models in plan
pub async fn list_plan_models(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify plan exists
    let _plan = get_plan(db, plan_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    let models = list_models_for_plan(db, plan_id).await;

    let response: Vec<serde_json::Value> = models
        .into_iter()
        .map(|m| {
            json!({
                "id": m.id,
                "name": m.name,
                "slug": m.slug,
                "active": m.active,
                "sort_order": m.sort_order,
            })
        })
        .collect();

    Ok(Json(json!({ "data": response })))
}

/// POST /admin/plans/:id/models - Add model(s) to plan
pub async fn add_models_to_plan(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
    Json(req): Json<PlanModelsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify plan exists
    let _plan = get_plan(db, plan_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    // Verify all models exist
    for model_id in &req.model_ids {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM models WHERE id = $1)")
                .bind(model_id)
                .fetch_one(db)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?;

        if !exists {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Model {} not found", model_id),
            ));
        }
    }

    // Add models to plan
    for model_id in &req.model_ids {
        add_model_to_plan(db, plan_id, *model_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    Ok(Json(
        json!({ "success": true, "message": "Models added to plan" }),
    ))
}

/// DELETE /admin/plans/:id/models/:model_id - Remove model from plan
pub async fn remove_model_from_plan_endpoint(
    State(state): State<AppState>,
    Path((plan_id, model_id)): Path<(i32, i32)>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify plan exists
    let _plan = get_plan(db, plan_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    // Verify model exists
    let _model = sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM models WHERE id = $1)")
        .bind(model_id)
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .then_some(())
        .ok_or((StatusCode::BAD_REQUEST, "Model not found".to_string()))?;

    remove_model_from_plan(db, plan_id, model_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(
        json!({ "success": true, "message": "Model removed from plan" }),
    ))
}

/// PUT /admin/plans/:id/models - Replace all models in plan
pub async fn set_all_plan_models(
    State(state): State<AppState>,
    Path(plan_id): Path<i32>,
    Json(req): Json<PlanModelsRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify plan exists
    let _plan = get_plan(db, plan_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Plan not found".to_string()))?;

    // Verify all models exist
    for model_id in &req.model_ids {
        let exists =
            sqlx::query_scalar::<_, bool>("SELECT EXISTS(SELECT 1 FROM models WHERE id = $1)")
                .bind(model_id)
                .fetch_one(db)
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Database error: {}", e),
                    )
                })?;

        if !exists {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Model {} not found", model_id),
            ));
        }
    }

    // Replace all models
    set_plan_models(db, plan_id, req.model_ids)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(Json(
        json!({ "success": true, "message": "Plan models updated" }),
    ))
}
