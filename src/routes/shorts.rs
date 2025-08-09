use crate::{AppState, controllers::shorts::create_short};
use axum::{Router, routing::post};
use std::sync::Arc;

pub fn get_shorts_routes() -> Router<Arc<AppState>> {
    Router::new().route("/shorts", post(create_short))
}
