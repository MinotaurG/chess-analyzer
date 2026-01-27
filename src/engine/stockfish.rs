//! Stockfish chess engine interface
//!
//! Spawns Stockfish as a subprocess and communicates via UCI protocol.

use std::io::{BufRead, BufReader, Write};
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::time::Duration;

use super::analysis::{Evaluation, PositionAnalysis};

/// Error type for engine operations
#[derive(Debug)]
pub enum EngineError {
    /// Failed to start the engine process
    SpawnError(String),
    /// Failed to communicate with engine
    IoError(std::io::Error),
    /// Engine returned unexpected response
    ProtocolError(String),
    /// Engine not initialized
    NotInitialized,
}

impl std::fmt::Display for EngineError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EngineError::SpawnError(s) => write!(f, "Failed to start engine: {}", s),
            EngineError::IoError(e) => write!(f, "I/O error: {}", e),
            EngineError::ProtocolError(s) => write!(f, "Protocol error: {}", s),
            EngineError::NotInitialized => write!(f, "Engine not initialized"),
        }
    }
}

impl From<std::io::Error> for EngineError {
    fn from(error: std::io::Error) -> Self {
        EngineError::IoError(error)
    }
}

/// Wrapper around Stockfish chess engine
pub struct StockfishEngine {
    /// The child process
    process: Child,
    /// Stdin for sending commands
    stdin: ChildStdin,
    /// Stdout reader for receiving responses
    stdout: BufReader<ChildStdout>,
    /// Whether UCI handshake completed
    initialized: bool,
}

impl StockfishEngine {
    /// Creates a new Stockfish engine instance
    ///
    /// # Arguments
    /// * `path` - Path to stockfish binary (or "stockfish" if in PATH)
    ///
    /// # Example
    /// ```ignore
    /// let mut engine = StockfishEngine::new("stockfish")?;
    /// ```
    pub fn new(path: &str) -> Result<Self, EngineError> {
        // Spawn Stockfish process
        let mut process = Command::new(path)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null()) // Ignore stderr
            .spawn()
            .map_err(|e| EngineError::SpawnError(e.to_string()))?;

        // Get handles to stdin/stdout
        let stdin = process
            .stdin
            .take()
            .ok_or_else(|| EngineError::SpawnError("Failed to open stdin".into()))?;

        let stdout = process
            .stdout
            .take()
            .ok_or_else(|| EngineError::SpawnError("Failed to open stdout".into()))?;

        let mut engine = StockfishEngine {
            process,
            stdin,
            stdout: BufReader::new(stdout),
            initialized: false,
        };

        // Initialize UCI protocol
        engine.init_uci()?;

