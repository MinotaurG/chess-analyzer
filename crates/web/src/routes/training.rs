use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
    Json,
    http::StatusCode,
};
use serde::Deserialize;
use std::sync::Arc;

use chess_analyzer_core::storage::{TrainingStats, AllTrainingStats};
use crate::AppState;

// ============================================================================
// TEMPLATES
// ============================================================================

#[derive(Template)]
#[template(path = "training/coordinates.html")]
pub struct CoordinatesTemplate {
    pub title: String,
}

#[derive(Template)]
#[template(path = "training/visualization.html")]
pub struct VisualizationTemplate {
    pub title: String,
    pub difficulty: String,
}

#[derive(Template)]
#[template(path = "training/openings.html")]
pub struct OpeningsTemplate {
    pub title: String,
    pub lines: Vec<OpeningLineView>,
}

#[derive(Template)]
#[template(path = "training/puzzles.html")]
pub struct PuzzlesTemplate {
    pub title: String,
    pub puzzle_count: usize,
}

#[derive(Template)]
#[template(path = "training/index.html")]
pub struct TrainingHubTemplate {
    pub streak: u32,
    pub today_drills: u32,
    pub total_drills: u32,
    pub accuracy: u32,
    pub coord_progress: u32,
    pub viz_progress: u32,
    pub opening_progress: u32,
    pub opening_lines: u32,
}

pub struct OpeningLineView {
    pub idx: usize,
    pub name: String,
    pub eco: String,
    pub color: String,
    pub accuracy: f32,
    pub times_drilled: u32,
}

// ============================================================================
// QUERY PARAMS
// ============================================================================

#[derive(Deserialize)]
pub struct DifficultyQuery {
    pub difficulty: Option<String>,
}

// ============================================================================
// HANDLERS
// ============================================================================

pub async fn training_hub(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    
    let default_stats = TrainingStats {
        today_attempts: 0,
        today_correct: 0,
        total_attempts: 0,
        total_correct: 0,
        total_time_ms: 0,
        best_time_ms: None,
        streak: 0,
    };
    
    let stats = db.get_all_training_stats().unwrap_or(AllTrainingStats {
        coordinates: default_stats.clone(),
        visualization: default_stats.clone(),
        openings: default_stats,
        today_total: 0,
        all_time_total: 0,
        overall_accuracy: 0,
        max_streak: 0,
    });

    let template = TrainingHubTemplate {
        streak: stats.max_streak,
        today_drills: stats.today_total,
        total_drills: stats.all_time_total,
        accuracy: stats.overall_accuracy,
        coord_progress: stats.coordinates.accuracy(),
        viz_progress: stats.visualization.accuracy(),
        opening_progress: stats.openings.accuracy(),
        opening_lines: 0,
    };
    Html(template.render().unwrap())
}

pub async fn coordinates_drill() -> impl IntoResponse {
    let template = CoordinatesTemplate {
        title: "Coordinate Training".to_string(),
    };
    Html(template.render().unwrap())
}

pub async fn visualization_drill(
    Query(params): Query<DifficultyQuery>,
) -> impl IntoResponse {
    let difficulty = params.difficulty.unwrap_or_else(|| "beginner".to_string());

    let template = VisualizationTemplate {
        title: "Board Visualization".to_string(),
        difficulty,
    };
    Html(template.render().unwrap())
}

pub async fn openings_trainer(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let username = state.username.lock().unwrap().clone();

    let lines: Vec<OpeningLineView> = if let Some(ref user) = username {
        let db = state.db.lock().unwrap();
        let games = db.get_all_games().unwrap_or_default();

        let extracted = chess_analyzer_core::training::openings::OpeningTrainer::extract_from_games(
            &games, user, 3
        );

        extracted.iter().enumerate().map(|(idx, line)| {
            OpeningLineView {
                idx,
                name: line.name.clone(),
                eco: line.eco.clone(),
                color: line.color_name().to_string(),
                accuracy: line.accuracy(),
                times_drilled: line.times_drilled,
            }
        }).collect()
    } else {
        Vec::new()
    };

    let template = OpeningsTemplate {
        title: "Opening Trainer".to_string(),
        lines,
    };
    Html(template.render().unwrap())
}

