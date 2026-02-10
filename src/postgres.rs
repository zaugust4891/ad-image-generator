use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use anyhow::Result;

pub async fn connect() -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&std::env::var("DATABASE_URL")?)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("database connected and migrations applied");
    Ok(pool)
}
