//! Opening repertoire trainer

use shakmaty::{Chess, san::San, Color, Position, EnPassantMode, fen::Fen};
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct OpeningLine {
    pub eco: String,
    pub name: String,
    pub moves: Vec<String>,
    pub for_color: Color,
    pub times_drilled: u32,
    pub times_correct: u32,
    pub last_drilled: Option<u64>,
}

impl OpeningLine {
    /// Returns the color as a string ("White" or "Black")
    pub fn color_name(&self) -> &'static str {
        if self.for_color == Color::White { "White" } else { "Black" }
    }

    pub fn accuracy(&self) -> f32 {
        if self.times_drilled == 0 {
            return 0.0;
        }
        (self.times_correct as f32 / self.times_drilled as f32) * 100.0
    }

    pub fn needs_review(&self, now: u64) -> bool {
        match self.last_drilled {
            None => true,
            Some(last) => {
                let days_since = (now - last) / 86400;
                let interval = if self.accuracy() > 90.0 {
                    7
                } else if self.accuracy() > 70.0 {
                    3
                } else {
                    1
                };
                days_since >= interval
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct DrillResult {
    pub line_name: String,
    pub move_number: u16,
    pub expected: String,
    pub played: String,
    pub correct: bool,
}

pub struct OpeningTrainer {
    repertoire: Vec<OpeningLine>,
    current_position: Chess,
    current_line_idx: Option<usize>,
    current_move_idx: usize,
}

impl OpeningTrainer {
    pub fn new() -> Self {
        Self {
            repertoire: Vec::new(),
            current_position: Chess::default(),
            current_line_idx: None,
            current_move_idx: 0,
        }
    }

    pub fn add_line(&mut self, line: OpeningLine) {
        self.repertoire.push(line);
    }

    pub fn extract_from_games(
        games: &[crate::storage::StoredGame],
        username: &str,
        min_games: u32,
    ) -> Vec<OpeningLine> {
        let mut opening_counts: HashMap<(String, String, Color), Vec<String>> = HashMap::new();

        for game in games {
            let eco = game.opening_eco.clone().unwrap_or_default();
            let name = game.opening_name.clone().unwrap_or_default();

            if eco.is_empty() {
                continue;
            }

            let color = if game.white_username.eq_ignore_ascii_case(username) {
                Color::White
            } else {
                Color::Black
            };

            let moves: Vec<String> = game.moves
                .split_whitespace()
                .take(20)
                .map(String::from)
                .collect();

            opening_counts
                .entry((eco.clone(), name.clone(), color))
                .or_default()
                .extend(moves);
        }

        opening_counts
            .into_iter()
            .filter(|(_, moves)| moves.len() >= min_games as usize * 10)
            .map(|((eco, name, color), moves)| {
                let canonical_moves: Vec<String> = moves
                    .chunks(20)
                    .next()
                    .unwrap_or(&[])
                    .to_vec();

                OpeningLine {
                    eco,
                    name,
                    moves: canonical_moves,
                    for_color: color,
                    times_drilled: 0,
                    times_correct: 0,
                    last_drilled: None,
                }
            })
            .collect()
    }

    pub fn start_line(&mut self, line_idx: usize) -> Option<&str> {
        if line_idx >= self.repertoire.len() {
            return None;
        }

        self.current_position = Chess::default();
        self.current_line_idx = Some(line_idx);
        self.current_move_idx = 0;

        let line = &self.repertoire[line_idx];

        if line.for_color == Color::Black && !line.moves.is_empty() {
            self.play_opponent_move();
        }

        self.get_prompt()
    }

    pub fn get_prompt(&self) -> Option<&str> {
        let line_idx = self.current_line_idx?;
        let line = &self.repertoire[line_idx];

        if self.current_move_idx >= line.moves.len() {
            return None;
        }

        Some(&line.name)
    }

    pub fn current_fen(&self) -> String {
        Fen::from_position(&self.current_position, EnPassantMode::Legal).to_string()
    }

    pub fn check_move(&mut self, player_move: &str) -> Option<DrillResult> {
        let line_idx = self.current_line_idx?;
        let line = &mut self.repertoire[line_idx];

        if self.current_move_idx >= line.moves.len() {
            return None;
        }

        let expected = &line.moves[self.current_move_idx];
        let correct = player_move.trim() == expected.trim();

        let result = DrillResult {
            line_name: line.name.clone(),
            move_number: (self.current_move_idx / 2 + 1) as u16,
            expected: expected.clone(),
            played: player_move.to_string(),
            correct,
        };

        line.times_drilled += 1;
        if correct {
            line.times_correct += 1;
        }

        if correct {
            if let Ok(san) = expected.parse::<San>() {
                if let Ok(mv) = san.to_move(&self.current_position) {
                    if let Ok(new_pos) = self.current_position.clone().play(mv) {
                        self.current_position = new_pos;
                        self.current_move_idx += 1;
                        self.play_opponent_move();
                    }
                }
            }
        }

        Some(result)
    }

    fn play_opponent_move(&mut self) {
        let line_idx = match self.current_line_idx {
            Some(idx) => idx,
            None => return,
        };

        let line = &self.repertoire[line_idx];
        let is_our_turn = (self.current_move_idx % 2 == 0) == (line.for_color == Color::White);

        if !is_our_turn && self.current_move_idx < line.moves.len() {
            let move_str = &line.moves[self.current_move_idx];
            if let Ok(san) = move_str.parse::<San>() {
                if let Ok(mv) = san.to_move(&self.current_position) {
                    if let Ok(new_pos) = self.current_position.clone().play(mv) {
                        self.current_position = new_pos;
                        self.current_move_idx += 1;
                    }
                }
            }
        }
    }

    pub fn lines_to_review(&self, now: u64) -> Vec<(usize, &OpeningLine)> {
        self.repertoire
            .iter()
            .enumerate()
            .filter(|(_, line)| line.needs_review(now))
            .collect()
    }

    pub fn summary(&self) -> RepertoireSummary {
        let total_lines = self.repertoire.len() as u32;
        let mastered = self.repertoire.iter().filter(|l| l.accuracy() > 90.0).count() as u32;
        let learning = self.repertoire.iter().filter(|l| l.accuracy() > 50.0 && l.accuracy() <= 90.0).count() as u32;
        let struggling = self.repertoire.iter().filter(|l| l.accuracy() <= 50.0 && l.times_drilled > 0).count() as u32;

        RepertoireSummary {
            total_lines,
            mastered,
            learning,
            struggling,
            not_started: total_lines - mastered - learning - struggling,
        }
    }
}

#[derive(Debug, Clone)]
pub struct RepertoireSummary {
    pub total_lines: u32,
    pub mastered: u32,
    pub learning: u32,
    pub struggling: u32,
    pub not_started: u32,
}
