use crate::{
    AppState,
    routes::{pages::get_pages_routes, shorts::get_shorts_routes},
};
use axum::Router;
use std::sync::Arc;
use tower_http::services::ServeDir;

pub mod pages;
pub mod shorts;

pub fn get_routes(state: Arc<AppState>) -> Router {
    Router::new()
        .merge(get_shorts_routes())
        .merge(get_pages_routes())
        .with_state(state)
        .nest_service("/assets", ServeDir::new("assets"))
}
