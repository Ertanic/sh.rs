use crate::{AppState, models::shorts::NewShortRequest};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
};
use small_uid::SmallUid;
use sqlx::Row;
use std::sync::Arc;
use tera::Context;

pub async fn create_short(
    State(state): State<Arc<AppState>>,
    Query(params): Query<NewShortRequest>,
) -> impl IntoResponse {
    tracing::trace!("Creating short URL: {}", params.long_url);

    let uid = SmallUid::new().to_string();
    tracing::trace!("Generated UID: {}", uid);

    let result = sqlx::query("INSERT INTO shorts (id, long_url) VALUES ($1, $2)")
        .bind(&uid)
        .bind(params.long_url)
        .execute(&state.pg_pool)
        .await;

    match result {
        Ok(_) => {
            tracing::trace!("Short URL was created");

            let page_data = {
                let mut context = Context::new();
                context.insert("short", &uid);
                context.insert("title", "sh.rs");
                context
            };

            let page = state
                .tera
                .render("new_short/index.html", &page_data)
                .expect("Failed to render template");

            Html(page).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create short URL: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

pub async fn goto_long_url(
    Path(uid): Path<String>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    tracing::trace!("Getting long URL for UID: {}", uid);

    let result = sqlx::query("SELECT long_url FROM shorts WHERE id = $1")
        .bind(uid)
        .fetch_one(&state.pg_pool)
        .await;

    match result {
        Ok(row) => {
            let long_url = row.get::<String, _>(0);
            tracing::trace!("Redirecting to long URL: {}", long_url);
            Redirect::to(&long_url).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to get long URL: {}", e);
            StatusCode::NOT_FOUND.into_response()
        }
    }
}
