use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{AppState, controllers::pages::main_page};

pub fn get_pages_routes() -> Router<Arc<AppState>> {
    Router::new().route("/", get(main_page))
}
