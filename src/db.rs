use std::env;

use sqlx::PgPool;

pub async fn get_pg_pool() -> sqlx::PgPool {
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing::debug!("Connection string: {}", connection_string);

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Connected to database successfully");

    pool
}
