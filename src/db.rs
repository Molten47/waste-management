use sqlx::{PgPool, postgres::PgPoolOptions};

pub async fn connect() -> Result<PgPool, Box<dyn std::error::Error>> {
    let url = std::env::var("DATABASE_URL")?;

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(30))
        .connect(&url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(pool)
}
