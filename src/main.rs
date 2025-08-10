use crate::{config::load_config, routes::get_routes, templates::load_templates};
use sqlx::PgPool;
use std::sync::Arc;
use tera::Tera;

mod config;
mod controllers;
mod db;
mod errors;
mod logs;
mod models;
mod routes;
mod templates;

struct AppState {
    tera: Tera,
    pg_pool: PgPool,
    redis_pool: r2d2::Pool<redis::Client>,
}

#[tokio::main]
async fn main() {
    let config = load_config().await;
    let _guard = logs::init_logs(config.logs);

    tracing::info!("Starting server");

    let tera = load_templates();
    let pg_pool = db::get_pg_pool(config.server.database).await;
    let redis_pool = db::get_redis_pool(config.server.redis);
    let state = Arc::new(AppState {
        tera,
        pg_pool,
        redis_pool,
    });

    let app = get_routes(state);

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", config.server.port.unwrap_or(3000)))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
