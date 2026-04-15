use axum::{Json, extract::State, http::StatusCode};
use base64::Engine;
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::auth::jwt::generate_token;
use crate::db::balances::get_or_create_balance;
use crate::db::plans::get_plan_by_slug;
use crate::db::user_plans::assign_plan;
use crate::db::users::{CreateUserInput, find_or_create_user};

#[derive(Debug, Deserialize)]
pub struct GoogleAuthRequest {
    pub id_token: String,
}

#[derive(Debug, Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub token_type: String,
    pub user: UserSummary,
}

#[derive(Debug, Serialize)]
pub struct UserSummary {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub locale: String,
}

/// Handle Google OAuth callback
/// Verifies Google token, creates/finds user, assigns free plan, returns JWT
pub async fn handle_google_auth(
    State(state): State<AppState>,
    Json(payload): Json<GoogleAuthRequest>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    // Verify Google token (simplified - in production, verify with Google API)
    // For now, we trust the frontend sent a valid token from NextAuth
    let token_info = verify_google_token(&payload.id_token).map_err(|e| {
        (
            StatusCode::UNAUTHORIZED,
            format!("Invalid Google token: {}", e),
        )
    })?;

    let db = &state.db;

    // Find or create user
    let user = find_or_create_user(
        db,
        CreateUserInput {
            email: token_info.email.clone(),
            name: Some(token_info.name),
            avatar_url: Some(token_info.picture),
            provider: "google".to_string(),
            provider_id: Some(token_info.sub),
            locale: None,
        },
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Get or create balance
    if let Err(e) = get_or_create_balance(db, user.id).await {
        tracing::error!("Failed to create balance: {}", e);
    }

    // Assign free plan if no active plan
    let has_plan = crate::db::user_plans::get_active_plan(db, user.id)
        .await
        .ok()
        .flatten()
        .is_some();

    if !has_plan {
        if let Some(free_plan) = get_plan_by_slug(db, "free").await.ok().flatten() {
            assign_plan(db, user.id, free_plan.id, None).await.ok();
        }
    }

    // Generate JWT token
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default-secret-change-in-production".to_string());
    let token = generate_token(user.id, &user.email, &jwt_secret).map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("JWT error: {}", e),
        )
    })?;

    Ok(Json(AuthResponse {
        access_token: token,
        token_type: "Bearer".to_string(),
        user: UserSummary {
            id: user.id,
            email: user.email,
            name: user.name,
            avatar_url: user.avatar_url,
            locale: user.locale,
        },
    }))
}

#[derive(Debug, Deserialize)]
struct GoogleTokenInfo {
    sub: String,
    email: String,
    name: String,
    picture: String,
}

fn verify_google_token(token: &str) -> Result<GoogleTokenInfo, String> {
    // Simplified verification - decode JWT without signature check
    // In production, verify with Google's public key or OAuth API
    let parts: Vec<&str> = token.split('.').collect();
    if parts.len() != 3 {
        return Err("Invalid token format".to_string());
    }

    // Decode payload (base64url)
    let mut padded = parts[1].to_string();
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    let payload = base64::engine::general_purpose::URL_SAFE
        .decode(padded)
        .map_err(|e| format!("Decode error: {}", e))?;

    let token_info: GoogleTokenInfo =
        serde_json::from_slice(&payload).map_err(|e| format!("Parse error: {}", e))?;

    Ok(token_info)
}
