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
    pub username: Option<String>,
}

#[derive(Template)]
#[template(path = "patterns.html")]
pub struct PatternsTemplate {
    pub title: String,
    pub patterns: Vec<PatternRow>,
    pub summary: PatternSummaryView,
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

pub struct PatternRow {
    pub game_id: i64,
    pub move_number: u16,
    pub pattern_type: String,
    pub severity: String,
    pub cp_loss: i32,
    pub description: String,
}

pub struct PatternSummaryView {
    pub total_games: u32,
    pub blunders: u32,
    pub mistakes: u32,
    pub inaccuracies: u32,
}

#[derive(serde::Deserialize)]
pub struct SyncForm {
    pub username: String,
}

pub async fn index(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let games_count = db.count_games().unwrap_or(0);
    let patterns_found = db.count_patterns().unwrap_or(0);

    let template = IndexTemplate {
        title: "Chess Analyzer".to_string(),
        games_count,
        patterns_found,
        username: state.username.lock().unwrap().clone(),
    };
    Html(template.render().unwrap())
}

pub async fn games_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let stored_games = db.get_recent_games(50).unwrap_or_default();
    
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
        username: state.username.lock().unwrap().clone(),
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
        return Redirect::to("/");
    }

    *state.username.lock().unwrap() = Some(username.clone());

    let client = match chess_analyzer_core::LichessClient::new() {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Failed to create client: {}", e);
            return Redirect::to("/");
        }
    };

    let params = chess_analyzer_core::lichess::GameExportParams::new()
        .max(500);

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

pub async fn analyze_games(State(state): State<Arc<AppState>>) -> Redirect {
    let username = state.username.lock().unwrap().clone();
    let username = match username {
        Some(u) => u,
        None => return Redirect::to("/"),
    };

    println!("Analyzing games for {}...", username);

    // Run blocking Stockfish analysis in a separate thread
    let games = {
        let db = state.db.lock().unwrap();
        db.get_unanalyzed_games(5).unwrap_or_default()
    };

    if games.is_empty() {
        println!("No unanalyzed games found");
        return Redirect::to("/patterns");
    }

    // Spawn blocking task for Stockfish
    let state_clone = state.clone();
    tokio::task::spawn_blocking(move || {
        let mut detector = match chess_analyzer_core::PatternDetector::new() {
            Ok(d) => d,
            Err(e) => {
                eprintln!("Failed to create detector: {}", e);
                return;
            }
        };

        for game in &games {
            let moves: Vec<String> = game.moves.split_whitespace().map(String::from).collect();
            if moves.is_empty() {
                continue;
            }

            println!("Analyzing game {} ({} vs {}, {} moves)...", 
                game.id, game.white_username, game.black_username, moves.len());

            match detector.analyze_game(&moves, &username, &game.white_username) {
                Ok(patterns) => {
                    println!("Found {} patterns in game {}", patterns.len(), game.id);
                    let db = state_clone.db.lock().unwrap();
                    for pattern in &patterns {
                        if let Err(e) = db.insert_pattern(game.id, pattern) {
                            eprintln!("Failed to insert pattern: {}", e);
                        }
                    }
                    let _ = db.mark_game_analyzed(game.id);
                }
                Err(e) => {
                    eprintln!("Failed to analyze game {}: {}", game.id, e);
                }
            }
        }
    }).await.ok();

    Redirect::to("/patterns")
}

pub async fn patterns_list(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let stored_patterns = db.get_all_patterns().unwrap_or_default();

    let patterns: Vec<PatternRow> = stored_patterns.iter().map(|p| {
        PatternRow {
            game_id: p.game_id,
            move_number: p.move_number,
            pattern_type: p.pattern_type.clone(),
            severity: p.severity.clone(),
            cp_loss: p.centipawn_loss.unwrap_or(0),
            description: p.description.clone(),
        }
    }).collect();

    let summary = PatternSummaryView {
        total_games: db.count_games().unwrap_or(0),
        blunders: stored_patterns.iter().filter(|p| p.severity == "blunder").count() as u32,
        mistakes: stored_patterns.iter().filter(|p| p.severity == "mistake").count() as u32,
        inaccuracies: stored_patterns.iter().filter(|p| p.severity == "inaccuracy").count() as u32,
    };

    let template = PatternsTemplate {
        title: "Patterns".to_string(),
        patterns,
        summary,
    };
    Html(template.render().unwrap())
}

pub async fn health() -> &'static str {
    "OK"
}
