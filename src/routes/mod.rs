use std::sync::Arc;

use axum::Router;
use tower_http::services::ServeDir;

use crate::{
    AppState,
    routes::{pages::get_pages_routes, shorts::get_shorts_routes},
};
use crate::routes::stats::get_stats_routes;

mod pages;
mod shorts;
mod stats;

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(get_shorts_routes())
        .merge(get_pages_routes())
        .nest("/api", get_stats_routes())
        .with_state(state)
        .nest_service("/assets", ServeDir::new("assets"))
}
