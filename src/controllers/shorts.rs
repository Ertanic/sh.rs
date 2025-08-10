use crate::{AppState, errors::RedisConnectError, models::shorts::NewShortRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use r2d2::PooledConnection;
use redis::Commands;
use small_uid::SmallUid;
use sqlx::Row;
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::select;

fn connect_to_redis(
    state: Arc<AppState>,
) -> Result<PooledConnection<redis::Client>, RedisConnectError> {
    match state.redis_pool.get() {
        Ok(conn) => Ok(conn),
        Err(e) => {
            tracing::error!("Failed to connect to Redis: {}", e);
            Err(RedisConnectError::ConnectionError(e))
        }
    }
}

fn add_long_to_cache(
    state: Arc<AppState>,
    uid: &str,
    long_url: &str,
) -> Result<(), RedisConnectError> {
    let mut conn = connect_to_redis(state)?;
    conn.set::<_, _, ()>(uid, long_url)?;
    Ok(())
}

fn get_long_from_cache(state: Arc<AppState>, uid: &str) -> Result<String, RedisConnectError> {
    let mut conn = connect_to_redis(state)?;
    let long_url = conn.get(uid)?;
    Ok(long_url)
}

fn add_short_to_cache(
    state: Arc<AppState>,
    long_url: &str,
    uid: &str,
) -> Result<(), RedisConnectError> {
    let mut conn = connect_to_redis(state)?;
    conn.set::<_, _, ()>(long_url, uid)?;
    Ok(())
}

fn get_short_from_cache(state: Arc<AppState>, long_url: &str) -> Result<String, RedisConnectError> {
    let mut conn = connect_to_redis(state)?;
    let short = conn.get(long_url)?;
    Ok(short)
}

async fn get_short_from_db(state: Arc<AppState>, long_url: &str) -> Option<String> {
    let (redis_state, rl_url) = (state.clone(), long_url.to_string());
    let redis_result = tokio::spawn(async move { get_short_from_cache(redis_state, &rl_url) });

    let db_result = sqlx::query("SELECT id FROM shorts WHERE long_url = $1")
        .bind(long_url)
        .fetch_one(&state.pg_pool);

    select! {
        Ok(Ok(short)) = redis_result => {
            Some(short)
        },
        Ok(row) = db_result => {
            let short = row.get("id");
            Some(short)
        },
        else => {
            None
        }
    }
}

fn render_result_page(
    name: &str,
    tera: &Tera,
    uid: &str,
) -> axum::http::Response<axum::body::Body> {
    let page_data = {
        let mut context = Context::new();
        context.insert("short", uid);
        context.insert("title", name);
        context
    };

    let page = tera
        .render("new_short/index.html", &page_data)
        .expect("Failed to render template");

    Html(page).into_response()
}

#[tracing::instrument(skip_all)]
pub async fn create_short(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NewShortRequest>,
) -> impl IntoResponse {
    tracing::trace!("Trying to find long URL in cache: {}", params.long_url);
    if let Some(uid) = get_short_from_db(state.clone(), &params.long_url).await {
        tracing::trace!("Found short URL in cache: {}", uid);
        render_result_page(&state.name, &state.tera, &uid).into_response()
    } else {
        tracing::trace!("Creating short URL: {}", params.long_url);

        let uid = SmallUid::new().to_string();
        tracing::trace!("Generated UID: {}", uid);

        let result = sqlx::query("INSERT INTO shorts (id, long_url) VALUES ($1, $2)")
            .bind(&uid)
            .bind(&params.long_url)
            .execute(&state.pg_pool)
            .await;

        match result {
            Ok(_) => {
                tracing::trace!("Short URL was created");

                if let Err(e) = add_long_to_cache(state.clone(), &uid, &params.long_url) {
                    tracing::error!("Failed to add long URL to cache: {}", e);
                }

                if let Err(e) = add_short_to_cache(state.clone(), &params.long_url, &uid) {
                    tracing::error!("Failed to add short URL to cache: {}", e);
                }

                render_result_page(&state.name, &state.tera, &uid).into_response()
            }
            Err(e) => {
                tracing::error!("Failed to create short URL: {}", e);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
        }
    }
}

#[tracing::instrument(skip_all)]
pub async fn goto_long_url(
    Path(uid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::trace!("Getting long URL for UID: {}", uid);

    let db_state = state.clone();
    let db_uid = uid.clone();
    let db_result = sqlx::query("SELECT long_url FROM shorts WHERE id = $1")
        .bind(&db_uid)
        .fetch_one(&db_state.pg_pool);

    let cache_result = tokio::spawn(async move { get_long_from_cache(state, &uid) });

    select! {
        Ok(row) = db_result => {
            let long_url: String = row.get("long_url");
            tracing::trace!("Long URL from database: {}", long_url);

            if let Err(e) = add_long_to_cache(db_state, &db_uid, &long_url) {
                tracing::error!("Failed to add long URL to cache: {}", e);
            }

            Redirect::to(&long_url).into_response()
        },
        Ok(Ok(long_url)) = cache_result => {
            tracing::trace!("Long URL from cache: {}", long_url);

            Redirect::to(&long_url).into_response()
        },
        else => {
            tracing::trace!("UID not found in database or cache");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
