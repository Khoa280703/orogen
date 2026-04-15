use axum::{
    Json,
    extract::{Extension, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::transactions::create_transaction;
use crate::middleware::jwt_auth::JwtUser;

#[derive(Debug, Deserialize)]
pub struct ManualTopupRequest {
    pub amount: f64,
    pub reference: String,
    pub proof_url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CryptoTopupRequest {
    pub amount: f64,
    pub currency: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct TopupResponse {
    pub transaction_id: i32,
    pub amount: f64,
    pub status: String,
    pub checkout_url: Option<String>,
}

/// POST /user/topup/manual - Submit manual topup request
pub async fn create_manual_topup(
    State(state): State<AppState>,
    user: Extension<JwtUser>,
    Json(payload): Json<ManualTopupRequest>,
) -> Result<Json<TopupResponse>, (StatusCode, String)> {
    if payload.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Amount must be positive".to_string(),
        ));
    }

    let db = &state.db;

    let transaction = create_transaction(
        db,
        user.user_id,
        "topup",
        Some("manual"),
        payload.amount,
        "USD",
        "pending",
        Some(&payload.reference),
        payload.proof_url.as_deref(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    Ok(Json(TopupResponse {
        transaction_id: transaction.id,
        amount: payload.amount,
        status: "pending".to_string(),
        checkout_url: None,
    }))
}

/// POST /user/topup/crypto - Create crypto topup via fpayment
pub async fn create_crypto_topup(
    State(state): State<AppState>,
    user: Extension<JwtUser>,
    Json(payload): Json<CryptoTopupRequest>,
) -> Result<Json<TopupResponse>, (StatusCode, String)> {
    if payload.amount <= 0.0 {
        return Err((
            StatusCode::BAD_REQUEST,
            "Amount must be positive".to_string(),
        ));
    }

    // TODO: Integrate with fpayment API
    // For now, return a placeholder
    let currency = payload.currency.unwrap_or_else(|| "USDT".to_string());

    let db = &state.db;

    // Create pending transaction
    let transaction = create_transaction(
        db,
        user.user_id,
        "topup",
        Some("crypto"),
        payload.amount,
        &currency,
        "pending",
        None,
        None,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // TODO: Create fpayment invoice and get checkout URL
    let checkout_url = Some(format!("https://fpayment.com/checkout/{}", transaction.id));

    Ok(Json(TopupResponse {
        transaction_id: transaction.id,
        amount: payload.amount,
        status: "pending".to_string(),
        checkout_url,
    }))
}
