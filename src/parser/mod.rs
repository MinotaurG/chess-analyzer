//! Parser module for reading chess game formats
//! 
//! Currently supports:
//! - PGN (Portable Game Notation)

pub mod pgn;

// Re-export commonly used items for convenience
pub use pgn::PgnGame;
pub use pgn::parse_pgn_file;
