use serde_json::Value;

pub async fn upsert_account_credential(
    pool: &sqlx::PgPool,
    account_id: i32,
    credential_type: &str,
    payload: &Value,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO account_credentials (account_id, credential_type, payload)
        VALUES ($1, $2, $3)
        ON CONFLICT (account_id)
        DO UPDATE SET
            credential_type = EXCLUDED.credential_type,
            payload = EXCLUDED.payload,
            updated_at = NOW()
        "#,
    )
    .bind(account_id)
    .bind(credential_type)
    .bind(payload)
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn delete_account_credentials(
    pool: &sqlx::PgPool,
    account_id: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM account_credentials WHERE account_id = $1")
        .bind(account_id)
        .execute(pool)
        .await?;
    Ok(())
}
