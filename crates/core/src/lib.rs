//! Chess Analyzer Core Library

use shakmaty::{Chess, Color, Position};

pub mod engine;
pub mod error;
pub mod lichess;
pub mod parser;
pub mod patterns;
pub mod storage;

pub use error::{Error, Result};
pub use lichess::LichessClient;
pub use patterns::{PatternDetector, DetectedPattern, PatternType, Severity};
pub use storage::Database;

/// Basic position information
#[derive(Debug)]
pub struct PositionInfo {
    pub piece_count: u32,
    pub legal_move_count: u32,
    pub side_to_move: Color,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_stalemate: bool,
}

/// Analyzes a chess position
pub fn analyze_position(position: &Chess) -> PositionInfo {
    let piece_count = position.board().occupied().count() as u32;
    let legal_moves = position.legal_moves();
    let legal_move_count = legal_moves.len() as u32;
    let side_to_move = position.turn();
    let is_check = position.is_check();
    let is_checkmate = position.is_checkmate();
    let is_stalemate = position.is_stalemate();

    PositionInfo {
        piece_count,
        legal_move_count,
        side_to_move,
        is_check,
        is_checkmate,
        is_stalemate,
    }
}

/// Creates the standard starting position
pub fn starting_position() -> Chess {
    Chess::default()
}
