// Import from our library (the crate name is chess_analyzer)
use chess_analyzer::{analyze_position, starting_position};

fn main() {
    println!("♟️  Chess Analyzer");
    println!("==================");
    println!();
    
    // Create starting position
    let position = starting_position();
    
    // Analyze it
    let info = analyze_position(&position);
    
    // Print results
    println!("📊 Starting Position Analysis:");
    println!("   Pieces on board: {}", info.piece_count);
    println!("   Legal moves: {}", info.legal_move_count);
    println!("   Side to move: {:?}", info.side_to_move);
    println!("   In check: {}", info.is_check);
}
