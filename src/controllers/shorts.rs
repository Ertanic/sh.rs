use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use small_uid::SmallUid;
use tera::{Context, Tera};
use tokio::select;

use crate::{
    AppState,
    models::shorts::NewShortRequest,
    repos::{
        cache::{add_short_to_cache, cache_get, cache_set},
        db::{add_goto_stat, get_short_from_db},
    },
};

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

                if let Err(e) = cache_set(state.clone(), &uid, &params.long_url).await {
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
pub async fn goto_long_url(
    Path(short): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::trace!("Getting long URL for UID: {}", short);

    let db_result =
        sqlx::query_scalar::<_, String>("SELECT long_url FROM public.shorts WHERE short = $1")
            .bind(&short)
            .fetch_optional(&state.pg_pool);

    let cache_result = cache_get(state.clone(), &short);

    select! {
        Ok(Some(long_url)) = db_result => {
            tracing::trace!("Long URL from database: {}", long_url);

            if let Err(e) = cache_set(state.clone(), &short, &long_url).await {
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
