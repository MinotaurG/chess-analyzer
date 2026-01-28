//! Database operations

use rusqlite::{Connection, params};
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use super::models::*;
use crate::error::Result;
use crate::lichess::LichessGame;
use crate::patterns::DetectedPattern;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = Connection::open(path)?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS games (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lichess_id TEXT UNIQUE NOT NULL,
                white_username TEXT NOT NULL,
                black_username TEXT NOT NULL,
                white_rating INTEGER,
                black_rating INTEGER,
                result TEXT NOT NULL,
                speed TEXT NOT NULL,
                rated INTEGER NOT NULL,
                opening_eco TEXT,
                opening_name TEXT,
                moves TEXT NOT NULL,
                pgn TEXT,
                analyzed INTEGER NOT NULL DEFAULT 0,
                played_at INTEGER NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS patterns (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                game_id INTEGER NOT NULL,
                move_number INTEGER NOT NULL,
                pattern_type TEXT NOT NULL,
                subtype TEXT,
                severity TEXT NOT NULL,
                centipawn_loss INTEGER,
                position_fen TEXT NOT NULL,
                description TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                FOREIGN KEY (game_id) REFERENCES games(id)
            );

            CREATE TABLE IF NOT EXISTS user_settings (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lichess_username TEXT UNIQUE NOT NULL,
                lichess_token TEXT,
                games_synced_at INTEGER,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_games_lichess_id ON games(lichess_id);
            CREATE INDEX IF NOT EXISTS idx_games_played_at ON games(played_at);
            CREATE INDEX IF NOT EXISTS idx_patterns_game_id ON patterns(game_id);
            CREATE INDEX IF NOT EXISTS idx_patterns_type ON patterns(pattern_type);
            "#,
        )?;
        Ok(())
    }

    fn now() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    pub fn insert_game(&self, game: &LichessGame) -> Result<i64> {
        let moves = game.moves.as_deref().unwrap_or("");
        let (eco, opening) = game.opening.as_ref()
            .map(|o| (Some(o.eco.clone()), Some(o.name.clone())))
            .unwrap_or((None, None));

        self.conn.execute(
            r#"
            INSERT OR IGNORE INTO games 
            (lichess_id, white_username, black_username, white_rating, black_rating,
             result, speed, rated, opening_eco, opening_name, moves, pgn, played_at, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)
            "#,
            params![
                game.id,
                game.white_username(),
                game.black_username(),
                game.white_rating(),
                game.black_rating(),
                game.result(),
                game.speed,
                game.rated,
                eco,
                opening,
                moves,
                game.pgn,
                game.last_move_at / 1000,
                Self::now(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn insert_games(&self, games: &[LichessGame]) -> Result<u32> {
        let mut count = 0;
        for game in games {
            if self.insert_game(game).is_ok() {
                count += 1;
            }
        }
        Ok(count)
    }

    pub fn insert_pattern(&self, game_id: i64, pattern: &DetectedPattern) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO patterns 
            (game_id, move_number, pattern_type, severity, centipawn_loss, position_fen, description, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                game_id,
                pattern.move_number,
                pattern.pattern_type.as_str(),
                pattern.severity.as_str(),
                pattern.cp_loss,
                pattern.fen_before,
                pattern.description,
                Self::now(),
            ],
        )?;

        Ok(self.conn.last_insert_rowid())
    }

    pub fn mark_game_analyzed(&self, game_id: i64) -> Result<()> {
        self.conn.execute(
            "UPDATE games SET analyzed = 1 WHERE id = ?1",
            params![game_id],
        )?;
        Ok(())
    }

    pub fn get_game(&self, id: i64) -> Result<Option<StoredGame>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM games WHERE id = ?1"
        )?;

        let game = stmt.query_row(params![id], |row| {
            Ok(StoredGame {
                id: row.get(0)?,
                lichess_id: row.get(1)?,
                white_username: row.get(2)?,
                black_username: row.get(3)?,
                white_rating: row.get(4)?,
                black_rating: row.get(5)?,
                result: row.get(6)?,
                speed: row.get(7)?,
                rated: row.get(8)?,
                opening_eco: row.get(9)?,
                opening_name: row.get(10)?,
                moves: row.get(11)?,
                pgn: row.get(12)?,
                analyzed: row.get(13)?,
                played_at: row.get(14)?,
                created_at: row.get(15)?,
            })
        }).ok();

        Ok(game)
    }

    pub fn get_all_games(&self) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM games ORDER BY played_at DESC"
        )?;

        let games = stmt.query_map([], |row| {
            Ok(StoredGame {
                id: row.get(0)?,
                lichess_id: row.get(1)?,
                white_username: row.get(2)?,
                black_username: row.get(3)?,
                white_rating: row.get(4)?,
                black_rating: row.get(5)?,
                result: row.get(6)?,
                speed: row.get(7)?,
                rated: row.get(8)?,
                opening_eco: row.get(9)?,
                opening_name: row.get(10)?,
                moves: row.get(11)?,
                pgn: row.get(12)?,
                analyzed: row.get(13)?,
                played_at: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(games)
    }

    pub fn get_recent_games(&self, limit: u32) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM games ORDER BY played_at DESC LIMIT ?1"
        )?;

        let games = stmt.query_map(params![limit], |row| {
            Ok(StoredGame {
                id: row.get(0)?,
                lichess_id: row.get(1)?,
                white_username: row.get(2)?,
                black_username: row.get(3)?,
                white_rating: row.get(4)?,
                black_rating: row.get(5)?,
                result: row.get(6)?,
                speed: row.get(7)?,
                rated: row.get(8)?,
                opening_eco: row.get(9)?,
                opening_name: row.get(10)?,
                moves: row.get(11)?,
                pgn: row.get(12)?,
                analyzed: row.get(13)?,
                played_at: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(games)
    }

    pub fn get_unanalyzed_games(&self, limit: u32) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM games WHERE analyzed = 0 ORDER BY played_at DESC LIMIT ?1"
        )?;

        let games = stmt.query_map(params![limit], |row| {
            Ok(StoredGame {
                id: row.get(0)?,
                lichess_id: row.get(1)?,
                white_username: row.get(2)?,
                black_username: row.get(3)?,
                white_rating: row.get(4)?,
                black_rating: row.get(5)?,
                result: row.get(6)?,
                speed: row.get(7)?,
                rated: row.get(8)?,
                opening_eco: row.get(9)?,
                opening_name: row.get(10)?,
                moves: row.get(11)?,
                pgn: row.get(12)?,
                analyzed: row.get(13)?,
                played_at: row.get(14)?,
                created_at: row.get(15)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(games)
    }

    pub fn count_games(&self) -> Result<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM games",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_patterns(&self) -> Result<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM patterns",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_all_patterns(&self) -> Result<Vec<StoredPattern>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM patterns ORDER BY id DESC"
        )?;

        let patterns = stmt.query_map([], |row| {
            Ok(StoredPattern {
                id: row.get(0)?,
                game_id: row.get(1)?,
                move_number: row.get(2)?,
                pattern_type: row.get(3)?,
                subtype: row.get(4)?,
                severity: row.get(5)?,
                centipawn_loss: row.get(6)?,
                position_fen: row.get(7)?,
                description: row.get(8)?,
                created_at: row.get(9)?,
            })
        })?.collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(patterns)
    }

    pub fn get_last_sync_time(&self, username: &str) -> Result<Option<u64>> {
        let time: Option<u64> = self.conn.query_row(
            "SELECT games_synced_at FROM user_settings WHERE lichess_username = ?1",
            params![username],
            |row| row.get(0),
        ).ok().flatten();
        Ok(time)
    }

    pub fn set_last_sync_time(&self, username: &str) -> Result<()> {
        self.conn.execute(
            r#"
            INSERT INTO user_settings (lichess_username, games_synced_at, created_at)
            VALUES (?1, ?2, ?3)
            ON CONFLICT(lichess_username) DO UPDATE SET games_synced_at = ?2
            "#,
            params![username, Self::now(), Self::now()],
        )?;
        Ok(())
    }
}
