//! Coordinate training - fundamental for algebraic notation

use shakmaty::{Square, Color};
use rand::seq::IndexedRandom;

pub struct CoordinateTrainer {
    mode: CoordinateMode,
    perspective: Color,
    history: Vec<CoordinateAttempt>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateMode {
    NameToSquare,
    SquareToName,
    SquareColor,
}

#[derive(Debug, Clone)]
pub struct CoordinateAttempt {
    pub square: Square,
    pub correct: bool,
    pub response_ms: u64,
}

impl CoordinateTrainer {
    pub fn new(mode: CoordinateMode, perspective: Color) -> Self {
        Self {
            mode,
            perspective,
            history: Vec::new(),
        }
    }

    pub fn mode(&self) -> CoordinateMode {
        self.mode
    }

    pub fn perspective(&self) -> Color {
        self.perspective
    }

    pub fn set_mode(&mut self, mode: CoordinateMode) {
        self.mode = mode;
    }

    pub fn set_perspective(&mut self, perspective: Color) {
        self.perspective = perspective;
    }

    pub fn next_square(&self) -> Square {
        let squares: Vec<Square> = Square::ALL.to_vec();
        let mut rng = rand::rng();
        *squares.choose(&mut rng).unwrap()
    }

    pub fn check_color(&self, square: Square, answer: &str) -> bool {
        let is_light = square.is_light();
        match answer.to_lowercase().as_str() {
            "light" | "white" => is_light,
            "dark" | "black" => !is_light,
            _ => false,
        }
    }

    pub fn square_color(square: Square) -> &'static str {
        if square.is_light() { "light" } else { "dark" }
    }

    pub fn check_square(&self, target: Square, clicked: Square) -> bool {
        target == clicked
    }

    pub fn check_name(&self, square: Square, typed: &str) -> bool {
        square.to_string().eq_ignore_ascii_case(typed.trim())
    }

    pub fn record(&mut self, square: Square, correct: bool, response_ms: u64) {
        self.history.push(CoordinateAttempt {
            square,
            correct,
            response_ms,
        });
    }

    pub fn attempts(&self) -> usize {
        self.history.len()
    }

    pub fn correct_count(&self) -> usize {
        self.history.iter().filter(|a| a.correct).count()
    }

    pub fn accuracy(&self) -> f32 {
        if self.history.is_empty() {
            return 0.0;
        }
        (self.correct_count() as f32 / self.history.len() as f32) * 100.0
    }

    pub fn avg_response_ms(&self) -> u64 {
        if self.history.is_empty() {
            return 0;
        }
        let total: u64 = self.history.iter().map(|a| a.response_ms).sum();
        total / self.history.len() as u64
    }

    pub fn best_time_ms(&self) -> Option<u64> {
        self.history
            .iter()
            .filter(|a| a.correct)
            .map(|a| a.response_ms)
            .min()
    }

    pub fn reset(&mut self) {
        self.history.clear();
    }

    pub fn weak_squares(&self) -> Vec<(Square, f32, u64)> {
        use std::collections::HashMap;

        let mut square_stats: HashMap<Square, (u32, u32, u64)> = HashMap::new();

        for attempt in &self.history {
            let entry = square_stats.entry(attempt.square).or_insert((0, 0, 0));
            entry.0 += 1;
            if attempt.correct {
                entry.1 += 1;
            }
            entry.2 += attempt.response_ms;
        }

        let avg_time = self.avg_response_ms();

        let mut weak: Vec<_> = square_stats
            .iter()
            .filter_map(|(sq, (total, correct, time))| {
                let accuracy = *correct as f32 / *total as f32;
                let avg_square_time = time / *total as u64;
                if accuracy < 0.8 || avg_square_time > avg_time * 2 {
                    Some((*sq, accuracy * 100.0, avg_square_time))
                } else {
                    None
                }
            })
            .collect();

        weak.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        weak
    }
}