// ============================================================================
// API
// ============================================================================

#[derive(Deserialize)]
pub struct SaveSessionRequest {
    pub training_type: String,
    pub attempts: u32,
    pub correct: u32,
    pub total_time_ms: u64,
    pub best_time_ms: Option<u64>,
}

#[derive(serde::Serialize)]
pub struct PuzzleResponse {
    pub id: i64,
    pub fen: String,
    pub best_move: String,
    pub player_move: String,
    pub severity: String,
    pub cp_loss: Option<i32>,
    pub game_id: i64,
    pub move_number: u16,
}

pub async fn puzzles_trainer(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    let count = db.get_all_patterns().map(|p| p.len()).unwrap_or(0);
    
    let template = PuzzlesTemplate {
        title: "Puzzle Training".to_string(),
        puzzle_count: count,
    };
    Html(template.render().unwrap())
}

pub async fn get_puzzle(
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    let db = state.db.lock().unwrap();
    
    match db.get_random_puzzle() {
        Ok(Some(pattern)) => {
            let response = PuzzleResponse {
                id: pattern.id,
                fen: pattern.position_fen,
                best_move: pattern.best_move,
                player_move: pattern.player_move,
                severity: pattern.severity,
                cp_loss: pattern.centipawn_loss,
                game_id: pattern.game_id,
                move_number: pattern.move_number,
            };
            Json(response).into_response()
        }
        Ok(None) => StatusCode::NOT_FOUND.into_response(),
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}

pub async fn save_session(
    State(state): State<Arc<AppState>>,
    Json(req): Json<SaveSessionRequest>,
) -> StatusCode {
    let db = state.db.lock().unwrap();
    
    match db.save_training_session(
        &req.training_type,
        req.attempts,
        req.correct,
        req.total_time_ms,
        req.best_time_ms,
    ) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::INTERNAL_SERVER_ERROR,
    }
}

// Continuation API - analyze position after correct move
#[derive(serde::Serialize)]
pub struct ContinuationResponse {
    pub moves: Vec<String>,
    pub eval: String,
}

pub async fn get_continuation(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(puzzle_id): axum::extract::Path<i64>,
) -> impl IntoResponse {
    // Get the puzzle
    let pattern = {
        let db = state.db.lock().unwrap();
        db.get_all_patterns()
            .ok()
            .and_then(|patterns| patterns.into_iter().find(|p| p.id == puzzle_id))
    };
    
    let Some(pattern) = pattern else {
        return StatusCode::NOT_FOUND.into_response();
    };
    
    // Create position after the best move
    let fen = &pattern.position_fen;
    let best_move = &pattern.best_move;
    
    // Spawn blocking task for Stockfish
    let fen = fen.clone();
    let best_move = best_move.clone();
    
    let result = tokio::task::spawn_blocking(move || {
        use chess_analyzer_core::engine::StockfishEngine;
        
        let mut engine = StockfishEngine::new("stockfish").ok()?;
        
        // Set position and make the best move
        engine.set_position(Some(&fen), Some(&[best_move])).ok()?;
        
        // Analyze for continuation
        let analysis = engine.analyze(16).ok()?;
        engine.quit().ok();
        
        // Return principal variation (skip first move since that's opponent's response)
        let moves: Vec<String> = analysis.pv.into_iter().take(4).collect();
        let eval = format!("{}", analysis.evaluation);
        
        Some(ContinuationResponse { moves, eval })
    }).await.ok().flatten();
    
    match result {
        Some(continuation) => Json(continuation).into_response(),
        None => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    }
}
