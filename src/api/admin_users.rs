use axum::{
    Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, QueryBuilder, Row};

use crate::AppState;
use crate::db::{balances, plans, transactions, user_plans, users};

#[derive(Debug, Serialize)]
pub struct UserListItem {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub plan_name: Option<String>,
    pub balance: String,
    pub active: bool,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct UserListResponse {
    pub items: Vec<UserListItem>,
    pub total: i64,
    pub page: u32,
    pub limit: u32,
}

#[derive(Debug, Serialize)]
pub struct UserDetail {
    pub id: i32,
    pub email: String,
    pub name: Option<String>,
    pub avatar_url: Option<String>,
    pub provider: String,
    pub locale: String,
    pub active: bool,
    pub plan: Option<PlanDetail>,
    pub balance: String,
    pub total_requests: i64,
    pub transactions: Vec<TransactionSummary>,
    pub created_at: String,
}

#[derive(Debug, Serialize)]
pub struct PlanDetail {
    pub id: i32,
    pub name: String,
    pub slug: String,
    pub price_usd: Option<String>,
    pub price_vnd: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct TransactionSummary {
    pub id: i32,
    pub tx_type: String,
    pub amount: String,
    pub currency: String,
    pub status: String,
    pub created_at: String,
}

#[derive(Debug, Deserialize)]
pub struct UserUpdateRequest {
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub plan_id: Option<i32>,
    #[serde(default)]
    pub balance_adjustment: Option<f64>,
}

#[derive(Debug, Deserialize)]
pub struct UserListQuery {
    #[serde(default)]
    pub search: Option<String>,
    #[serde(default)]
    pub plan: Option<String>,
    #[serde(default)]
    pub active: Option<bool>,
    #[serde(default)]
    pub page: u32,
    #[serde(default)]
    pub limit: u32,
}

/// GET /admin/users - List all users with pagination
pub async fn list_users(
    State(state): State<AppState>,
    Query(query): Query<UserListQuery>,
) -> Result<Json<UserListResponse>, (StatusCode, String)> {
    let db = &state.db;

    let page = query.page.max(1);
    let limit = query.limit.max(1).min(100);
    let offset = (page.saturating_sub(1) as i64) * limit as i64;

    let mut select_builder = QueryBuilder::<Postgres>::new(
        r#"SELECT u.id, u.email, u.name, u.avatar_url, u.active, u.created_at,
           COALESCE(b.amount::text, '0'::text) as balance,
           p.name as plan_name
        FROM users u
        LEFT JOIN balances b ON u.id = b.user_id
        LEFT JOIN user_plans up ON u.id = up.user_id AND up.active = true
        LEFT JOIN plans p ON up.plan_id = p.id"#,
    );
    push_user_filters(&mut select_builder, &query);
    select_builder.push(" ORDER BY u.created_at DESC LIMIT ");
    select_builder.push_bind(limit as i64);
    select_builder.push(" OFFSET ");
    select_builder.push_bind(offset);

    let rows = select_builder.build().fetch_all(db).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let mut count_builder = QueryBuilder::<Postgres>::new(
        r#"SELECT COUNT(*)::BIGINT as count
        FROM users u
        LEFT JOIN user_plans up ON u.id = up.user_id AND up.active = true
        LEFT JOIN plans p ON up.plan_id = p.id"#,
    );
    push_user_filters(&mut count_builder, &query);
    let total = count_builder
        .build()
        .fetch_one(db)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .get::<i64, _>("count");

    let users: Vec<UserListItem> = rows
        .into_iter()
        .map(|r| UserListItem {
            id: r.get("id"),
            email: r.get("email"),
            name: r.get("name"),
            avatar_url: r.get("avatar_url"),
            plan_name: r.get("plan_name"),
            balance: r.get("balance"),
            active: r.get("active"),
            created_at: r
                .get::<chrono::DateTime<chrono::Utc>, _>("created_at")
                .to_rfc3339(),
        })
        .collect();

    Ok(Json(UserListResponse {
        items: users,
        total,
        page,
        limit,
    }))
}

fn push_user_filters(builder: &mut QueryBuilder<Postgres>, query: &UserListQuery) {
    let mut has_where = false;

    if let Some(search) = query.search.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        push_user_filter_prefix(builder, &mut has_where);
        let pattern = format!("%{}%", search);
        builder.push("(u.email ILIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR COALESCE(u.name, '') ILIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }

    if let Some(plan) = query.plan.as_deref().map(str::trim).filter(|value| !value.is_empty() && *value != "all") {
        push_user_filter_prefix(builder, &mut has_where);
        builder.push("p.slug = ");
        builder.push_bind(plan.to_string());
    }

    if let Some(active) = query.active {
        push_user_filter_prefix(builder, &mut has_where);
        builder.push("u.active = ");
        builder.push_bind(active);
    }
}

fn push_user_filter_prefix(builder: &mut QueryBuilder<Postgres>, has_where: &mut bool) {
    if *has_where {
        builder.push(" AND ");
    } else {
        builder.push(" WHERE ");
        *has_where = true;
    }
}

/// GET /admin/users/:id - Get user detail
pub async fn get_user_detail(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<Json<UserDetail>, (StatusCode, String)> {
    let db = &state.db;

    // Get user
    let user = users::get_user(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Get balance
    let balance = balances::get_or_create_balance(db, id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Get active plan
    let plan = user_plans::get_active_plan(db, id).await.map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    let plan_detail = if let Some(up) = plan {
        match plans::get_plan(db, up.plan_id).await {
            Ok(Some(p)) => Some(PlanDetail {
                id: p.id,
                name: p.name,
                slug: p.slug,
                price_usd: p.price_usd,
                price_vnd: p.price_vnd,
            }),
            _ => None,
        }
    } else {
        None
    };

    // Get usage stats
    let total_requests = sqlx::query_scalar::<_, i64>(
        r#"SELECT COUNT(*)::bigint FROM usage_logs WHERE user_id = $1"#,
    )
    .bind(id)
    .fetch_one(db)
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Database error: {}", e),
        )
    })?;

    // Get recent transactions
    let transactions = transactions::list_by_user(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .into_iter()
        .take(10)
        .map(|t| TransactionSummary {
            id: t.id,
            tx_type: t.tx_type,
            amount: t.amount,
            currency: t.currency,
            status: t.status,
            created_at: t.created_at.to_rfc3339(),
        })
        .collect();

    Ok(Json(UserDetail {
        id: user.id,
        email: user.email,
        name: user.name.clone(),
        avatar_url: user.avatar_url.clone(),
        provider: user.provider,
        locale: user.locale,
        active: user.active,
        plan: plan_detail,
        balance: balance.amount,
        total_requests,
        transactions,
        created_at: user.created_at.map(|d| d.to_rfc3339()).unwrap_or_default(),
    }))
}

/// PUT /admin/users/:id - Update user
pub async fn update_user(
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(req): Json<UserUpdateRequest>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
    let db = &state.db;

    // Verify user exists
    let _user = users::get_user(db, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("Database error: {}", e),
            )
        })?
        .ok_or((StatusCode::NOT_FOUND, "User not found".to_string()))?;

    // Update active status
    if let Some(active) = req.active {
        sqlx::query("UPDATE users SET active = $1 WHERE id = $2")
            .bind(active)
            .bind(id)
            .execute(db)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    // Update plan
    if let Some(plan_id) = req.plan_id {
        // Verify plan exists
        let _plan = plans::get_plan(db, plan_id)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?
            .ok_or((StatusCode::BAD_REQUEST, "Plan not found".to_string()))?;

        // Assign new plan (this will deactivate old plan)
        user_plans::assign_plan(db, id, plan_id, None)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    // Adjust balance
    if let Some(adjustment) = req.balance_adjustment {
        balances::add_credit(db, id, adjustment)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Database error: {}", e),
                )
            })?;
    }

    Ok(Json(serde_json::json!({ "success": true })))
}
