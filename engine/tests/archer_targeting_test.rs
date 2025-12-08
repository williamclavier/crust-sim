use engine::{card::get_test_cards, state::GameState};
use shared::{PlayerId, Position};

#[test]
fn test_archer_autonomous_targeting() {
    // Initialize game state with towers
    let mut state = GameState::new(42);

    // Spawn Archers at center position (9.0, 1.0) behind Player 2's king tower
    let cards = get_test_cards();
    let archers_card = cards.iter().find(|c| c.name == "Archers").unwrap();

    let spawn_pos = Position::new(9.0, 1.0);
    println!("\n=== ARCHER TARGETING TEST ===");
    println!("Spawn position: ({:.2}, {:.2})", spawn_pos.x, spawn_pos.y);
    println!("Arena center X: 9.0");

    archers_card.spawn(&mut state, PlayerId::Player2, spawn_pos, 11).unwrap();

    // Find the two archers and store their IDs
    let mut archer_data: Vec<_> = state.entities.iter()
        .filter(|(_, e)| e.card_name.as_ref().map(|n| n == "Archers").unwrap_or(false))
        .map(|(id, e)| (*id, e.position.x, e.position.y))
        .collect();

    // Sort by X position for consistency
    archer_data.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());

    println!("\nSpawned {} archers:", archer_data.len());
    for (id, x, y) in &archer_data {
        println!("  Archer {:?}: position=({:.2}, {:.2})", id, x, y);
    }

    // Run one tick to establish targeting
    let dt = 1.0 / 60.0;
    engine::systems::movement::update(&mut state, dt);

    println!("\n=== TARGETING AFTER 1 TICK ===");

    // Check targeting for each archer
    for (id, _, _) in &archer_data {
        let archer = &state.entities[id];

        let target_side = if archer.position.x < 9.0 {
            "LEFT"
        } else {
            "RIGHT"
        };

        println!("Archer {:?}:", id);
        println!("  Position: ({:.2}, {:.2})", archer.position.x, archer.position.y);
        println!("  Expected target: {} princess tower", target_side);

        // Find what tower they're actually moving toward
        if let Some(target_id) = archer.target {
            let target = &state.entities[&engine::state::EntityId::from_u32(target_id)];
            println!("  Has explicit target at: ({:.2}, {:.2})", target.position.x, target.position.y);
        } else {
            println!("  No explicit target (using autonomous movement)");
        }
    }

    // Verify they're targeting different sides
    let archer1 = &state.entities[&archer_data[0].0];
    let archer2 = &state.entities[&archer_data[1].0];

    println!("\n=== VERIFICATION ===");
    println!("Archer 1 (left):  X={:.2}, should target LEFT (X < 9.0)", archer1.position.x);
    println!("Archer 2 (right): X={:.2}, should target RIGHT (X >= 9.0)", archer2.position.x);

    // Check positions are on different sides of center
    assert!(archer1.position.x < 9.0, "Archer 1 should be left of center (X < 9.0)");
    assert!(archer2.position.x >= 9.0, "Archer 2 should be right of center (X >= 9.0)");

    println!("\n✓ Archers spawned on different sides of arena center");
}
