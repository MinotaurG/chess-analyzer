//! Database models

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredGame {
    pub id: i64,
    pub lichess_id: String,
    pub white_username: String,
    pub black_username: String,
    pub white_rating: Option<u16>,
    pub black_rating: Option<u16>,
    pub result: String,
    pub speed: String,
    pub rated: bool,
    pub opening_eco: Option<String>,
    pub opening_name: Option<String>,
    pub moves: String,
    pub pgn: Option<String>,
    pub analyzed: bool,
    pub played_at: u64,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPattern {
    pub id: i64,
    pub game_id: i64,
    pub move_number: u16,
    pub pattern_type: String,
    pub subtype: Option<String>,
    pub severity: String,
    pub centipawn_loss: Option<i32>,
    pub position_fen: String,
    pub description: String,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserSettings {
    pub id: i64,
    pub lichess_username: String,
    pub lichess_token: Option<String>,
    pub games_synced_at: Option<u64>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrainingStats {
    pub today_attempts: u32,
    pub today_correct: u32,
    pub total_attempts: u32,
    pub total_correct: u32,
    pub total_time_ms: u64,
    pub best_time_ms: Option<u64>,
    pub streak: u32,
}

impl TrainingStats {
    pub fn accuracy(&self) -> u32 {
        if self.total_attempts == 0 {
            0
        } else {
            ((self.total_correct as f64 / self.total_attempts as f64) * 100.0) as u32
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AllTrainingStats {
    pub coordinates: TrainingStats,
    pub visualization: TrainingStats,
    pub openings: TrainingStats,
    pub today_total: u32,
    pub all_time_total: u32,
    pub overall_accuracy: u32,
    pub max_streak: u32,
}
