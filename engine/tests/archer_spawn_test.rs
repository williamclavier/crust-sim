use engine::{card::get_test_cards, state::GameState};
use shared::{PlayerId, Position};

#[test]
fn test_archer_spawn_positions() {
    // Initialize game state
    let mut state = GameState::new(42);

    // Spawn Archers at position (10.0, 20.0)
    let cards = get_test_cards();
    let archers_card = cards.iter().find(|c| c.name == "Archers").unwrap();

    let spawn_pos = Position::new(10.0, 20.0);
    println!("\n=== ARCHER SPAWN TEST ===");
    println!("Spawn click position: ({}, {})", spawn_pos.x, spawn_pos.y);
    println!("Deploy pattern: {:?}", archers_card.deploy_pattern);

    archers_card.spawn(&mut state, PlayerId::Player1, spawn_pos, 11).unwrap();

    // Find the two archers
    let archers: Vec<_> = state.entities.values()
        .filter(|e| e.card_name.as_ref().map(|n| n == "Archers").unwrap_or(false))
        .collect();

    println!("\nSpawned {} archers", archers.len());
    assert_eq!(archers.len(), 2, "Should spawn 2 archers");

    // Get positions
    let pos1 = archers[0].position;
    let pos2 = archers[1].position;

    println!("Archer 1: ({:.2}, {:.2})", pos1.x, pos1.y);
    println!("Archer 2: ({:.2}, {:.2})", pos2.x, pos2.y);

    // With deploy_pattern [(-1.0, 0.0), (1.0, 0.0)]
    // Expected positions:
    // - Archer 1: (10.0 - 1.0, 20.0) = (9.0, 20.0)
    // - Archer 2: (10.0 + 1.0, 20.0) = (11.0, 20.0)

    println!("\nExpected:");
    println!("  Archer 1: (9.00, 20.00)");
    println!("  Archer 2: (11.00, 20.00)");
    println!("  Distance: 2.00 tiles");

    let distance = ((pos2.x - pos1.x).powi(2) + (pos2.y - pos1.y).powi(2)).sqrt();
    println!("\nActual distance: {:.2} tiles", distance);

    // Check if they're 2 tiles apart
    assert!((distance - 2.0).abs() < 0.01,
        "Archers should be 2 tiles apart, got {:.2}", distance);

    println!("\n✓ Archers spawned correctly with 2-tile spacing");
}
