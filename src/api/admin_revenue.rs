use axum::{Json, extract::State, http::StatusCode};
use serde::Serialize;
use sqlx::Row;

use crate::AppState;

#[derive(Debug, Serialize)]
pub struct RevenueOverview {
    pub total_revenue: String,
    pub today_revenue: String,
    pub week_revenue: String,
    pub month_revenue: String,
    pub active_subscribers: i64,
    pub top_users: Vec<TopUserSpend>,
}

#[derive(Debug, Serialize)]
pub struct TopUserSpend {
    pub user_id: i32,
    pub email: String,
    pub total_spent: String,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize)]
pub struct RevenueByDay {
    pub date: String,
    pub revenue: String,
    pub transaction_count: i64,
}

#[derive(Debug, Serialize)]
pub struct RevenueByMethod {
    pub method: String,
    pub revenue: String,
    pub transaction_count: i64,
}

/// GET /admin/revenue/overview - Get revenue statistics
pub async fn get_revenue_overview(
    State(state): State<AppState>,
) -> Result<Json<RevenueOverview>, (StatusCode, String)> {
    let db = &state.db;

    // Total revenue (all completed transactions)
    let total_revenue = sqlx::query_scalar::<_, String>(
        r#"SELECT COALESCE(SUM(amount)::text, '0') FROM transactions WHERE status = 'completed'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Today's revenue
    let today_revenue = sqlx::query_scalar::<_, String>(
        r#"SELECT COALESCE(SUM(amount)::text, '0') FROM transactions
           WHERE status = 'completed' AND created_at >= NOW() - INTERVAL '1 day'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Week revenue
    let week_revenue = sqlx::query_scalar::<_, String>(
        r#"SELECT COALESCE(SUM(amount)::text, '0') FROM transactions
           WHERE status = 'completed' AND created_at >= NOW() - INTERVAL '7 days'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Month revenue
    let month_revenue = sqlx::query_scalar::<_, String>(
        r#"SELECT COALESCE(SUM(amount)::text, '0') FROM transactions
           WHERE status = 'completed' AND created_at >= NOW() - INTERVAL '30 days'"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Active subscribers (users with active plan and balance > 0)
    let active_subscribers = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(DISTINCT u.id) FROM users u
           INNER JOIN user_plans up ON u.id = up.user_id AND up.active = true
           INNER JOIN balances b ON u.id = b.user_id
           WHERE b.amount > 0 AND u.active = true"#,
    )
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Top users by spend
    let top_users_rows = sqlx::query(
        r#"SELECT u.id as user_id, u.email,
                  COALESCE(SUM(t.amount), 0) as total_spent,
                  COUNT(t.id) as transaction_count
           FROM users u
           INNER JOIN transactions t ON u.id = t.user_id AND t.status = 'completed'
           GROUP BY u.id, u.email
           ORDER BY total_spent DESC
           LIMIT 10"#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let top_users: Vec<TopUserSpend> = top_users_rows
        .into_iter()
        .map(|r| TopUserSpend {
            user_id: r.get("user_id"),
            email: r.get("email"),
            total_spent: r.get("total_spent"),
            transaction_count: r.get("transaction_count"),
        })
        .collect();

    Ok(Json(RevenueOverview {
        total_revenue,
        today_revenue,
        week_revenue,
        month_revenue,
        active_subscribers,
        top_users,
    }))
}

/// GET /admin/revenue/daily - Get daily revenue for last 30 days
pub async fn get_daily_revenue(
    State(state): State<AppState>,
) -> Result<Json<Vec<RevenueByDay>>, (StatusCode, String)> {
    let db = &state.db;

    let rows = sqlx::query(
        r#"SELECT date_trunc('day', created_at)::date as date,
                  COALESCE(SUM(amount), 0)::text as revenue,
                  COUNT(id)::bigint as transaction_count
           FROM transactions
           WHERE status = 'completed' AND created_at >= NOW() - INTERVAL '30 days'
           GROUP BY date
           ORDER BY date DESC"#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let revenue: Vec<RevenueByDay> = rows
        .into_iter()
        .map(|r| RevenueByDay {
            date: r.get("date"),
            revenue: r.get("revenue"),
            transaction_count: r.get("transaction_count"),
        })
        .collect();

    Ok(Json(revenue))
}

/// GET /admin/revenue/methods - Get revenue by payment method
pub async fn get_revenue_by_method(
    State(state): State<AppState>,
) -> Result<Json<Vec<RevenueByMethod>>, (StatusCode, String)> {
    let db = &state.db;

    let rows = sqlx::query(
        r#"SELECT COALESCE(method, 'unknown') as method,
                  COALESCE(SUM(amount), 0)::text as revenue,
                  COUNT(id)::bigint as transaction_count
           FROM transactions
           WHERE status = 'completed'
           GROUP BY method
           ORDER BY revenue DESC"#,
    )
    .fetch_all(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let revenue: Vec<RevenueByMethod> = rows
        .into_iter()
        .map(|r| RevenueByMethod {
            method: r.get("method"),
            revenue: r.get("revenue"),
            transaction_count: r.get("transaction_count"),
        })
        .collect();

    Ok(Json(revenue))
}
