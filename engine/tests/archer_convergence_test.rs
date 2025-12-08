use engine::{card::get_test_cards, state::GameState};
use shared::{PlayerId, Position};

#[test]
fn test_archer_convergence_over_time() {
    let mut state = GameState::new(42);

    let cards = get_test_cards();
    let archers_card = cards.iter().find(|c| c.name == "Archers").unwrap();

    let spawn_pos = Position::new(9.0, 1.0);
    archers_card.spawn(&mut state, PlayerId::Player2, spawn_pos, 11).unwrap();

    println!("\n=== ARCHER CONVERGENCE TEST ===");
    println!("Spawn position: (9.00, 1.00)\n");

    // Track archer positions over 5 seconds
    let dt = 1.0 / 60.0;
    for second in 0..=5 {
        if second > 0 {
            // Run 60 ticks (1 second)
            for _ in 0..60 {
                engine::systems::movement::update(&mut state, dt);
            }
        }

        let mut archers: Vec<_> = state.entities.iter()
            .filter(|(_, e)| e.card_name.as_ref().map(|n| n == "Archers").unwrap_or(false))
            .map(|(id, e)| (*id, e.position.x, e.position.y))
            .collect();

        archers.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

        println!("Second {}:", second);
        for (i, (id, x, y)) in archers.iter().enumerate() {
            let side = if *x < 9.0 { "LEFT" } else { "RIGHT" };
            println!("  Archer {} ({:?}): X={:.2}, Y={:.2} -> targets {} tower",
                i+1, id, x, y, side);
        }

        // Check distance between archers
        if archers.len() == 2 {
            let distance = (archers[1].1 - archers[0].1).abs();
            println!("  Distance apart: {:.2} tiles", distance);

            // Check if both are on same side
            let both_left = archers[0].1 < 9.0 && archers[1].1 < 9.0;
            let both_right = archers[0].1 >= 9.0 && archers[1].1 >= 9.0;

            if both_left {
                println!("  ⚠️  BOTH ARCHERS ON LEFT SIDE - will target left tower!");
            } else if both_right {
                println!("  ⚠️  BOTH ARCHERS ON RIGHT SIDE - will target right tower!");
            }
        }

        println!();
    }
}
