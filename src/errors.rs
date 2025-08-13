use mobc_redis::redis::RedisError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RedisConnectError {
    #[error("Failed to connect to Redis: {0}")]
    ConnectionError(#[from] mobc::Error<RedisError>),
    #[error("Failed to get value from Redis: {0}")]
    GettingValueError(#[from] RedisError),
}
