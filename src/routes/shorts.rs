use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{
    AppState,
    controllers::shorts::{create_short, goto_long_url},
};

pub fn get_shorts_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/shorts", get(create_short))
        .route("/{uid}", get(goto_long_url))
}
