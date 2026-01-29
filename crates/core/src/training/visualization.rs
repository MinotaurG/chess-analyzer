//! Board visualization training

use shakmaty::{Chess, Position, Square, Role, Color, CastlingMode, fen::Fen};
use rand::seq::IndexedRandom;
use rand::Rng;

pub struct VisualizationDrill {
    position: Chess,
    fen: String,
    drill_type: VisualizationType,
    difficulty: Difficulty,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VisualizationType {
    PieceOnSquare,
    FindPiece,
    IsAttacked,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone)]
pub struct VisualizationQuestion {
    pub fen: String,
    pub question: String,
    pub correct_answer: String,
    pub options: Option<Vec<String>>,
    pub show_board_for_ms: u64,
}

impl VisualizationDrill {
    pub fn new(drill_type: VisualizationType, difficulty: Difficulty) -> Self {
        Self {
            position: Chess::default(),
            fen: "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
            drill_type,
            difficulty,
        }
    }

    pub fn with_position(mut self, fen: &str) -> Result<Self, String> {
        let parsed: Fen = fen.parse().map_err(|e| format!("Invalid FEN: {}", e))?;
        self.position = parsed
            .into_position(CastlingMode::Standard)
            .map_err(|e| format!("Invalid position: {}", e))?;
        self.fen = fen.to_string();
        Ok(self)
    }

    pub fn generate_question(&self) -> VisualizationQuestion {
        let mut rng = rand::rng();

        match self.drill_type {
            VisualizationType::PieceOnSquare => {
                let square = self.random_relevant_square(&mut rng);
                let piece = self.position.board().piece_at(square);

                let correct_answer = match piece {
                    Some(p) => format!("{} {}",
                        if p.color == Color::White { "White" } else { "Black" },
                        piece_name(p.role)
                    ),
                    None => "Empty".to_string(),
                };

                VisualizationQuestion {
                    fen: self.fen.clone(),
                    question: format!("What is on {}?", square),
                    correct_answer,
                    options: Some(vec![
                        "Empty".to_string(),
                        "White Pawn".to_string(),
                        "Black Pawn".to_string(),
                        "White Knight".to_string(),
                        "Black Knight".to_string(),
                        "White Bishop".to_string(),
                        "Black Bishop".to_string(),
                        "White Rook".to_string(),
                        "Black Rook".to_string(),
                        "White Queen".to_string(),
                        "Black Queen".to_string(),
                        "White King".to_string(),
                        "Black King".to_string(),
                    ]),
                    show_board_for_ms: self.show_duration(),
                }
            }
            VisualizationType::FindPiece => {
                // Get all occupied squares with their pieces
                let occupied: Vec<Square> = self.position.board().occupied().into_iter().collect();
                if occupied.is_empty() {
                    return self.fallback_question();
                }

                let square = *occupied.choose(&mut rng).unwrap();
                let piece = self.position.board().piece_at(square).unwrap();
                let color_name = if piece.color == Color::White { "White" } else { "Black" };

                VisualizationQuestion {
                    fen: self.fen.clone(),
                    question: format!("Where is the {} {}?", color_name, piece_name(piece.role)),
                    correct_answer: square.to_string(),
                    options: None,
                    show_board_for_ms: self.show_duration(),
                }
            }
            VisualizationType::IsAttacked => {
                let square = self.random_relevant_square(&mut rng);
                let color = if rng.random_bool(0.5) { Color::White } else { Color::Black };

                let attackers = self.position.board().attacks_to(
                    square,
                    color,
                    self.position.board().occupied()
                );
                let is_attacked = !attackers.is_empty();

                VisualizationQuestion {
                    fen: self.fen.clone(),
                    question: format!("Is {} attacked by {}?",
                        square,
                        if color == Color::White { "White" } else { "Black" }
                    ),
                    correct_answer: if is_attacked { "Yes" } else { "No" }.to_string(),
                    options: Some(vec!["Yes".to_string(), "No".to_string()]),
                    show_board_for_ms: self.show_duration(),
                }
            }
        }
    }

    fn random_relevant_square(&self, rng: &mut impl Rng) -> Square {
        let occupied: Vec<Square> = self.position.board().occupied().into_iter().collect();

        if !occupied.is_empty() && rng.random_bool(0.7) {
            *occupied.choose(rng).unwrap()
        } else {
            let all_squares: Vec<Square> = Square::ALL.to_vec();
            *all_squares.choose(rng).unwrap()
        }
    }

    fn show_duration(&self) -> u64 {
        match self.difficulty {
            Difficulty::Beginner => 5000,
            Difficulty::Intermediate => 3000,
            Difficulty::Advanced => 1500,
        }
    }

    fn fallback_question(&self) -> VisualizationQuestion {
        VisualizationQuestion {
            fen: self.fen.clone(),
            question: "How many pieces are on the board?".to_string(),
            correct_answer: self.position.board().occupied().count().to_string(),
            options: None,
            show_board_for_ms: self.show_duration(),
        }
    }
}

fn piece_name(role: Role) -> &'static str {
    match role {
        Role::Pawn => "Pawn",
        Role::Knight => "Knight",
        Role::Bishop => "Bishop",
        Role::Rook => "Rook",
        Role::Queen => "Queen",
        Role::King => "King",
    }
}

pub fn training_positions(difficulty: Difficulty) -> Vec<&'static str> {
    match difficulty {
        Difficulty::Beginner => vec![
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
            "rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1",
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/8/PPPP1PPP/RNBQKBNR w KQkq - 0 2",
            "rnbqkbnr/pppp1ppp/8/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R b KQkq - 1 2",
        ],
        Difficulty::Intermediate => vec![
            "r1bqkbnr/pppp1ppp/2n5/4p3/4P3/5N2/PPPP1PPP/RNBQKB1R w KQkq - 2 3",
            "rnbqkb1r/pppp1ppp/5n2/4p3/2B1P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
            "r1bqkbnr/pppp1ppp/2n5/1B2p3/4P3/5N2/PPPP1PPP/RNBQK2R b KQkq - 3 3",
        ],
        Difficulty::Advanced => vec![
            "r1bq1rk1/ppp2ppp/2np1n2/2b1p3/2B1P3/2NP1N2/PPP2PPP/R1BQ1RK1 w - - 0 7",
            "r2qkb1r/ppp2ppp/2n1bn2/3pp3/2B1P3/2N2N2/PPPP1PPP/R1BQK2R w KQkq - 0 5",
        ],
    }
}
