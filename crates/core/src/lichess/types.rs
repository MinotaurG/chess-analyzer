//! Lichess API data types

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default)]
pub struct GameExportParams {
    pub max: Option<u32>,
    pub perf_type: Option<PerfType>,
    pub rated_only: bool,
    pub with_analysis: bool,
    pub since: Option<u64>,  // Unix timestamp in milliseconds
}

impl GameExportParams {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn max(mut self, max: u32) -> Self {
        self.max = Some(max);
        self
    }

    pub fn perf_type(mut self, perf_type: PerfType) -> Self {
        self.perf_type = Some(perf_type);
        self
    }

    pub fn rated_only(mut self) -> Self {
        self.rated_only = true;
        self
    }

    pub fn with_analysis(mut self) -> Self {
        self.with_analysis = true;
        self
    }

    pub fn since(mut self, timestamp: u64) -> Self {
        self.since = Some(timestamp);
        self
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PerfType {
    UltraBullet,
    Bullet,
    Blitz,
    Rapid,
    Classical,
    Correspondence,
}

impl PerfType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PerfType::UltraBullet => "ultraBullet",
            PerfType::Bullet => "bullet",
            PerfType::Blitz => "blitz",
            PerfType::Rapid => "rapid",
            PerfType::Classical => "classical",
            PerfType::Correspondence => "correspondence",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessGame {
    pub id: String,
    pub rated: bool,
    pub variant: String,
    pub speed: String,
    pub perf: String,
    pub created_at: u64,
    pub last_move_at: u64,
    pub status: String,
    pub players: Players,
    #[serde(default)]
    pub winner: Option<String>,
    #[serde(default)]
    pub moves: Option<String>,
    #[serde(default)]
    pub pgn: Option<String>,
    #[serde(default)]
    pub opening: Option<Opening>,
    #[serde(default)]
    pub clock: Option<Clock>,
}

impl LichessGame {
    pub fn result(&self) -> &str {
        match self.winner.as_deref() {
            Some("white") => "1-0",
            Some("black") => "0-1",
            None if self.status == "draw" || self.status == "stalemate" => "1/2-1/2",
            _ => "*",
        }
    }

    pub fn white_username(&self) -> &str {
        self.players.white.user.as_ref()
            .map(|u| u.name.as_str())
            .unwrap_or("Anonymous")
    }

    pub fn black_username(&self) -> &str {
        self.players.black.user.as_ref()
            .map(|u| u.name.as_str())
            .unwrap_or("Anonymous")
    }

    pub fn white_rating(&self) -> Option<u16> {
        self.players.white.rating
    }

    pub fn black_rating(&self) -> Option<u16> {
        self.players.black.rating
    }

    pub fn move_list(&self) -> Vec<String> {
        self.moves.as_ref()
            .map(|m| m.split_whitespace().map(String::from).collect())
            .unwrap_or_default()
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Players {
    pub white: Player,
    pub black: Player,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    pub user: Option<User>,
    pub rating: Option<u16>,
    pub rating_diff: Option<i16>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub name: String,
    #[serde(default)]
    pub id: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Opening {
    pub eco: String,
    pub name: String,
    pub ply: u8,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Clock {
    pub initial: u32,
    pub increment: u32,
    pub total_time: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CloudEval {
    pub fen: String,
    pub knodes: u64,
    pub depth: u8,
    pub pvs: Vec<PvLine>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PvLine {
    pub moves: String,
    pub cp: Option<i32>,
    pub mate: Option<i32>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LichessUser {
    pub id: String,
    pub username: String,
    #[serde(default)]
    pub perfs: Option<serde_json::Value>,
    #[serde(default)]
    pub created_at: Option<u64>,
    #[serde(default)]
    pub seen_at: Option<u64>,
    #[serde(default)]
    pub count: Option<GameCount>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GameCount {
    pub all: u32,
    pub rated: u32,
    pub win: u32,
    pub loss: u32,
    pub draw: u32,
}
