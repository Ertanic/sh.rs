use r2d2::Pool;
use redis::Client;
use sqlx::PgPool;
use std::env;

pub async fn get_pg_pool() -> PgPool {
    let connection_string = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    tracing::debug!("Connection string: {}", connection_string);

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Connected to database successfully");

    pool
}

pub fn get_redis_pool() -> Pool<Client> {
    let connection_string = env::var("REDIS_URL").expect("REDIS_URL must be set");
    tracing::debug!("Connection string: {}", connection_string);

    let client = Client::open(connection_string).expect("Failed to connect to Redis");
    let pool = Pool::builder()
        .build(client)
        .expect("Failed to create Redis pool");

    tracing::info!("Connected to Redis successfully");

    pool
}
