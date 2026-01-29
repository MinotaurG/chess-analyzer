use axum::{
    routing::{get, post},
    Router,
};
use std::sync::{Arc, Mutex};
use tower_http::services::ServeDir;

use chess_analyzer_core::Database;

mod routes;

pub struct AppState {
    pub db: Mutex<Database>,
    pub username: Mutex<Option<String>>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let db = Database::open("chess_analyzer.db").expect("Failed to open database");

    let state = Arc::new(AppState {
        db: Mutex::new(db),
        username: Mutex::new(None),
    });

    let app = Router::new()
        .route("/", get(routes::index))
        .route("/games", get(routes::games_list))
        .route("/patterns", get(routes::patterns_list))
        .route("/sync", post(routes::sync_games))
        .route("/analyze", get(routes::analyze_games))
        .route("/health", get(routes::health))
        .route("/training/coordinates", get(routes::training::coordinates_drill))
        .route("/training/visualization", get(routes::training::visualization_drill))
        .route("/training/openings", get(routes::training::openings_trainer))
        .nest_service("/static", ServeDir::new("crates/web/static"))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();

    println!("Server running at http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
