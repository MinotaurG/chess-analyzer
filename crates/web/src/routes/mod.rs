use askama::Template;
use axum::{
    extract::State,
    response::{Html, IntoResponse, Redirect},
    Form,
};
use std::sync::Arc;

use crate::AppState;

#[derive(Template)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub title: String,
    pub games_count: u32,
    pub patterns_found: u32,
    pub username: Option<String>,
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
    pub opening: String,
    pub speed: String,
    pub date: String,
}

#[derive(serde::Deserialize)]
pub struct SyncForm {
    pub username: String,
}

pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let games_count = db.count_games().unwrap_or(0);

    let template = IndexTemplate {
        title: "Chess Analyzer".to_string(),
        games_count,
        patterns_found: 0,
        username: state.username.lock().unwrap().clone(),
    };
    Html(template.render().unwrap())
}

pub async fn games_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let stored_games = db.get_recent_games(50).unwrap_or_default();
    
    println!("Found {} games in database", stored_games.len());

    let games: Vec<GameRow> = stored_games.iter().map(|g| {
        let date = chrono::DateTime::from_timestamp(g.played_at as i64, 0)
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_default();

        GameRow {
            id: g.id,
            white: g.white_username.clone(),
            black: g.black_username.clone(),
            result: g.result.clone(),
            opening: g.opening_name.clone().unwrap_or_else(|| "-".to_string()),
            speed: g.speed.clone(),
            date,
        }
    }).collect();

    let template = GamesTemplate {
        title: "Your Games".to_string(),
        games,
    };
    Html(template.render().unwrap())
}

pub async fn sync_games(
    State(state): State<Arc<AppState>>,
    Form(form): Form<SyncForm>,
) -> Redirect {
    let username = form.username.trim().to_string();
    println!("Sync requested for: '{}'", username);
    
    if username.is_empty() {
        println!("Empty username, redirecting to /");
        return Redirect::to("/");
    }

    // Store username
    *state.username.lock().unwrap() = Some(username.clone());

    // Fetch games from Lichess
    println!("Creating Lichess client...");
    let client = match chess_analyzer_core::LichessClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
            return Redirect::to("/");
        }
    };

    let params = chess_analyzer_core::lichess::GameExportParams::new()
        .max(100);

    println!("Fetching games from Lichess...");
    match client.get_user_games(&username, &params).await {
        Ok(games) => {
            println!("Fetched {} games from Lichess", games.len());
            let db = state.db.lock().unwrap();
            match db.insert_games(&games) {
                Ok(count) => println!("Inserted {} games into database", count),
                Err(e) => eprintln!("Failed to insert games: {}", e),
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch games: {}", e);
        }
    }

    Redirect::to("/games")
}

pub async fn health() -> &'static str {
    "OK"
}
