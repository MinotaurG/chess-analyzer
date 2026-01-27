use chess_analyzer::analyze_position;
use chess_analyzer::engine::StockfishEngine;
use chess_analyzer::parser::parse_pgn_file;
use shakmaty::{fen::Fen, san::San, uci::Uci, Chess, Position};
use std::env;
use std::process;

fn main() {
    println!("♟️  Chess Analyzer");
    println!("==================");
    println!();

    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        process::exit(1);
    }

    match args[1].as_str() {
        "analyze" => {
            if args.len() < 3 {
                println!("❌ Error: Please provide a PGN file");
                println!("Usage: {} analyze <pgn_file>", args[0]);
                process::exit(1);
            }
            analyze_games(&args[2]);
        }
        "eval" => {
            if args.len() < 3 {
                println!("❌ Error: Please provide a FEN string");
                println!("Usage: {} eval \"<fen>\"", args[0]);
                process::exit(1);
            }
            eval_position(&args[2]);
        }
        "test-engine" => {
            test_engine();
        }
        _ => {
            print_usage(&args[0]);
            process::exit(1);
        }
    }
}

fn print_usage(program: &str) {
    println!("Usage: {} <command> [arguments]", program);
    println!();
    println!("Commands:");
    println!("  analyze <pgn_file>   Analyze games from a PGN file");
    println!("  eval \"<fen>\"         Evaluate a position (FEN string)");
    println!("  test-engine          Test Stockfish connection");
    println!();
    println!("Examples:");
    println!("  {} analyze games.pgn", program);
    println!("  {} eval \"rnbqkbnr/pppppppp/8/8/4P3/8/PPPP1PPP/RNBQKBNR b KQkq - 0 1\"", program);
}

fn test_engine() {
    println!("🔧 Testing Stockfish connection...");
    println!();

    match StockfishEngine::new("stockfish") {
        Ok(mut engine) => {
            println!("✅ Stockfish started successfully!");
            println!();

            // Analyze starting position
            println!("📊 Analyzing starting position (depth 12)...");
            engine.set_position(None, None).unwrap();

            match engine.analyze(12) {
                Ok(analysis) => {
                    println!();
                    println!("   Best move: {}", analysis.best_move);
                    println!("   Evaluation: {}", analysis.evaluation);
                    println!("   Depth: {}", analysis.depth);
                    println!("   Nodes: {}", analysis.nodes);
                    println!("   Time: {}ms", analysis.time_ms);
                    println!("   PV: {}", analysis.pv.iter().take(5).cloned().collect::<Vec<_>>().join(" "));
                }
                Err(e) => {
                    println!("❌ Analysis failed: {}", e);
                }
            }

            // Test a tactical position
            println!();
            println!("📊 Analyzing tactical position...");
            let tactical_fen = "r1bqkb1r/pppp1ppp/2n2n2/4p2Q/2B1P3/8/PPPP1PPP/RNB1K1NR w KQkq - 4 4";
            engine.set_position(Some(tactical_fen), None).unwrap();

            match engine.analyze(15) {
                Ok(analysis) => {
                    println!("   Position: Scholar's Mate threat");
                    println!("   Best move: {} (should be Qxf7#)", analysis.best_move);
                    println!("   Evaluation: {}", analysis.evaluation);
                }
                Err(e) => {
                    println!("❌ Analysis failed: {}", e);
                }
            }

            println!();
            println!("✅ Engine test complete!");
        }
        Err(e) => {
            println!("❌ Failed to start Stockfish: {}", e);
            println!();
            println!("Make sure Stockfish is installed:");
            println!("  sudo apt install stockfish");
        }
    }
}

fn eval_position(fen: &str) {
    println!("📊 Evaluating position...");
    println!("   FEN: {}", fen);
    println!();

    // Parse FEN to validate it
    let parsed_fen: Result<Fen, _> = fen.parse();
    if let Err(e) = parsed_fen {
        println!("❌ Invalid FEN: {}", e);
        process::exit(1);
    }

    match StockfishEngine::new("stockfish") {
        Ok(mut engine) => {
            engine.set_position(Some(fen), None).unwrap();

            match engine.analyze(18) {
                Ok(analysis) => {
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                    println!("   Evaluation: {}", analysis.evaluation);
                    println!("   Best move: {}", analysis.best_move);
                    println!("   Depth: {}", analysis.depth);
                    println!("   Best line: {}", analysis.pv.iter().take(8).cloned().collect::<Vec<_>>().join(" "));
                    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                }
                Err(e) => {
                    println!("❌ Analysis failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("❌ Failed to start Stockfish: {}", e);
        }
    }
}

fn analyze_games(file_path: &str) {
    println!("📂 Loading: {}", file_path);
    println!();

    // Parse PGN
    let games = match parse_pgn_file(file_path) {
        Ok(g) => g,
        Err(e) => {
            println!("❌ Error: {}", e);
            process::exit(1);
        }
    };

    println!("✅ Found {} game(s)", games.len());
    println!();

    // Start engine
    let mut engine = match StockfishEngine::new("stockfish") {
        Ok(e) => {
            println!("✅ Stockfish engine ready");
            println!();
            e
        }
        Err(e) => {
            println!("⚠️  Stockfish not available: {}", e);
            println!("   Continuing without engine analysis...");
            println!();

            // Show basic info without engine
            for (index, game) in games.iter().enumerate() {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📋 Game {}: {}", index + 1, game.summary());
                println!("   Moves: {}", game.move_count());
                let info = analyze_position(&game.final_position);
                println!("   Final: {} pieces, {:?} to move", info.piece_count, info.side_to_move);
            }
            return;
        }
    };

    // Analyze each game
    for (index, game) in games.iter().enumerate() {
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        println!("📋 Game {}: {}", index + 1, game.summary());
        println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

        if let Some(event) = &game.event {
            println!("   Event: {}", event);
        }
        if let Some(date) = &game.date {
            println!("   Date: {}", date);
        }
        if let (Some(we), Some(be)) = (&game.white_elo, &game.black_elo) {
            println!("   Ratings: {} vs {}", we, be);
        }
        println!("   Moves: {}", game.move_count());
        println!();

        // Analyze key positions in the game
        println!("   📊 Position Analysis:");

        // Starting position
        engine.set_position(None, None).unwrap();
        if let Ok(analysis) = engine.analyze(12) {
            println!("      Start: {} ({})", analysis.evaluation, analysis.best_move);
        }

        // Position after opening (move 10)
        if game.moves.len() >= 20 {
            let opening_moves: Vec<String> = game.moves.iter().take(20).cloned().collect();
            engine.set_position(None, Some(&opening_moves)).unwrap();
            if let Ok(analysis) = engine.analyze(12) {
                println!("      Move 10: {} (best: {})", analysis.evaluation, analysis.best_move);
            }
        }

        // Final position
        if !game.moves.is_empty() {
            let all_moves: Vec<String> = convert_san_to_uci(&game.moves);
            if !all_moves.is_empty() {
                engine.set_position(None, Some(&all_moves)).unwrap();
                if let Ok(analysis) = engine.analyze(12) {
                    println!("      Final: {}", analysis.evaluation);
                }
            }
        }

        println!();
    }

    println!("✅ Analysis complete!");
}

/// Converts SAN moves to UCI format (simplified - just passes through for now)
/// TODO: Implement proper SAN to UCI conversion using position tracking
fn convert_san_to_uci(san_moves: &[String]) -> Vec<String> {
    // For now, we'll use a different approach in the next iteration
    // This is a placeholder that won't work correctly yet
    Vec::new()
}
