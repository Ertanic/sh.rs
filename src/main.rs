use crate::{routes::{pages::get_pages_routes, shorts::get_shorts_routes}, templates::load_templates};
use axum::Router;
use sqlx::PgPool;
use std::sync::Arc;
use tera::Tera;

mod controllers;
mod db;
mod logs;
mod models;
mod routes;
mod templates;

struct AppState {
    tera: Tera,
    pg_pool: PgPool,
}

#[tokio::main]
async fn main() {
    let _guard = logs::init_logs();

    tracing::info!("Starting server");

    let tera = load_templates();
    let pg_pool = db::get_pg_pool().await;
    let state = Arc::new(AppState { tera, pg_pool });

    let app = Router::new()
        .merge(get_pages_routes())
        .merge(get_shorts_routes())
        .with_state(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
