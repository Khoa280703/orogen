use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required in environment or .env")?;
    let account_id: i32 = env::args()
        .nth(1)
        .ok_or("usage: cargo run --example set-account-routing-healthy -- <account_id>")?
        .parse()?;

    let pool = sqlx::PgPool::connect(&database_url).await?;

    let updated = sqlx::query(
        r#"
        UPDATE accounts
        SET
            session_status = 'healthy',
            session_error = NULL,
            session_checked_at = NOW(),
            routing_state = 'healthy',
            cooldown_until = NULL,
            last_routing_error = NULL,
            rate_limit_streak = 0,
            auth_failure_streak = 0,
            refresh_failure_streak = 0
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .execute(&pool)
    .await?;

    println!("updated_rows={}", updated.rows_affected());
    let row = sqlx::query_as::<_, (i32, String, Option<String>, String)>(
        r#"
        SELECT id, name, session_status, routing_state
        FROM accounts
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .fetch_one(&pool)
    .await?;
    println!(
        "account_id={} name={} session_status={} routing_state={}",
        row.0,
        row.1,
        row.2.unwrap_or_else(|| "null".to_string()),
        row.3
    );
    Ok(())
}
