//! Types for representing chess analysis results

use std::fmt;

/// Represents a position evaluation
#[derive(Debug, Clone)]
pub enum Evaluation {
    /// Centipawn score (positive = white advantage)
    Centipawns(i32),
    /// Forced mate (positive = white mates, negative = black mates)
    Mate(i32),
}

impl Evaluation {
    /// Returns true if the position is winning for white
    pub fn is_white_winning(&self) -> bool {
        match self {
            Evaluation::Centipawns(cp) => *cp > 100,
            Evaluation::Mate(moves) => *moves > 0,
        }
    }

    /// Returns true if the position is winning for black
    pub fn is_black_winning(&self) -> bool {
        match self {
            Evaluation::Centipawns(cp) => *cp < -100,
            Evaluation::Mate(moves) => *moves < 0,
        }
    }

    /// Converts evaluation to a human-readable score
    pub fn as_score(&self) -> f32 {
        match self {
            Evaluation::Centipawns(cp) => *cp as f32 / 100.0,
            Evaluation::Mate(moves) => {
                if *moves > 0 {
                    100.0 // White mating
                } else {
                    -100.0 // Black mating
                }
            }
        }
    }
}

impl fmt::Display for Evaluation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Evaluation::Centipawns(cp) => {
                let score = *cp as f32 / 100.0;
                if score >= 0.0 {
                    write!(f, "+{:.2}", score)
                } else {
                    write!(f, "{:.2}", score)
                }
            }
            Evaluation::Mate(moves) => {
                if *moves > 0 {
                    write!(f, "M{}", moves)
                } else {
                    write!(f, "M{}", moves) // Already negative
                }
            }
        }
    }
}

/// Analysis of a single move
#[derive(Debug, Clone)]
pub struct MoveAnalysis {
    /// The move in UCI notation (e.g., "e2e4")
    pub mv: String,
    /// The move in SAN notation (e.g., "e4")
    pub san: Option<String>,
    /// Evaluation after this move
    pub evaluation: Evaluation,
    /// Principal variation (best line)
    pub pv: Vec<String>,
    /// Depth of analysis
    pub depth: u8,
}

/// Complete analysis of a position
#[derive(Debug, Clone)]
pub struct PositionAnalysis {
    /// Best move found
    pub best_move: String,
    /// Evaluation of the position
    pub evaluation: Evaluation,
    /// Analysis depth reached
    pub depth: u8,
    /// Principal variation (best line of play)
    pub pv: Vec<String>,
    /// Time spent analyzing (milliseconds)
    pub time_ms: u64,
    /// Nodes searched
    pub nodes: u64,
}

impl PositionAnalysis {
    /// Returns a brief summary of the analysis
    pub fn summary(&self) -> String {
        format!(
            "Eval: {} | Best: {} | Depth: {} | PV: {}",
            self.evaluation,
            self.best_move,
            self.depth,
            self.pv.iter().take(5).cloned().collect::<Vec<_>>().join(" ")
        )
    }
}
