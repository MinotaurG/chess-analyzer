//! Chess Analyzer Library
//! 
//! This library provides tools for analyzing chess games
//! and identifying patterns in your play.

use shakmaty::{Chess, Position, Color};

// Declare and export the parser module
pub mod parser;

/// Represents the result of analyzing a position
#[derive(Debug)]
pub struct PositionInfo {
    pub piece_count: u32,
    pub legal_move_count: u32,
    pub side_to_move: Color,
    pub is_check: bool,
    pub is_checkmate: bool,
    pub is_stalemate: bool,
}

/// Analyzes a chess position and returns basic information
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

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_starting_position() {
        let pos = starting_position();
        let info = analyze_position(&pos);
        
        assert_eq!(info.piece_count, 32);
        assert_eq!(info.side_to_move, Color::White);
        assert_eq!(info.legal_move_count, 20);
        assert!(!info.is_check);
        assert!(!info.is_checkmate);
        assert!(!info.is_stalemate);
    }
}
