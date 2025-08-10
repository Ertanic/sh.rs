use crate::{routes::get_routes, templates::load_templates};
use sqlx::PgPool;
use std::sync::Arc;
use tera::Tera;

mod controllers;
mod db;
mod logs;
mod models;
mod routes;
mod templates;
pub mod errors;

struct AppState {
    tera: Tera,
    pg_pool: PgPool,
    redis_pool: r2d2::Pool<redis::Client>,
}

#[tokio::main]
async fn main() {
    let _guard = logs::init_logs();

    tracing::info!("Starting server");

    let tera = load_templates();
    let pg_pool = db::get_pg_pool().await;
    let redis_pool = db::get_redis_pool();
    let state = Arc::new(AppState {
        tera,
        pg_pool,
        redis_pool,
    });

    let app = get_routes(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
