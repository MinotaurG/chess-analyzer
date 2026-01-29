//! Database operations

use rusqlite::{Connection, params, Row};
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

            CREATE TABLE IF NOT EXISTS training_sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                training_type TEXT NOT NULL,
                attempts INTEGER NOT NULL,
                correct INTEGER NOT NULL,
                total_time_ms INTEGER NOT NULL,
                best_time_ms INTEGER,
                date TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_games_lichess_id ON games(lichess_id);
            CREATE INDEX IF NOT EXISTS idx_games_played_at ON games(played_at);
            CREATE INDEX IF NOT EXISTS idx_patterns_game_id ON patterns(game_id);
            CREATE INDEX IF NOT EXISTS idx_patterns_type ON patterns(pattern_type);
            CREATE INDEX IF NOT EXISTS idx_training_date ON training_sessions(date);
            CREATE INDEX IF NOT EXISTS idx_training_type ON training_sessions(training_type);
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

    fn today() -> String {
        let now = Self::now();
        let days = now / 86400;
        format!("{}", days)
    }

    // ========================================================================
    // GAMES
    // ========================================================================

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

    fn row_to_game(row: &Row) -> rusqlite::Result<StoredGame> {
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
    }

    pub fn get_game(&self, id: i64) -> Result<Option<StoredGame>> {
        let mut stmt = self.conn.prepare("SELECT * FROM games WHERE id = ?1")?;
        let game = stmt.query_row(params![id], Self::row_to_game).ok();
        Ok(game)
    }

    pub fn get_all_games(&self) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare("SELECT * FROM games ORDER BY played_at DESC")?;
        let games = stmt.query_map([], Self::row_to_game)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(games)
    }

    pub fn get_recent_games(&self, limit: u32) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare("SELECT * FROM games ORDER BY played_at DESC LIMIT ?1")?;
        let games = stmt.query_map(params![limit], Self::row_to_game)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        Ok(games)
    }

    pub fn get_unanalyzed_games(&self, limit: u32) -> Result<Vec<StoredGame>> {
        let mut stmt = self.conn.prepare(
            "SELECT * FROM games WHERE analyzed = 0 ORDER BY played_at DESC LIMIT ?1"
        )?;
        let games = stmt.query_map(params![limit], Self::row_to_game)?
            .collect::<std::result::Result<Vec<_>, _>>()?;
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

    // ========================================================================
    // PATTERNS
    // ========================================================================

    pub fn count_patterns(&self) -> Result<u32> {
        let count: u32 = self.conn.query_row(
            "SELECT COUNT(*) FROM patterns",
            [],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn get_all_patterns(&self) -> Result<Vec<StoredPattern>> {
        let mut stmt = self.conn.prepare("SELECT * FROM patterns ORDER BY id DESC")?;
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

    // ========================================================================
    // USER SETTINGS
    // ========================================================================

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

    // ========================================================================
    // TRAINING
    // ========================================================================

    pub fn save_training_session(
        &self,
        training_type: &str,
        attempts: u32,
        correct: u32,
        total_time_ms: u64,
        best_time_ms: Option<u64>,
    ) -> Result<i64> {
        self.conn.execute(
            r#"
            INSERT INTO training_sessions 
            (training_type, attempts, correct, total_time_ms, best_time_ms, date, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                training_type,
                attempts,
                correct,
                total_time_ms,
                best_time_ms,
                Self::today(),
                Self::now(),
            ],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn get_training_stats(&self, training_type: &str) -> Result<TrainingStats> {
        let today = Self::today();

        let (today_attempts, today_correct): (u32, u32) = self.conn.query_row(
            r#"
            SELECT COALESCE(SUM(attempts), 0), COALESCE(SUM(correct), 0)
            FROM training_sessions
            WHERE training_type = ?1 AND date = ?2
            "#,
            params![training_type, today],
            |row| Ok((row.get(0)?, row.get(1)?)),
        ).unwrap_or((0, 0));

        let (total_attempts, total_correct, total_time): (u32, u32, u64) = self.conn.query_row(
            r#"
            SELECT COALESCE(SUM(attempts), 0), COALESCE(SUM(correct), 0), COALESCE(SUM(total_time_ms), 0)
            FROM training_sessions
            WHERE training_type = ?1
            "#,
            params![training_type],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).unwrap_or((0, 0, 0));

        let best_time: Option<u64> = self.conn.query_row(
            "SELECT MIN(best_time_ms) FROM training_sessions WHERE training_type = ?1 AND best_time_ms IS NOT NULL",
            params![training_type],
            |row| row.get(0),
        ).ok().flatten();

        let streak = self.calculate_streak(training_type).unwrap_or(0);

        Ok(TrainingStats {
            today_attempts,
            today_correct,
            total_attempts,
            total_correct,
            total_time_ms: total_time,
            best_time_ms: best_time,
            streak,
        })
    }

    fn calculate_streak(&self, training_type: &str) -> Result<u32> {
        let today: i64 = Self::today().parse().unwrap_or(0);
        
        let mut stmt = self.conn.prepare(
            "SELECT DISTINCT date FROM training_sessions WHERE training_type = ?1 ORDER BY date DESC"
        )?;

        let dates: Vec<i64> = stmt.query_map(params![training_type], |row| {
            let date_str: String = row.get(0)?;
            Ok(date_str.parse::<i64>().unwrap_or(0))
        })?.filter_map(|r| r.ok()).collect();

        if dates.is_empty() {
            return Ok(0);
        }

        let mut streak = 0u32;
        let mut expected = today;

        for date in dates {
            if date == expected || date == expected - 1 {
                streak += 1;
                expected = date - 1;
            } else {
                break;
            }
        }

        Ok(streak)
    }

    pub fn get_all_training_stats(&self) -> Result<AllTrainingStats> {
        let coords = self.get_training_stats("coordinates")?;
        let viz = self.get_training_stats("visualization")?;
        let openings = self.get_training_stats("openings")?;

        let total_today = coords.today_attempts + viz.today_attempts + openings.today_attempts;
        let total_all = coords.total_attempts + viz.total_attempts + openings.total_attempts;
        let total_correct = coords.total_correct + viz.total_correct + openings.total_correct;
        let accuracy = if total_all > 0 {
            ((total_correct as f64 / total_all as f64) * 100.0) as u32
        } else {
            0
        };

        let max_streak = coords.streak.max(viz.streak).max(openings.streak);

        Ok(AllTrainingStats {
            coordinates: coords,
            visualization: viz,
            openings,
            today_total: total_today,
            all_time_total: total_all,
            overall_accuracy: accuracy,
            max_streak,
        })
    }
}
