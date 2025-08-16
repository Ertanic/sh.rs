use std::sync::Arc;

use axum::{Json, extract::State, http::StatusCode, response::IntoResponse};

use crate::{
    AppState,
    models::stats::{GetShortsGotoStatsModel, GetShortsGotoStatsResponse},
};

pub async fn get_shorts_goto_stats(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let result = sqlx::query_as::<_, GetShortsGotoStatsModel>(
        r#"
        SELECT s.long_url, COUNT(sg.id) AS total
        FROM public.shorts s
        JOIN public.shorts_goto_stats sg ON s.id = sg.short_id
        GROUP BY s.long_url
        ORDER BY total DESC
        LIMIT 10
        "#,
    )
    .fetch_all(&state.pg_pool)
    .await;

    match result {
        Ok(stats) => Json(GetShortsGotoStatsResponse { stats }).into_response(),
        Err(e) => {
            tracing::error!("Failed to get goto stats: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}
