//! Quick test for Lichess API

use chess_analyzer_core::lichess::{LichessClient, GameExportParams};

fn main() {
    let username = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: lichess_test <username>");
        std::process::exit(1);
    });

    println!("Fetching games for: {}", username);

    let client = LichessClient::new().expect("Failed to create client");

    // Get user info first
    match client.get_user(&username) {
        Ok(user) => {
            println!("User: {}", user.username);
            if let Some(count) = user.count {
                println!("Total games: {}", count.all);
                println!("Win/Loss/Draw: {}/{}/{}", count.win, count.loss, count.draw);
            }
        }
        Err(e) => {
            eprintln!("Failed to get user: {}", e);
            std::process::exit(1);
        }
    }

    println!("\nFetching last 5 games...\n");

    let params = GameExportParams::new().max(5);
    
    match client.get_user_games(&username, &params) {
        Ok(games) => {
            println!("Found {} games:\n", games.len());
            for game in &games {
                println!("  {} vs {} [{}]", 
                    game.white_username(),
                    game.black_username(),
                    game.result()
                );
                if let Some(opening) = &game.opening {
                    println!("    Opening: {} ({})", opening.name, opening.eco);
                }
                println!("    Speed: {} | Rated: {}", game.speed, game.rated);
                println!();
            }
        }
        Err(e) => {
            eprintln!("Failed to fetch games: {}", e);
            std::process::exit(1);
        }
    }
}
