use sqlx::SqlitePool;

/// Get a config value by key.
pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT value FROM app_config WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await?;
    Ok(row.map(|(v,)| v))
}

/// Set a config value (upsert).
pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO app_config (key, value, created_at) VALUES (?, ?, CURRENT_TIMESTAMP)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
    )
    .bind(key)
    .bind(value)
    .execute(pool)
    .await?;
    Ok(())
}
