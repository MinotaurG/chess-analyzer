use axum::{
    routing::get,
    Router,
};
use tower_http::services::ServeDir;

mod routes;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/games", get(routes::games_list))
        .route("/health", get(routes::health))
        .nest_service("/static", ServeDir::new("crates/web/static"));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://127.0.0.1:3000");

    axum::serve(listener, app).await.unwrap();
}
