//! PGN file parsing functionality

use pgn_reader::{RawTag, SanPlus, Skip, Visitor};
use shakmaty::{Chess, Position};
use std::fs;
use std::io::{self, Cursor};
use std::ops::ControlFlow;
use std::path::Path;

/// Represents a parsed chess game
#[derive(Debug, Clone)]
pub struct PgnGame {
    pub event: Option<String>,
    pub site: Option<String>,
    pub date: Option<String>,
    pub white: Option<String>,
    pub black: Option<String>,
    pub result: Option<String>,
    pub white_elo: Option<u16>,
    pub black_elo: Option<u16>,
    pub moves: Vec<String>,
    pub final_position: Chess,
}

impl PgnGame {
    pub fn move_count(&self) -> usize {
        self.moves.len()
    }

    pub fn summary(&self) -> String {
        let white = self.white.as_deref().unwrap_or("Unknown");
        let black = self.black.as_deref().unwrap_or("Unknown");
        let result = self.result.as_deref().unwrap_or("*");
        format!("{} vs {} - {}", white, black, result)
    }
}

#[derive(Default)]
struct GameTags {
    event: Option<String>,
    site: Option<String>,
    date: Option<String>,
    white: Option<String>,
    black: Option<String>,
    result: Option<String>,
    white_elo: Option<u16>,
    black_elo: Option<u16>,
}

struct GameMoves {
    tags: GameTags,
    moves: Vec<String>,
    current_position: Chess,
    success: bool,
}

struct GameParser;

impl Visitor for GameParser {
    type Tags = GameTags;
    type Movetext = GameMoves;
    type Output = Option<PgnGame>;

    fn begin_tags(&mut self) -> ControlFlow<Self::Output, Self::Tags> {
        ControlFlow::Continue(GameTags::default())
    }

    fn tag(
        &mut self,
        tags: &mut Self::Tags,
        name: &[u8],
        value: RawTag<'_>,
    ) -> ControlFlow<Self::Output> {
        let name_str = String::from_utf8_lossy(name);
        let value_str = value.decode_utf8_lossy().to_string();

        match name_str.as_ref() {
            "Event" => tags.event = Some(value_str),
            "Site" => tags.site = Some(value_str),
            "Date" => tags.date = Some(value_str),
            "White" => tags.white = Some(value_str),
            "Black" => tags.black = Some(value_str),
            "Result" => tags.result = Some(value_str),
            "WhiteElo" => tags.white_elo = value_str.parse().ok(),
            "BlackElo" => tags.black_elo = value_str.parse().ok(),
            _ => {}
        }

        ControlFlow::Continue(())
    }

    fn begin_movetext(&mut self, tags: Self::Tags) -> ControlFlow<Self::Output, Self::Movetext> {
        ControlFlow::Continue(GameMoves {
            tags,
            moves: Vec::new(),
            current_position: Chess::default(),
            success: true,
        })
    }

    fn san(&mut self, movetext: &mut Self::Movetext, san: SanPlus) -> ControlFlow<Self::Output> {
        if !movetext.success {
            return ControlFlow::Continue(());
        }

        movetext.moves.push(san.san.to_string());

        match san.san.to_move(&movetext.current_position) {
            Ok(m) => {
                match movetext.current_position.clone().play(m) {
                    Ok(new_pos) => {
                        movetext.current_position = new_pos;
                    }
                    Err(_) => {
                        movetext.success = false;
                    }
                }
            }
            Err(_) => {
                movetext.success = false;
            }
        }

        ControlFlow::Continue(())
    }

    fn begin_variation(
        &mut self,
        _movetext: &mut Self::Movetext,
    ) -> ControlFlow<Self::Output, Skip> {
        ControlFlow::Continue(Skip(true))
    }

    fn end_game(&mut self, movetext: Self::Movetext) -> Self::Output {
        if movetext.success {
            Some(PgnGame {
                event: movetext.tags.event,
                site: movetext.tags.site,
                date: movetext.tags.date,
                white: movetext.tags.white,
                black: movetext.tags.black,
                result: movetext.tags.result,
                white_elo: movetext.tags.white_elo,
                black_elo: movetext.tags.black_elo,
                moves: movetext.moves,
                final_position: movetext.current_position,
            })
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub enum PgnError {
    FileError(io::Error),
    NoGamesFound,
    ParseError(String),
}

impl std::fmt::Display for PgnError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PgnError::FileError(e) => write!(f, "File error: {}", e),
            PgnError::NoGamesFound => write!(f, "No valid games found in PGN"),
            PgnError::ParseError(s) => write!(f, "Parse error: {}", s),
        }
    }
}

impl From<io::Error> for PgnError {
    fn from(error: io::Error) -> Self {
        PgnError::FileError(error)
    }
}

pub fn parse_pgn_file<P: AsRef<Path>>(path: P) -> Result<Vec<PgnGame>, PgnError> {
    let contents = fs::read_to_string(path)?;
    parse_pgn_string(&contents)
}

pub fn parse_pgn_string(pgn: &str) -> Result<Vec<PgnGame>, PgnError> {
    let mut parser = GameParser;
    let mut games: Vec<PgnGame> = Vec::new();

    let cursor = Cursor::new(pgn.as_bytes());
    let mut reader = pgn_reader::Reader::new(cursor);

    loop {
        match reader.read_game(&mut parser) {
            Ok(Some(maybe_game)) => {
                if let Some(game) = maybe_game {
                    games.push(game);
                }
            }
            Ok(None) => break,
            Err(e) => return Err(PgnError::ParseError(e.to_string())),
        }
    }

    if games.is_empty() {
        Err(PgnError::NoGamesFound)
    } else {
        Ok(games)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use shakmaty::Color;

    const SAMPLE_PGN: &str = r#"[Event "Test"]
[White "Alice"]
[Black "Bob"]
[Result "1-0"]

1. e4 e5 2. Nf3 Nc6 3. Bb5 1-0
"#;

    #[test]
    fn test_parse_pgn_string() {
        let games = parse_pgn_string(SAMPLE_PGN).unwrap();
        assert_eq!(games.len(), 1);

        let game = &games[0];
        assert_eq!(game.white.as_deref(), Some("Alice"));
        assert_eq!(game.black.as_deref(), Some("Bob"));
        assert_eq!(game.result.as_deref(), Some("1-0"));
        assert_eq!(game.move_count(), 5);
    }

    #[test]
    fn test_game_summary() {
        let games = parse_pgn_string(SAMPLE_PGN).unwrap();
        let summary = games[0].summary();
        assert_eq!(summary, "Alice vs Bob - 1-0");
    }

    #[test]
    fn test_position_tracking() {
        let games = parse_pgn_string(SAMPLE_PGN).unwrap();
        let game = &games[0];
        assert_eq!(game.final_position.turn(), Color::Black);
        assert_eq!(game.final_position.board().occupied().count(), 32);
    }
}
