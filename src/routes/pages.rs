use crate::{AppState, controllers::pages::main_page};
use axum::{Router, routing::get};
use std::sync::Arc;

pub fn get_pages_routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(main_page))
}