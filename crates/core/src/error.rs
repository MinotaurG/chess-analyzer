//! Error types for chess-analyzer-core

use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("JSON parsing failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("Lichess API error: {0}")]
    Lichess(String),

    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),

    #[error("PGN parsing error: {0}")]
    Pgn(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

pub type Result<T> = std::result::Result<T, Error>;
