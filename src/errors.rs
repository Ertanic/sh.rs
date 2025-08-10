use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisConnectError {
    #[error("Failed to connect to Redis: {0}")]
    ConnectionError(#[from] r2d2::Error),
    #[error("Failed to get value from Redis: {0}")]
    GettingValueError(#[from] redis::RedisError),
}