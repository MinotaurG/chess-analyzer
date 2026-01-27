use askama::Template;
use axum::response::{Html, IntoResponse};

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub games_count: u32,
    pub patterns_found: u32,
}

#[derive(Template)]
#[template(path = "games.html")]
pub struct GamesTemplate {
    pub title: String,
    pub games: Vec<GameRow>,
}

pub struct GameRow {
    pub id: i64,
    pub white: String,
    pub black: String,
    pub result: String,
    pub date: String,
    pub moves: u32,
}

pub async fn index() -> impl IntoResponse {
    let template = IndexTemplate {
        title: "Chess Analyzer".to_string(),
        games_count: 0,
        patterns_found: 0,
    };
    Html(template.render().unwrap())
}

pub async fn games_list() -> impl IntoResponse {
    let template = GamesTemplate {
        title: "Your Games".to_string(),
        games: vec![],
    };
    Html(template.render().unwrap())
}

pub async fn health() -> &'static str {
    "OK"
}
