use std::io::{self, BufRead, Write};
use engine::state::{GameState, step_with_action};
use engine::card;
use shared::PlayerId;

fn main() {
    let stdin = io::stdin();
    let mut stdout = io::stdout();

    // Start with some default game; we'll replace it on RESET.
    let mut game = GameState::new(0);

    eprintln!("crust_sim_server ready. Commands: RESET <seed>, STATE, EXIT");

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(_) => break,
        };
        let parts: Vec<_> = line.trim().split_whitespace().collect();
        if parts.is_empty() {
            continue;
        }

        match parts[0] {
            "RESET" => {
                let seed: u64 = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                game = GameState::new(seed);
                
                // Set up decks for both players using test cards
                // Cycle through test cards to fill 8-card deck
                let test_cards = card::get_test_cards();
                let player1_deck: Vec<String> = test_cards.iter().cycle().take(8).map(|c| c.name.clone()).collect();
                let player2_deck: Vec<String> = test_cards.iter().rev().cycle().take(8).map(|c| c.name.clone()).collect();
                
                game.set_player_deck(shared::PlayerId::Player1, player1_deck)
                    .expect("Failed to set Player1 deck");
                game.set_player_deck(shared::PlayerId::Player2, player2_deck)
                    .expect("Failed to set Player2 deck");
                
                eprintln!(
                    "RESET: Player1 hand size = {}, Player2 hand size = {}",
                    game.players.get(&shared::PlayerId::Player1).unwrap().hand.len(),
                    game.players.get(&shared::PlayerId::Player2).unwrap().hand.len()
                );
                
                let snapshot = game.export_cr_state(PlayerId::Player1);
                let json = serde_json::to_string(&snapshot).unwrap();
                writeln!(stdout, "{}", json).unwrap();
                stdout.flush().unwrap();
            }
            "STATE" => {
                let snapshot = game.export_cr_state(PlayerId::Player1);
                let json = serde_json::to_string(&snapshot).unwrap();
                writeln!(stdout, "{}", json).unwrap();
                stdout.flush().unwrap();
            }
            "STEP" => {
                let card_idx: usize = parts.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);
                let tile_idx: usize = parts.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);

                eprintln!(
                    "DEBUG: STEP command received card_idx={}, tile_idx={}",
                    card_idx, tile_idx
                );

                step_with_action(&mut game, PlayerId::Player1, card_idx, tile_idx);

                let snapshot = game.export_cr_state(PlayerId::Player1);
                let json = serde_json::to_string(&snapshot).unwrap();
                writeln!(stdout, "{}", json).unwrap();
                stdout.flush().unwrap();
            }
            "EXIT" => {
                break;
            }
            _ => {
                eprintln!("Unknown command: {}", parts[0]);
            }
        }
    }
}