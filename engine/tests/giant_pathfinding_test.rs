use engine::{card::get_test_cards, state::GameState};
use shared::{PlayerId, Position};

#[test]
fn test_giant_pathfinding_behind_king() {
    let mut state = GameState::new(42);

    let cards = get_test_cards();
    let giant_card = cards.iter().find(|c| c.name == "Giant").unwrap();

    // Spawn Giant at (8.5, 31.5) - left of center, behind Player 1 king tower
    let spawn_pos = Position::new(8.5, 31.5);
    println!("\n=== GIANT PATHFINDING TEST ===");
    println!("Spawn position: ({:.2}, {:.2})", spawn_pos.x, spawn_pos.y);
    println!("King tower at: (9.0, 29.0)");
    println!("Left princess tower at: (3.5, 25.5)");
    println!("Right princess tower at: (14.5, 25.5)");
    println!();

    giant_card.spawn(&mut state, PlayerId::Player1, spawn_pos, 11).unwrap();

    // Track Giant movement for 10 seconds
    let dt = 1.0 / 60.0;
    for second in 0..=10 {
        if second > 0 {
            for _ in 0..60 {
                engine::systems::movement::update(&mut state, dt);
            }
        }

        let giant = state.entities.values()
            .find(|e| e.card_name.as_ref().map(|n| n == "Giant").unwrap_or(false))
            .unwrap();

        let relative_to_king = if giant.position.x < 9.0 {
            format!("LEFT of king (X={:.2} < 9.0)", giant.position.x)
        } else if giant.position.x > 9.0 {
            format!("RIGHT of king (X={:.2} > 9.0)", giant.position.x)
        } else {
            "EXACTLY at king X".to_string()
        };

        println!("Second {:2}: Giant at ({:.2}, {:.2}) - {}",
            second, giant.position.x, giant.position.y, relative_to_king);
    }

    let giant = state.entities.values()
        .find(|e| e.card_name.as_ref().map(|n| n == "Giant").unwrap_or(false))
        .unwrap();

    println!("\n=== ANALYSIS ===");
    println!("Started at: (8.50, 31.50) - left of king");
    println!("Ended at: ({:.2}, {:.2})", giant.position.x, giant.position.y);

    if giant.position.x < 9.0 {
        println!("✓ Stayed on LEFT side - should go to left princess tower");
    } else {
        println!("✗ Crossed to RIGHT side - BUG: should stay left!");
    }
}
