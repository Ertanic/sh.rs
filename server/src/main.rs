use axum::{Router, routing::get};

mod logs;

#[tokio::main]
async fn main() {
    let _guard = logs::init_logs();

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    let listener = tokio::net::TcpListener::bind(("0.0.0.0", 3000))
        .await
        .expect("Failed to bind port");

    tracing::info!("Listening on http://localhost:3000");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}