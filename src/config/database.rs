use sqlx::postgres::PgPool;

pub async fn connect() -> PgPool {
    let database_url =
        std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");

    let pool = PgPool::connect(&database_url)
        .await
        .expect("Failed to connect to database");

    tracing::info!("connected to database");
    pool
}
