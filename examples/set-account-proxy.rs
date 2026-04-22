use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL is required in environment or .env")?;
    let account_id: i32 = env::args()
        .nth(1)
        .ok_or("usage: cargo run --example set-account-proxy -- <account_id> <proxy_id|none>")?
        .parse()?;
    let proxy_arg = env::args()
        .nth(2)
        .ok_or("usage: cargo run --example set-account-proxy -- <account_id> <proxy_id|none>")?;
    let proxy_id = if proxy_arg.eq_ignore_ascii_case("none") {
        None
    } else {
        Some(proxy_arg.parse::<i32>()?)
    };

    let pool = sqlx::PgPool::connect(&database_url).await?;
    sqlx::query(
        r#"
        UPDATE accounts
        SET proxy_id = $2
        WHERE id = $1
        "#,
    )
    .bind(account_id)
    .bind(proxy_id)
    .execute(&pool)
    .await?;

    Ok(())
}
