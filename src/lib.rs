//! Chess Analyzer Library
//! 
//! This library provides tools for analyzing chess games
//! and identifying patterns in your play.

use shakmaty::{Chess, Position, Square, Role, Color};

/// Represents the result of analyzing a position
#[derive(Debug)]
pub struct PositionInfo {
    /// Total number of pieces on the board
    pub piece_count: u32,
    /// Number of legal moves available
    pub legal_move_count: u32,
    /// Whose turn it is
    pub side_to_move: Color,
    /// Is the current player in check?
    pub is_check: bool,
}

/// Analyzes a chess position and returns basic information
/// 
/// # Arguments
/// * `position` - A reference to a Chess position
/// 
/// # Returns
/// A `PositionInfo` struct containing position details
pub fn analyze_position(position: &Chess) -> PositionInfo {
    // Count all pieces on the board
    let piece_count = position.board().occupied().count() as u32;
    
    // Count legal moves
    let legal_moves = position.legal_moves();
    let legal_move_count = legal_moves.len() as u32;
    
    // Get side to move
    let side_to_move = position.turn();
    
    // Check if in check
    let is_check = position.is_check();
    
    PositionInfo {
        piece_count,
        legal_move_count,
        side_to_move,
        is_check,
    }
}

/// Creates the standard starting position
pub fn starting_position() -> Chess {
    Chess::default()
}

// Tests module - only compiled when running tests
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_starting_position() {
        let pos = starting_position();
        let info = analyze_position(&pos);
        
        // Starting position should have 32 pieces
        assert_eq!(info.piece_count, 32);
        
        // White moves first
        assert_eq!(info.side_to_move, Color::White);
        
        // White has 20 legal moves at start
        // (16 pawn moves + 4 knight moves)
        assert_eq!(info.legal_move_count, 20);
        
        // Not in check at start
        assert!(!info.is_check);
    }
}
