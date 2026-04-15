use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub id: i32,
    pub user_id: i32,
    pub amount: String,
    pub updated_at: DateTime<Utc>,
}

/// Get or create balance for a user
pub async fn get_or_create_balance(
    pool: &sqlx::PgPool,
    user_id: i32,
) -> Result<Balance, sqlx::Error> {
    // Try to get existing balance
    let row = sqlx::query(
        r#"
        SELECT id, user_id, amount::text as amount, updated_at
        FROM balances
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        return Ok(Balance {
            id: row.get("id"),
            user_id: row.get("user_id"),
            amount: row.get::<String, _>("amount"),
            updated_at: row
                .get::<Option<DateTime<Utc>>, _>("updated_at")
                .unwrap_or(Utc::now()),
        });
    }

    // Create new balance
    let row = sqlx::query(
        r#"
        INSERT INTO balances (user_id, amount, updated_at)
        VALUES ($1, 0, NOW())
        RETURNING id, user_id, amount::text as amount, updated_at
        "#,
    )
    .bind(user_id)
    .fetch_one(pool)
    .await?;

    Ok(Balance {
        id: row.get("id"),
        user_id: row.get("user_id"),
        amount: row.get::<String, _>("amount"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

/// Get balance for a user
pub async fn get_balance(
    pool: &sqlx::PgPool,
    user_id: i32,
) -> Result<Option<Balance>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        SELECT id, user_id, amount::text as amount, updated_at
        FROM balances
        WHERE user_id = $1
        "#,
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Balance {
        id: r.get("id"),
        user_id: r.get("user_id"),
        amount: r.get::<String, _>("amount"),
        updated_at: r
            .get::<Option<DateTime<Utc>>, _>("updated_at")
            .unwrap_or(Utc::now()),
    }))
}

/// Add credit to user's balance
pub async fn add_credit(
    pool: &sqlx::PgPool,
    user_id: i32,
    amount: f64,
) -> Result<Balance, sqlx::Error> {
    let row = sqlx::query(
        r#"
        INSERT INTO balances (user_id, amount, updated_at)
        VALUES ($1, $2, NOW())
        ON CONFLICT (user_id) DO UPDATE SET
            amount = balances.amount + $2,
            updated_at = NOW()
        RETURNING id, user_id, amount::text as amount, updated_at
        "#,
    )
    .bind(user_id)
    .bind(amount)
    .fetch_one(pool)
    .await?;

    Ok(Balance {
        id: row.get("id"),
        user_id: row.get("user_id"),
        amount: row.get::<String, _>("amount"),
        updated_at: row.get::<DateTime<Utc>, _>("updated_at"),
    })
}

/// Deduct from user's balance
#[allow(dead_code)]
pub async fn deduct_credit(
    pool: &sqlx::PgPool,
    user_id: i32,
    amount: f64,
) -> Result<Option<Balance>, sqlx::Error> {
    let row = sqlx::query(
        r#"
        UPDATE balances
        SET amount = amount - $2,
            updated_at = NOW()
        WHERE user_id = $1 AND amount >= $2
        RETURNING id, user_id, amount::text as amount, updated_at
        "#,
    )
    .bind(user_id)
    .bind(amount)
    .fetch_optional(pool)
    .await?;

    Ok(row.map(|r| Balance {
        id: r.get("id"),
        user_id: r.get("user_id"),
        amount: r.get::<String, _>("amount"),
        updated_at: r
            .get::<Option<DateTime<Utc>>, _>("updated_at")
            .unwrap_or(Utc::now()),
    }))
}
