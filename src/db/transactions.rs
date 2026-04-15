use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: i32,
    pub user_id: i32,
    pub tx_type: String,
    pub method: Option<String>,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub reference: Option<String>,
    pub proof_url: Option<String>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// List transactions by user
pub async fn list_by_user(
    pool: &sqlx::PgPool,
    user_id: i32,
) -> Result<Vec<Transaction>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT id, user_id, type, method, amount::text as amount, currency, status,
               reference, proof_url, notes, created_at, updated_at
        FROM transactions
        WHERE user_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Transaction {
            id: r.get("id"),
            user_id: r.get("user_id"),
            tx_type: r.get("type"),
            method: r.get("method"),
            amount: r.get("amount"),
            currency: r.get("currency"),
            status: r.get("status"),
            reference: r.get("reference"),
            proof_url: r.get("proof_url"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect::<Vec<_>>())
}

/// List pending manual topups for admin
pub async fn list_pending_manual(pool: &sqlx::PgPool) -> Result<Vec<Transaction>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT t.id, t.user_id, t.type, t.method, t.amount::text as amount, t.currency, t.status,
               t.reference, t.proof_url, t.notes, t.created_at, t.updated_at,
               u.email, u.name
        FROM transactions t
        LEFT JOIN users u ON t.user_id = u.id
        WHERE t.method = 'manual' AND t.status = 'pending'
        ORDER BY t.created_at DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| Transaction {
            id: r.get("id"),
            user_id: r.get("user_id"),
            tx_type: r.get("type"),
            method: r.get("method"),
            amount: r.get("amount"),
            currency: r.get("currency"),
            status: r.get("status"),
            reference: r.get("reference"),
            proof_url: r.get("proof_url"),
            notes: r.get("notes"),
            created_at: r.get("created_at"),
            updated_at: r.get("updated_at"),
        })
        .collect::<Vec<_>>())
}

/// Create a new transaction
pub async fn create_transaction(
    pool: &sqlx::PgPool,
    user_id: i32,
    tx_type: &str,
    method: Option<&str>,
    amount: f64,
    currency: &str,
    status: &str,
    reference: Option<&str>,
    proof_url: Option<&str>,
) -> Result<Transaction, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO transactions (user_id, type, method, amount, currency, status, reference, proof_url)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        RETURNING id, user_id, type, method, amount::text as amount, currency, status,
                  reference, proof_url, notes, created_at, updated_at
        "#,
    )
    .bind(user_id)
    .bind(tx_type)
    .bind(method)
    .bind(amount)
    .bind(currency)
    .bind(status)
    .bind(reference)
    .bind(proof_url)
    .fetch_one(pool)
    .await?;

    Ok(Transaction {
        id: row.get("id"),
        user_id: row.get("user_id"),
        tx_type: row.get("type"),
        method: row.get("method"),
        amount: row.get("amount"),
        currency: row.get("currency"),
        status: row.get("status"),
        reference: row.get("reference"),
        proof_url: row.get("proof_url"),
        notes: row.get("notes"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

/// Update transaction status
pub async fn update_status(
    pool: &sqlx::PgPool,
    id: i32,
    status: &str,
    notes: Option<&str>,
) -> Result<bool, sqlx::Error> {
    let query = if notes.is_some() {
        r#"UPDATE transactions SET status = $1, notes = $2, updated_at = NOW() WHERE id = $3"#
    } else {
        r#"UPDATE transactions SET status = $1, updated_at = NOW() WHERE id = $3"#
    };

    let result = if notes.is_some() {
        sqlx::query(query)
            .bind(status)
            .bind(notes)
            .bind(id)
            .execute(pool)
            .await?
    } else {
        sqlx::query(query)
            .bind(status)
            .bind(id)
            .execute(pool)
            .await?
    };

    Ok(result.rows_affected() > 0)
}

/// Get transaction by ID
pub async fn get_transaction(
    pool: &sqlx::PgPool,
    id: i32,
) -> Result<Option<Transaction>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, type, method, amount::text as amount, currency, status,
               reference, proof_url, notes, created_at, updated_at
        FROM transactions
        WHERE id = $1
        "#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Transaction {
        id: r.get("id"),
        user_id: r.get("user_id"),
        tx_type: r.get("type"),
        method: r.get("method"),
        amount: r.get("amount"),
        currency: r.get("currency"),
        status: r.get("status"),
        reference: r.get("reference"),
        proof_url: r.get("proof_url"),
        notes: r.get("notes"),
        created_at: r.get("created_at"),
        updated_at: r.get("updated_at"),
    }))
}
