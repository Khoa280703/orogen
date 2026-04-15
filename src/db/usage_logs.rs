use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use sqlx::{FromRow, Postgres, QueryBuilder, Row};

#[derive(Debug, Clone, FromRow)]
pub struct UsageLog {
    pub id: Option<i64>,
    pub api_key_id: Option<i32>,
    pub account_id: Option<i32>,
    pub model: Option<String>,
    pub status: Option<String>,
    pub latency_ms: Option<i32>,
    pub created_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsageLogWithUser {
    pub id: i64,
    pub api_key_id: Option<i32>,
    pub account_id: Option<i32>,
    pub user_id: Option<i32>,
    pub model: String,
    pub status_code: i32,
    pub latency_ms: i32,
    pub created_at: DateTime<Utc>,
}

impl Serialize for UsageLog {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        use serde::ser::SerializeStruct;
        let mut state = serializer.serialize_struct("UsageLog", 7)?;
        state.serialize_field("id", &self.id)?;
        state.serialize_field("api_key_id", &self.api_key_id)?;
        state.serialize_field("account_id", &self.account_id)?;
        state.serialize_field("model", &self.model)?;
        state.serialize_field("status", &self.status)?;
        state.serialize_field("latency_ms", &self.latency_ms)?;
        state.serialize_field("created_at", &self.created_at.map(|d| d.to_rfc3339()))?;
        state.end()
    }
}

#[derive(Debug, Clone, FromRow, Serialize)]
pub struct DailyUsage {
    pub day: Option<String>,
    pub total: Option<i64>,
    pub success: Option<i64>,
}

#[derive(Debug, Clone, FromRow)]
pub struct CountResult {
    pub count: Option<i64>,
}

/// Log a request
pub async fn log_request(
    pool: &sqlx::PgPool,
    api_key_id: Option<i32>,
    user_id: Option<i32>,
    account_id: Option<i32>,
    model: Option<&str>,
    request_kind: Option<&str>,
    status: &str,
    latency_ms: i32,
) -> Result<i64, sqlx::Error> {
    let result = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO usage_logs (api_key_id, user_id, account_id, model, request_kind, status, latency_ms)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        RETURNING id
        "#,
    )
    .bind(api_key_id)
    .bind(user_id)
    .bind(account_id)
    .bind(model)
    .bind(request_kind)
    .bind(status)
    .bind(latency_ms)
    .fetch_one(pool)
    .await?;

    Ok(result)
}

