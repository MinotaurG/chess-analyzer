//! Chess engine integration
//! 
//! Provides interface to UCI-compatible engines like Stockfish.

pub mod analysis;
pub mod stockfish;

// Re-export main types for convenience
pub use analysis::{Evaluation, MoveAnalysis, PositionAnalysis};
pub use stockfish::{EngineError, StockfishEngine};
