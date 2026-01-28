//! Pattern types for chess mistake detection

use serde::{Deserialize, Serialize};

/// Severity of a mistake
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// >= 300 centipawn loss
    Blunder,
    /// >= 100 centipawn loss
    Mistake,
    /// >= 50 centipawn loss
    Inaccuracy,
}

impl Severity {
    pub fn from_cp_loss(cp_loss: i32) -> Option<Self> {
        match cp_loss {
            l if l >= 300 => Some(Severity::Blunder),
            l if l >= 100 => Some(Severity::Mistake),
            l if l >= 50 => Some(Severity::Inaccuracy),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Severity::Blunder => "blunder",
            Severity::Mistake => "mistake",
            Severity::Inaccuracy => "inaccuracy",
        }
    }
}

/// Type of tactical/positional pattern
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PatternType {
    // Tactical
    HangingPiece,
    MissedFork,
    MissedPin,
    MissedSkewer,
    MissedBackRank,
    MissedDiscoveredAttack,
    AllowedFork,
    AllowedPin,
    AllowedBackRank,
    
    // Material
    QueenBlunder,
    RookBlunder,
    MinorPieceBlunder,
    
    // Positional
    BadTrade,
    WeakeningMove,
    
    // Phase-specific
    OpeningInaccuracy,
    EndgameError,
    
    // Generic (when we can't classify further)
    TacticalMiss,
    Unknown,
}

impl PatternType {
    pub fn as_str(&self) -> &'static str {
        match self {
            PatternType::HangingPiece => "hanging_piece",
            PatternType::MissedFork => "missed_fork",
            PatternType::MissedPin => "missed_pin",
            PatternType::MissedSkewer => "missed_skewer",
            PatternType::MissedBackRank => "missed_back_rank",
            PatternType::MissedDiscoveredAttack => "missed_discovered_attack",
            PatternType::AllowedFork => "allowed_fork",
            PatternType::AllowedPin => "allowed_pin",
            PatternType::AllowedBackRank => "allowed_back_rank",
            PatternType::QueenBlunder => "queen_blunder",
            PatternType::RookBlunder => "rook_blunder",
            PatternType::MinorPieceBlunder => "minor_piece_blunder",
            PatternType::BadTrade => "bad_trade",
            PatternType::WeakeningMove => "weakening_move",
            PatternType::OpeningInaccuracy => "opening_inaccuracy",
            PatternType::EndgameError => "endgame_error",
            PatternType::TacticalMiss => "tactical_miss",
            PatternType::Unknown => "unknown",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            PatternType::HangingPiece => "Hanging Piece",
            PatternType::MissedFork => "Missed Fork",
            PatternType::MissedPin => "Missed Pin",
            PatternType::MissedSkewer => "Missed Skewer",
            PatternType::MissedBackRank => "Missed Back Rank",
            PatternType::MissedDiscoveredAttack => "Missed Discovered Attack",
            PatternType::AllowedFork => "Allowed Fork",
            PatternType::AllowedPin => "Allowed Pin",
            PatternType::AllowedBackRank => "Allowed Back Rank",
            PatternType::QueenBlunder => "Queen Blunder",
            PatternType::RookBlunder => "Rook Blunder",
            PatternType::MinorPieceBlunder => "Minor Piece Blunder",
            PatternType::BadTrade => "Bad Trade",
            PatternType::WeakeningMove => "Weakening Move",
            PatternType::OpeningInaccuracy => "Opening Inaccuracy",
            PatternType::EndgameError => "Endgame Error",
            PatternType::TacticalMiss => "Tactical Miss",
            PatternType::Unknown => "Unknown",
        }
    }
}

/// A detected pattern in a game
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedPattern {
    pub move_number: u16,
    pub ply: u16,
    pub pattern_type: PatternType,
    pub severity: Severity,
    pub cp_loss: i32,
    pub player_move: String,
    pub best_move: String,
    pub fen_before: String,
    pub fen_after: String,
    pub description: String,
}

/// Summary of patterns for a player
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternSummary {
    pub total_games: u32,
    pub total_moves: u32,
    pub blunders: u32,
    pub mistakes: u32,
    pub inaccuracies: u32,
    pub patterns: Vec<PatternCount>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternCount {
    pub pattern_type: PatternType,
    pub count: u32,
    pub total_cp_loss: i32,
}
