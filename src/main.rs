use chess_analyzer::analyze_position;
use chess_analyzer::parser::parse_pgn_file;
use std::env;
use std::process;

fn main() {
    println!("♟️  Chess Analyzer");
    println!("==================");
    println!();
    
    // Get command line arguments
    let args: Vec<String> = env::args().collect();
    
    // Check if a file path was provided
    if args.len() < 2 {
        println!("Usage: {} <pgn_file>", args[0]);
        println!();
        println!("Example: {} data/sample.pgn", args[0]);
        process::exit(1);
    }
    
    let file_path = &args[1];
    
    println!("📂 Loading: {}", file_path);
    println!();
    
    // Parse the PGN file - here's where we handle the Result!
    match parse_pgn_file(file_path) {
        Ok(games) => {
            println!("✅ Found {} game(s)", games.len());
            println!();
            
            // Analyze each game
            for (index, game) in games.iter().enumerate() {
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                println!("📋 Game {}: {}", index + 1, game.summary());
                println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
                
                // Print metadata
                if let Some(event) = &game.event {
                    println!("   Event: {}", event);
                }
                if let Some(date) = &game.date {
                    println!("   Date: {}", date);
                }
                if let Some(site) = &game.site {
                    println!("   Site: {}", site);
                }
                
                // Print ratings if available
                match (&game.white_elo, &game.black_elo) {
                    (Some(w), Some(b)) => println!("   Ratings: {} vs {}", w, b),
                    (Some(w), None) => println!("   White Elo: {}", w),
                    (None, Some(b)) => println!("   Black Elo: {}", b),
                    (None, None) => {}
                }
                
                println!("   Moves: {}", game.move_count());
                
                // Analyze final position
                let final_analysis = analyze_position(&game.final_position);
                
                println!();
                println!("   📊 Final Position:");
                println!("      Pieces remaining: {}", final_analysis.piece_count);
                println!("      Side to move: {:?}", final_analysis.side_to_move);
                
                if final_analysis.is_checkmate {
                    println!("      ⚔️  CHECKMATE!");
                } else if final_analysis.is_stalemate {
                    println!("      🤝 Stalemate");
                } else if final_analysis.is_check {
                    println!("      ⚠️  In check");
                }
                
                // Show first few moves
                println!();
                println!("   📝 Opening moves:");
                let preview_moves: Vec<&String> = game.moves.iter().take(10).collect();
                print!("      ");
                for (i, mv) in preview_moves.iter().enumerate() {
                    if i % 2 == 0 {
                        print!("{}.", (i / 2) + 1);
                    }
                    print!("{} ", mv);
                }
                if game.moves.len() > 10 {
                    print!("...");
                }
                println!();
                println!();
            }
        }
        Err(e) => {
            // Handle errors nicely
            println!("❌ Error: {}", e);
            process::exit(1);
        }
    }
}
