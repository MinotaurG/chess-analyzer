use askama::Template;
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse},
};
use serde::Deserialize;
use std::sync::Arc;

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
// TRAINING HUB
// ============================================================================

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

pub async fn training_hub() -> impl IntoResponse {
    // TODO: Load real stats from database
    let template = TrainingHubTemplate {
        streak: 0,
        today_drills: 0,
        total_drills: 0,
        accuracy: 0,
        coord_progress: 0,
        viz_progress: 0,
        opening_progress: 0,
        opening_lines: 0,
    };
    Html(template.render().unwrap())
}
