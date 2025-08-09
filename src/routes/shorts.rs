use crate::{
    AppState,
    controllers::shorts::{create_short, goto_long_url},
};
use axum::{
    Router,
    routing::{get, post},
};
use std::sync::Arc;

pub fn get_shorts_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/shorts", post(create_short))
        .route("/{uid}", get(goto_long_url))
}
