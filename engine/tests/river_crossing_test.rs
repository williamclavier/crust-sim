//! Test for river crossing and bridge pathfinding

use engine::action::Action;
use engine::entities::{Entity, EntityKind, TowerData};
use engine::state::GameState;
use engine::systems;
use shared::{PlayerId, Position};

#[test]
fn test_knight_crosses_river_to_tower() {
    // Create game state
    let mut state = GameState::new(12345);

    // Give Player 1 enough elixir
    state.players.get_mut(&PlayerId::Player1).unwrap().elixir = 10.0;

    // Spawn Knight on Player 1 side (below river at y=20)
    let knight_pos = Position::new(9.0, 20.0);
    let action = Action::PlayCard {
        player: PlayerId::Player1,
        card_name: "Knight".to_string(),
        position: knight_pos,
        level: 11,
    };
    state.apply_action(&action).unwrap();

    // Spawn a fake tower for Player 2 (above river at y=10)
    // Make it invincible so combat doesn't interfere with pathfinding
    let tower_pos = Position::new(9.0, 10.0);
    let tower = Entity::new(
        PlayerId::Player2,
        tower_pos,
        EntityKind::Tower(TowerData {
            base_hp: 999999.0, // Invincible for testing
            damage: 0.0,       // No damage
            range: 0.0,        // No range (won't target Knight)
            attack_speed: 999.0,
        }),
    );
    state.add_entity(tower);

    println!("=== River Crossing Test ===");
    println!("Knight starting position: {:?}", knight_pos);
    println!("Tower target position: {:?}", tower_pos);
    println!("River: y=15-16, Bridges: x=4-6 and x=11-13");
    println!();

    // Run simulation for 30 seconds
    let dt = 1.0 / 60.0; // 60 FPS
    let total_time = 30.0;
    let steps = (total_time / dt) as usize;

    for i in 0..steps {
        // Run systems (NO COMBAT - just movement)
        systems::movement::update(&mut state, dt);
        systems::lifecycle::update(&mut state, dt);

        state.tick += 1;

        // Print position every second for debugging
        if i % 60 == 0 {
            if let Some(knight) = state.entities.values().find(|e| {
                matches!(e.kind, EntityKind::Troop(_)) && e.owner == PlayerId::Player1
            }) {
                let time = i as f32 * dt;
                println!(
                    "t={:.1}s: Knight at ({:.2}, {:.2}), velocity=({:.2}, {:.2})",
                    time, knight.position.x, knight.position.y, knight.velocity.x, knight.velocity.y
                );

                // Check if knight crossed the river
                if knight.position.y < 15.0 {
                    println!("\n✅ SUCCESS: Knight crossed the river!");
                    println!("Final position: ({:.2}, {:.2})", knight.position.x, knight.position.y);
                    return;
                }
            }
        }
    }

    // Check final position
    if let Some(knight) = state.entities.values().find(|e| {
        matches!(e.kind, EntityKind::Troop(_)) && e.owner == PlayerId::Player1
    }) {
        println!("\n❌ FAILED: Knight did not cross the river in 30 seconds");
        println!("Final position: ({:.2}, {:.2})", knight.position.x, knight.position.y);
        println!("Expected: y < 15.0 (above river)");
        panic!("Knight failed to cross river");
    } else {
        panic!("Knight entity not found");
    }
}
