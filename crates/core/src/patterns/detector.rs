//! Pattern detection engine

use shakmaty::{Chess, Position, Move, Role, fen::Fen, EnPassantMode, san::San};

use super::types::*;
use crate::engine::StockfishEngine;
use crate::error::{Result, Error};

pub struct PatternDetector {
    engine: StockfishEngine,
}

impl PatternDetector {
    pub fn new() -> Result<Self> {
        let engine = StockfishEngine::new("stockfish")
            .map_err(|e| Error::Lichess(format!("Failed to start Stockfish: {}", e)))?;
        Ok(Self { engine })
    }

    /// Analyze a game and detect patterns
    /// moves: list of moves in SAN format (e.g., "e4", "Nf3")
    /// username: the player we're analyzing for
    pub fn analyze_game(
        &mut self,
        moves: &[String],
        username: &str,
        white_player: &str,
    ) -> Result<Vec<DetectedPattern>> {
        let mut patterns = Vec::new();
        let mut position = Chess::default();
        let is_white = username.eq_ignore_ascii_case(white_player);
        
        let mut prev_eval: Option<i32> = None;

        for (ply, move_str) in moves.iter().enumerate() {
            let move_number = (ply / 2) + 1;
            let is_player_move = (ply % 2 == 0) == is_white;

            // Parse SAN move
            let san: San = match move_str.parse() {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Warning: Could not parse SAN '{}' at ply {}: {}", move_str, ply, e);
                    break;
                }
            };

            let mv = match san.to_move(&position) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!("Warning: Invalid move '{}' at ply {}: {}", move_str, ply, e);
                    break;
                }
            };

            let fen_before = Fen::from_position(&position, EnPassantMode::Legal).to_string();

            if is_player_move {
                // Get Stockfish eval for this position
                self.engine.set_position(Some(&fen_before), None)
                    .map_err(|e| Error::Lichess(format!("Engine error: {}", e)))?;
                
                let analysis = self.engine.analyze(12)
                    .map_err(|e| Error::Lichess(format!("Analysis error: {}", e)))?;

                let best_move = &analysis.best_move;
                let current_eval = match &analysis.evaluation {
                    crate::engine::Evaluation::Centipawns(cp) => *cp,
                    crate::engine::Evaluation::Mate(m) => if *m > 0 { 10000 } else { -10000 },
                };

                // Adjust eval perspective (negative for black)
                let eval_for_player = if is_white { current_eval } else { -current_eval };
                let player_uci = move_to_uci(&mv);

                // Check if player's move differs from best
                if !best_move.is_empty() && *best_move != player_uci {
                    // Calculate centipawn loss
                    if let Some(prev) = prev_eval {
                        let cp_loss = (prev - eval_for_player).max(0);
                        
                        if let Some(severity) = Severity::from_cp_loss(cp_loss) {
                            let pattern_type = classify_pattern(&position, &mv, cp_loss);
                            
                            patterns.push(DetectedPattern {
                                move_number: move_number as u16,
                                ply: ply as u16,
                                pattern_type,
                                severity,
                                cp_loss,
                                player_move: move_str.clone(),
                                best_move: best_move.clone(),
                                fen_before: fen_before.clone(),
                                fen_after: String::new(),
                                description: format!(
                                    "Move {}: played {} instead of {} (-{} cp)",
                                    move_number, move_str, best_move, cp_loss
                                ),
                            });
                        }
                    }
                }

                prev_eval = Some(eval_for_player);
            }

            // Apply the move
            position = match position.play(mv.clone()) {
                Ok(p) => p,
                Err(_) => break,
            };

            // Update fen_after for last pattern
            if let Some(last) = patterns.last_mut() {
                if last.ply == ply as u16 {
                    last.fen_after = Fen::from_position(&position, EnPassantMode::Legal).to_string();
                }
            }
        }

        Ok(patterns)
    }
}

/// Convert shakmaty Move to UCI string
fn move_to_uci(mv: &Move) -> String {
    match mv {
        Move::Normal { from, to, promotion, .. } => {
            let promo = promotion.map(|r| match r {
                Role::Queen => "q",
                Role::Rook => "r",
                Role::Bishop => "b",
                Role::Knight => "n",
                _ => "",
            }).unwrap_or("");
            format!("{}{}{}", from, to, promo)
        }
        Move::EnPassant { from, to, .. } => format!("{}{}", from, to),
        Move::Castle { king, rook } => {
            let king_to = if rook.file() > king.file() {
                shakmaty::Square::from_coords(shakmaty::File::G, king.rank())
            } else {
                shakmaty::Square::from_coords(shakmaty::File::C, king.rank())
            };
            format!("{}{}", king, king_to)
        }
        Move::Put { .. } => String::new(),
    }
}

fn classify_pattern(position: &Chess, played_move: &Move, cp_loss: i32) -> PatternType {
    let moved_piece = match played_move {
        Move::Normal { role, .. } => Some(*role),
        Move::EnPassant { .. } => Some(Role::Pawn),
        Move::Castle { .. } => Some(Role::King),
        _ => None,
    };

    if cp_loss >= 800 {
        if moved_piece == Some(Role::Queen) {
            return PatternType::QueenBlunder;
        }
    }

    if cp_loss >= 400 {
        if moved_piece == Some(Role::Rook) {
            return PatternType::RookBlunder;
        }
        if moved_piece == Some(Role::Bishop) || moved_piece == Some(Role::Knight) {
            return PatternType::MinorPieceBlunder;
        }
    }

    let piece_count = position.board().occupied().count();
    if piece_count >= 28 {
        return PatternType::OpeningInaccuracy;
    }
    
    if piece_count <= 10 {
        return PatternType::EndgameError;
    }

    PatternType::TacticalMiss
}