        Ok(engine)
    }

    /// Sends a command to the engine
    fn send(&mut self, cmd: &str) -> Result<(), EngineError> {
        writeln!(self.stdin, "{}", cmd)?;
        self.stdin.flush()?;
        Ok(())
    }

    /// Reads a line from the engine
    fn read_line(&mut self) -> Result<String, EngineError> {
        let mut line = String::new();
        self.stdout.read_line(&mut line)?;
        Ok(line.trim().to_string())
    }

    /// Reads lines until we get the expected response
    fn read_until(&mut self, expected: &str) -> Result<Vec<String>, EngineError> {
        let mut lines = Vec::new();
        loop {
            let line = self.read_line()?;
            let done = line.starts_with(expected);
            lines.push(line);
            if done {
                break;
            }
        }
        Ok(lines)
    }

    /// Initialize UCI protocol
    fn init_uci(&mut self) -> Result<(), EngineError> {
        self.send("uci")?;
        self.read_until("uciok")?;

        self.send("isready")?;
        self.read_until("readyok")?;

        self.initialized = true;
        Ok(())
    }

    /// Sets a position from a FEN string
    ///
    /// # Arguments
    /// * `fen` - FEN string, or None for starting position
    /// * `moves` - Optional list of moves to play from the position
    pub fn set_position(&mut self, fen: Option<&str>, moves: Option<&[String]>) -> Result<(), EngineError> {
        if !self.initialized {
            return Err(EngineError::NotInitialized);
        }

        let pos_str = match fen {
            Some(f) => format!("position fen {}", f),
            None => "position startpos".to_string(),
        };

        let cmd = match moves {
            Some(m) if !m.is_empty() => format!("{} moves {}", pos_str, m.join(" ")),
            _ => pos_str,
        };

        self.send(&cmd)?;
        Ok(())
    }

    /// Analyzes the current position
    ///
    /// # Arguments
    /// * `depth` - How many moves ahead to search
    ///
    /// # Returns
    /// Analysis results including best move and evaluation
    pub fn analyze(&mut self, depth: u8) -> Result<PositionAnalysis, EngineError> {
        if !self.initialized {
            return Err(EngineError::NotInitialized);
        }

        self.send(&format!("go depth {}", depth))?;

        let mut best_move = String::new();
        let mut evaluation = Evaluation::Centipawns(0);
        let mut pv = Vec::new();
        let mut final_depth = 0u8;
        let mut time_ms = 0u64;
        let mut nodes = 0u64;

        // Read until we get bestmove
        loop {
            let line = self.read_line()?;

            if line.starts_with("bestmove") {
                // Parse: "bestmove e2e4 ponder e7e5"
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    best_move = parts[1].to_string();
                }
                break;
            } else if line.starts_with("info") {
                // Parse info line
                self.parse_info_line(&line, &mut evaluation, &mut pv, &mut final_depth, &mut time_ms, &mut nodes);
            }
        }

        Ok(PositionAnalysis {
            best_move,
            evaluation,
            depth: final_depth,
            pv,
            time_ms,
            nodes,
        })
    }

    /// Parses an info line from Stockfish
    fn parse_info_line(
        &self,
        line: &str,
        evaluation: &mut Evaluation,
        pv: &mut Vec<String>,
        depth: &mut u8,
        time_ms: &mut u64,
        nodes: &mut u64,
    ) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        let mut i = 0;

        while i < parts.len() {
            match parts[i] {
                "depth" => {
                    if i + 1 < parts.len() {
                        *depth = parts[i + 1].parse().unwrap_or(0);
                    }
                    i += 2;
                }
                "score" => {
                    if i + 2 < parts.len() {
                        match parts[i + 1] {
                            "cp" => {
                                if let Ok(cp) = parts[i + 2].parse::<i32>() {
                                    *evaluation = Evaluation::Centipawns(cp);
                                }
                            }
                            "mate" => {
                                if let Ok(m) = parts[i + 2].parse::<i32>() {
                                    *evaluation = Evaluation::Mate(m);
                                }
                            }
                            _ => {}
                        }
                    }
                    i += 3;
                }
                "time" => {
                    if i + 1 < parts.len() {
                        *time_ms = parts[i + 1].parse().unwrap_or(0);
                    }
                    i += 2;
                }
                "nodes" => {
                    if i + 1 < parts.len() {
                        *nodes = parts[i + 1].parse().unwrap_or(0);
                    }
                    i += 2;
                }
                "pv" => {
                    // Everything after "pv" is the principal variation
                    *pv = parts[i + 1..].iter().map(|s| s.to_string()).collect();
                    break;
                }
                _ => {
                    i += 1;
                }
            }
        }
    }

    /// Quick evaluation - just get best move and score
    pub fn quick_eval(&mut self, depth: u8) -> Result<(String, Evaluation), EngineError> {
        let analysis = self.analyze(depth)?;
        Ok((analysis.best_move, analysis.evaluation))
    }

    /// Check if a move is the best move
    pub fn is_best_move(&mut self, player_move: &str, depth: u8) -> Result<bool, EngineError> {
        let analysis = self.analyze(depth)?;
        Ok(analysis.best_move == player_move)
    }

    /// Quit the engine cleanly
    pub fn quit(&mut self) -> Result<(), EngineError> {
        self.send("quit")?;
        // Give it a moment to exit
        std::thread::sleep(Duration::from_millis(100));
        let _ = self.process.kill(); // Kill if still running
        Ok(())
    }
}

impl Drop for StockfishEngine {
    fn drop(&mut self) {
        let _ = self.quit();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore] // Ignore by default - requires stockfish installed
    fn test_stockfish_init() {
        let engine = StockfishEngine::new("stockfish");
        assert!(engine.is_ok());
    }

    #[test]
    #[ignore]
    fn test_analyze_starting_position() {
        let mut engine = StockfishEngine::new("stockfish").unwrap();
        engine.set_position(None, None).unwrap();
        let analysis = engine.analyze(10).unwrap();

        assert!(!analysis.best_move.is_empty());
        println!("Best move: {}", analysis.best_move);
        println!("Evaluation: {}", analysis.evaluation);
    }
}
