use std::sync::Arc;

use axum::{Router, routing::get};

use crate::{AppState, controllers::stats::get_shorts_goto_stats};

pub fn get_stats_routes() -> Router<Arc<AppState>> {
    Router::new().route("/shorts/goto", get(get_shorts_goto_stats))
}
