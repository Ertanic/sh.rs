use crate::{AppState, errors::RedisConnectError, models::shorts::NewShortRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use mobc::Connection;
use mobc_redis::{RedisConnectionManager, redis::AsyncCommands};
use sha2::{Digest, Sha256};
use small_uid::SmallUid;
use sqlx::{PgPool, Row};
use std::sync::Arc;
use tera::{Context, Tera};
use tokio::select;

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

async fn add_long_to_cache(
    state: Arc<AppState>,
    short: &str,
    long_url: &str,
) -> Result<(), RedisConnectError> {
    let mut conn = connect_to_redis(state.clone()).await?;
    conn.set::<_, _, ()>(short, long_url).await?;
    conn.expire::<_, ()>(short, state.cache_lifetime).await?;
    Ok(())
}

async fn get_long_from_cache(
    state: Arc<AppState>,
    short: &str,
) -> Result<Option<String>, RedisConnectError> {
    let mut conn = connect_to_redis(state).await?;
    let long_url = conn.get(short).await?;
    Ok(long_url)
}

fn get_long_url_key(long_url: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(long_url);
    hex::encode(hasher.finalize())
}

async fn add_short_to_cache(
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

async fn get_short_from_cache(
    state: Arc<AppState>,
    long_url: &str,
) -> Result<Option<String>, RedisConnectError> {
    let mut conn = connect_to_redis(state).await?;
    let key = get_long_url_key(long_url);
    let short = conn.get(key).await?;
    Ok(short)
}

async fn get_short_from_db(state: Arc<AppState>, long_url: &str) -> Option<String> {
    let redis_result = get_short_from_cache(state.clone(), long_url);

    let db_result =
        sqlx::query_scalar::<_, String>("SELECT short FROM public.shorts WHERE long_url = $1")
            .bind(long_url)
            .fetch_optional(&state.pg_pool);

    select! {
        Ok(Some(short)) = redis_result => {
            Some(short)
        },
        Ok(Some(short)) = db_result => {
            let (cache_short, long_url) = (short.clone(), long_url.to_string());
            tokio::spawn(async move {
                if let Err(e) = add_short_to_cache(state.clone(), &long_url, &cache_short).await {
                    tracing::error!("Failed to add short URL to cache: {}", e);
                }
            });
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

    let page = match tera.render("new_short/index.html", &page_data) {
        Ok(page) => page,
        Err(_) => {
            tracing::error!("Failed to render page");
            return (StatusCode::INTERNAL_SERVER_ERROR, "Failed to render page").into_response();
        }
    };

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

        let result = sqlx::query("INSERT INTO public.shorts (short, long_url) VALUES ($1, $2)")
            .bind(&uid)
            .bind(&params.long_url)
            .execute(&state.pg_pool)
            .await;

        match result {
            Ok(_) => {
                tracing::trace!("Short URL was created");

                if let Err(e) = add_long_to_cache(state.clone(), &uid, &params.long_url).await {
                    tracing::error!("Failed to add long URL to cache: {}", e);
                }

                if let Err(e) = add_short_to_cache(state.clone(), &params.long_url, &uid).await {
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
fn add_goto_stat(state: Arc<AppState>, short: &str) {
    let short = short.to_string();
    tokio::spawn(async move {
        match sqlx::query("INSERT INTO public.shorts_goto_stats (short_id) SELECT id FROM public.shorts WHERE short = $1")
            .bind(&short)
            .execute(&state.pg_pool)
        .await
        {
            Ok(_) => tracing::trace!("Incremented stat for UID: {}", short),
            Err(e) => tracing::error!("Failed to increment stat: {}", e),
        }
    });
}

#[tracing::instrument(skip_all)]
pub async fn goto_long_url(
    Path(short): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::trace!("Getting long URL for UID: {}", short);

    let db_result =
        sqlx::query_scalar::<_, String>("SELECT long_url FROM public.shorts WHERE short = $1")
            .bind(&short)
            .fetch_optional(&state.pg_pool);

    let cache_result = get_long_from_cache(state.clone(), &short);

    select! {
        Ok(Some(long_url)) = db_result => {
            tracing::trace!("Long URL from database: {}", long_url);

            if let Err(e) = add_long_to_cache(state.clone(), &short, &long_url).await {
                tracing::error!("Failed to add long URL to cache: {}", e);
            }

            add_goto_stat(state, &short);

            Redirect::to(&long_url).into_response()
        },
        Ok(Some(long_url)) = cache_result => {
            tracing::trace!("Long URL from cache: {}", long_url);

            add_goto_stat(state, &short);

            Redirect::to(&long_url).into_response()
        },
        else => {
            tracing::trace!("UID not found in database or cache");
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
