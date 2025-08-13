use mobc::Pool;
use mobc_redis::{RedisConnectionManager, redis::Client};
use sqlx::PgPool;
use std::env;

pub async fn get_pg_pool(connection_string: Option<String>) -> PgPool {
    let connection_string = connection_string
        .unwrap_or_else(|| env::var("DATABASE_URL").expect("DATABASE_URL must be set"));
    tracing::debug!("Connection string: {}", connection_string);

    let pool = PgPool::connect(&connection_string)
        .await
        .expect("Failed to connect to database");
    tracing::info!("Connected to database successfully");

    pool
}

pub fn get_redis_pool(connection_string: Option<String>) -> Pool<RedisConnectionManager> {
    let connection_string =
        connection_string.unwrap_or_else(|| env::var("REDIS_URL").expect("REDIS_URL must be set"));
    tracing::debug!("Connection string: {}", connection_string);

    let client = Client::open(connection_string).expect("Failed to connect to Redis");
    let manager = RedisConnectionManager::new(client);
    let pool = Pool::new(manager);

    tracing::info!("Connected to Redis successfully");

    pool
}
