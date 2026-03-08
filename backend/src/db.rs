use sqlx::PgPool;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = PgPool::connect(database_url).await?;
    Ok(pool)
}

pub async fn migrate(pool: &PgPool) -> anyhow::Result<()> {
    sqlx::query(
        r#"
            CREATE TABLE IF NOT EXISTS videos (
                id          UUID PRIMARY KEY,
                filename    TEXT NOT NULL,
                size_bytes  BIGINT,
                mime_type   TEXT,
                r2_key      TEXT NOT NULL,
                status      TEXT NOT NULL DEFAULT 'pending',
                created_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
        "#,
    )
    .execute(pool)
    .await?;

    tracing::info!("Database migration applied");
    Ok(())
}
