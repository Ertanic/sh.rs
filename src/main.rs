use crate::templates::load_templates;
use axum::{Extension, Router, response::Html, routing::get};
use tera::{Context, Tera};

mod logs;
mod models;
mod templates;

#[tokio::main]
async fn main() {
    let _guard = logs::init_logs();

    tracing::info!("Starting server");

    let tera = load_templates();

    let app = Router::new().route(
        "/",
        get(|Extension(tera): Extension<Tera>| async move {
            let mut context = Context::new();
            context.insert("title", "Home");
            Html(tera.render("index.html", &context).unwrap())
        })
        .layer(Extension(tera)),
    );

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}