/// Get usage logs with pagination
pub async fn get_usage_logs(
    pool: &sqlx::PgPool,
    offset: i64,
    limit: i64,
    search: Option<&str>,
    status: Option<&str>,
    model: Option<&str>,
) -> Result<Vec<UsageLog>, sqlx::Error> {
    let mut builder = QueryBuilder::<Postgres>::new(
        "SELECT id, api_key_id, account_id, model, status, latency_ms, created_at FROM usage_logs",
    );
    push_usage_log_filters(&mut builder, search, status, model);
    builder.push(" ORDER BY created_at DESC LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind(offset);

    builder
        .build_query_as::<UsageLog>()
        .fetch_all(pool)
        .await
}

/// Get total count of usage logs
pub async fn get_usage_log_count(
    pool: &sqlx::PgPool,
    search: Option<&str>,
    status: Option<&str>,
    model: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let mut builder =
        QueryBuilder::<Postgres>::new("SELECT COUNT(*)::BIGINT as count FROM usage_logs");
    push_usage_log_filters(&mut builder, search, status, model);

    let row = builder.build().fetch_one(pool).await?;
    Ok(row.get::<i64, _>("count"))
}

fn push_usage_log_filters(
    builder: &mut QueryBuilder<Postgres>,
    search: Option<&str>,
    status: Option<&str>,
    model: Option<&str>,
) {
    let mut has_where = false;

    if let Some(search_value) = search.map(str::trim).filter(|value| !value.is_empty()) {
        let pattern = format!("%{}%", search_value);
        push_filter_prefix(builder, &mut has_where);
        builder.push(
            "(COALESCE(model, '') ILIKE ",
        );
        builder.push_bind(pattern.clone());
        builder.push(" OR COALESCE(status, '') ILIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR COALESCE(account_id::text, '') ILIKE ");
        builder.push_bind(pattern.clone());
        builder.push(" OR COALESCE(api_key_id::text, '') ILIKE ");
        builder.push_bind(pattern);
        builder.push(")");
    }

    if let Some(status_value) = status
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
    {
        push_filter_prefix(builder, &mut has_where);
        builder.push("status = ");
        builder.push_bind(status_value.to_string());
    }

    if let Some(model_value) = model
        .map(str::trim)
        .filter(|value| !value.is_empty() && *value != "all")
    {
        push_filter_prefix(builder, &mut has_where);
        builder.push("model = ");
        builder.push_bind(model_value.to_string());
    }
}

fn push_filter_prefix(builder: &mut QueryBuilder<Postgres>, has_where: &mut bool) {
    if *has_where {
        builder.push(" AND ");
    } else {
        builder.push(" WHERE ");
        *has_where = true;
    }
}

/// Get stats overview (total accounts, active, requests today, errors today)
pub async fn get_stats_overview(pool: &sqlx::PgPool) -> Result<(i64, i64, i64, i64), sqlx::Error> {
    let total_accounts = sqlx::query_as!(
        CountResult,
        r#"SELECT COUNT(*)::BIGINT as count FROM accounts"#
    )
    .fetch_one(pool)
    .await?;

    let active_accounts = sqlx::query_as!(
        CountResult,
        r#"SELECT COUNT(*)::BIGINT as count FROM accounts WHERE active = true"#
    )
    .fetch_one(pool)
    .await?;

    let requests_today = sqlx::query_as!(
        CountResult,
        r#"
        SELECT COUNT(*)::BIGINT as count FROM usage_logs
        WHERE created_at >= CURRENT_DATE
        "#
    )
    .fetch_one(pool)
    .await?;

    let errors_today = sqlx::query_as!(
        CountResult,
        r#"
        SELECT COUNT(*)::BIGINT as count FROM usage_logs
        WHERE created_at >= CURRENT_DATE AND status != 'success'
        "#
    )
    .fetch_one(pool)
    .await?;

    Ok((
        total_accounts.count.unwrap_or(0),
        active_accounts.count.unwrap_or(0),
        requests_today.count.unwrap_or(0),
        errors_today.count.unwrap_or(0),
    ))
}

/// Get daily usage breakdown for last N days
pub async fn get_daily_usage(
    pool: &sqlx::PgPool,
    days: i32,
) -> Result<Vec<DailyUsage>, sqlx::Error> {
    sqlx::query_as!(
        DailyUsage,
        r#"
        SELECT
            DATE(created_at)::TEXT as day,
            COUNT(*)::BIGINT as total,
            COUNT(CASE WHEN status = 'success' THEN 1 END)::BIGINT as success
        FROM usage_logs
        WHERE created_at >= CURRENT_DATE - ($1::INTEGER || ' days')::INTERVAL
        GROUP BY DATE(created_at)
        ORDER BY day DESC
        "#,
        days
    )
    .fetch_all(pool)
    .await
}

/// List usage logs by user
pub async fn list_by_user(
    pool: &sqlx::PgPool,
    user_id: i32,
    since: Option<NaiveDateTime>,
) -> Result<Vec<UsageLogWithUser>, sqlx::Error> {
    let rows = sqlx::query(
        r#"
        SELECT ul.id, ul.api_key_id, ul.account_id, COALESCE(ul.user_id, ak.user_id) as user_id, ul.model,
               ul.status as status, ul.latency_ms, ul.created_at
        FROM usage_logs ul
        LEFT JOIN api_keys ak ON ul.api_key_id = ak.id
        WHERE COALESCE(ul.user_id, ak.user_id) = $1
          AND ($2::timestamp IS NULL OR ul.created_at >= $2)
        ORDER BY ul.created_at DESC
        LIMIT 100
        "#,
    )
    .bind(user_id)
    .bind(since)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| UsageLogWithUser {
            id: r.get("id"),
            api_key_id: r.get("api_key_id"),
            account_id: r.get("account_id"),
            user_id: r.get("user_id"),
            model: r.get::<Option<String>, _>("model").unwrap_or_default(),
            status_code: r.get::<String, _>("status").parse().unwrap_or(200),
            latency_ms: r.get::<Option<i32>, _>("latency_ms").unwrap_or(0),
            created_at: r
                .get::<Option<DateTime<Utc>>, _>("created_at")
                .unwrap_or(Utc::now()),
        })
        .collect::<Vec<_>>())
}

/// Count usage by user for today
pub async fn count_today_by_user(pool: &sqlx::PgPool, user_id: i32) -> Result<i64, sqlx::Error> {
    count_today_by_user_scope(pool, user_id, None, None).await
}

pub async fn count_today_by_user_scope(
    pool: &sqlx::PgPool,
    user_id: i32,
    request_kind: Option<&str>,
    model: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let today = Utc::now().date_naive();

    let result: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM usage_logs
        WHERE user_id = $1
          AND DATE(created_at) = $2
          AND ($3::TEXT IS NULL OR request_kind = $3)
          AND ($4::TEXT IS NULL OR model = $4)
        "#,
    )
    .bind(user_id)
    .bind(today)
    .bind(request_kind)
    .bind(model)
    .fetch_one(pool)
    .await?;

    Ok(result.unwrap_or(0))
}

pub async fn count_today_by_api_key(
    pool: &sqlx::PgPool,
    api_key_id: i32,
) -> Result<i64, sqlx::Error> {
    count_today_by_api_key_scope(pool, api_key_id, None, None).await
}

pub async fn count_today_by_api_key_scope(
    pool: &sqlx::PgPool,
    api_key_id: i32,
    request_kind: Option<&str>,
    model: Option<&str>,
) -> Result<i64, sqlx::Error> {
    let today = Utc::now().date_naive();

    let result: Option<i64> = sqlx::query_scalar(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM usage_logs
        WHERE api_key_id = $1
          AND DATE(created_at) = $2
          AND ($3::TEXT IS NULL OR request_kind = $3)
          AND ($4::TEXT IS NULL OR model = $4)
        "#,
    )
    .bind(api_key_id)
    .bind(today)
    .bind(request_kind)
    .bind(model)
    .fetch_one(pool)
    .await?;

    Ok(result.unwrap_or(0))
}
