use std::sync::Arc;

use mobc::Connection;
use mobc_redis::{RedisConnectionManager, redis::AsyncCommands};
use sha2::{Digest, Sha256};

use crate::{AppState, errors::RedisConnectError};

async fn connect_to_redis(
    state: Arc<AppState>,
) -> Result<Connection<RedisConnectionManager>, RedisConnectError> {
    match state.redis_pool.get().await {
        Ok(conn) => Ok(conn),
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            Err(RedisConnectError::ConnectionError(e))
        }
    }
}

pub async fn cache_set(
    state: Arc<AppState>,
    key: &str,
    value: &str,
) -> Result<(), RedisConnectError> {
    let mut conn = connect_to_redis(state.clone()).await?;
    conn.set::<_, _, ()>(key, value).await?;
    conn.expire::<_, ()>(key, state.cache_lifetime).await?;
    Ok(())
}

pub async fn cache_get(
    state: Arc<AppState>,
    key: &str,
) -> Result<Option<String>, RedisConnectError> {
    let mut conn = connect_to_redis(state).await?;
    let long_url = conn.get(key).await?;
    Ok(long_url)
}

pub fn get_long_url_key(long_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(long_url);
    hex::encode(hasher.finalize())
}

pub async fn add_short_to_cache(
    state: Arc<AppState>,
    long_url: &str,
    short: &str,
) -> Result<(), RedisConnectError> {
    let mut conn = connect_to_redis(state.clone()).await?;
    let key = get_long_url_key(long_url);
    conn.set::<_, _, ()>(&key, short).await?;
    conn.expire::<_, ()>(key, state.cache_lifetime).await?;
    Ok(())
}

pub async fn get_short_from_cache(
    state: Arc<AppState>,
    long_url: &str,
) -> Result<Option<String>, RedisConnectError> {
    let mut conn = connect_to_redis(state).await?;
    let key = get_long_url_key(long_url);
    let short = conn.get(key).await?;
    Ok(short)
}
