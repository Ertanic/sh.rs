use crate::{AppState, models::shorts::NewShortRequest};
use axum::{Form, extract::State, http::StatusCode, response::IntoResponse};
use small_uid::SmallUid;
use std::sync::Arc;

pub async fn create_short(
    State(state): State<Arc<AppState>>,
    Form(form): Form<NewShortRequest>,
) -> impl IntoResponse {
    tracing::trace!("Creating short URL: {}", form.long_url);

    let uid = SmallUid::new().to_string();
    tracing::trace!("Generated UID: {}", uid);

    let result = sqlx::query("INSERT INTO shorts (id, long_url) VALUES ($1, $2)")
        .bind(&uid)
        .bind(form.long_url)
        .execute(&state.pg_pool)
        .await;

    match result {
        Ok(_) => {
            tracing::trace!("Short URL was created");
            (StatusCode::OK, format!("Short URL created: {uid}")).into_response()
        }
        Err(e) => {
            tracing::error!("Failed to create short URL: {}", e);
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response()
        }
    }
}
