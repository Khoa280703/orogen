use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};

use crate::AppState;
use crate::db::balances::add_credit;
use crate::db::transactions::{get_transaction, list_pending_manual, update_status};

#[derive(Debug, Serialize)]
pub struct PaymentListResponse {
    pub payments: Vec<PaymentDetail>,
}

#[derive(Debug, Serialize)]
pub struct PaymentDetail {
    pub id: i32,
    pub user_id: i32,
    pub user_email: Option<String>,
    pub user_name: Option<String>,
    pub amount: String,
    pub currency: String,
    pub reference: Option<String>,
    pub proof_url: Option<String>,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct ApproveRequest {
    pub notes: Option<String>,
}

/// GET /admin/payments - List pending manual topups
pub async fn list_payments(
    State(state): State<AppState>,
) -> Result<Json<PaymentListResponse>, (StatusCode, String)> {
    let db = &state.db;

    let transactions = list_pending_manual(db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let payments: Vec<PaymentDetail> = transactions
        .into_iter()
        .map(|t| PaymentDetail {
            id: t.id,
            user_id: t.user_id,
            user_email: None, // TODO: Include from JOIN
            user_name: None,
            amount: t.amount,
            currency: t.currency,
            reference: t.reference,
            proof_url: t.proof_url,
            status: t.status,
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(PaymentListResponse { payments }))
}

/// PUT /admin/payments/:id/approve - Approve payment
pub async fn approve_payment(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<ApproveRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;

    // Get transaction
    let transaction = get_transaction(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Payment not found".to_string()))?;

    // Verify it's a pending manual topup
    if transaction.status != "pending" || transaction.method.as_deref() != Some("manual") {
        return Err((
            StatusCode::BAD_REQUEST,
            "Invalid payment status or method".to_string(),
        ));
    }

    // Update status to completed
    update_status(db, id, "completed", payload.notes.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    // Credit user's balance
    let amount: f64 = transaction.amount.parse().unwrap_or(0.0);
    add_credit(db, transaction.user_id, amount)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}

/// PUT /admin/payments/:id/reject - Reject payment
pub async fn reject_payment(
    Path(id): Path<i32>,
    State(state): State<AppState>,
    Json(payload): Json<ApproveRequest>,
) -> Result<StatusCode, (StatusCode, String)> {
    let db = &state.db;

    // Get transaction
    let _transaction = get_transaction(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "Payment not found".to_string()))?;

    // Update status to rejected
    update_status(db, id, "rejected", payload.notes.as_deref())
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?;

    Ok(StatusCode::OK)
}
