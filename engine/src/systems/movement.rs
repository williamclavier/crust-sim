//! Movement system for entities.

use crate::state::{EntityId, GameState};
use shared::{Position, Velocity};

/// Updates entity movement - sets velocity toward targets and applies movement.
pub fn update(state: &mut GameState, dt: f32) {
    // First pass: Update velocities based on targets
    let mut velocity_updates: Vec<(EntityId, Velocity)> = Vec::new();

    for (id, entity) in &state.entities {
        // Only move troops (not towers)
        if !entity.can_move() {
            continue;
        }

        // Check if entity has a target
        if let Some(target_id) = entity.target {
            let target_entity_id = EntityId::from_u32(target_id);

            // Get target position (if target still exists)
            if let Some(target) = state.entities.get(&target_entity_id) {
                let distance = entity.position.distance_to(&target.position);
                let attack_range = entity.attack_range();

                // If target is out of range, move toward it (possibly via bridge waypoint)
                if distance > attack_range {
                    let move_speed = entity.movement_speed();

                    // Check if we need to path through a bridge
                    let move_target = if entity.can_cross_river() {
                        // Air units / jumping units: go direct
                        target.position
                    } else {
                        // Ground units: check if target is across river
                        get_movement_waypoint(&entity.position, &target.position, &state.arena)
                    };

                    let (dir_x, dir_y) = entity.position.direction_to(&move_target);

                    velocity_updates.push((
                        *id,
                        Velocity::new(dir_x * move_speed, dir_y * move_speed),
                    ));
                } else {
                    // Target in range - stop moving
                    velocity_updates.push((*id, Velocity::zero()));
                }
            } else {
                // Target doesn't exist anymore - stop
                velocity_updates.push((*id, Velocity::zero()));
            }
        } else {
            // No target - move autonomously toward enemy towers
            // Based on legacy engine autonomous pathfinding (lines 6286-6453)
            let move_target = get_autonomous_movement_target(state, entity);

            if let Some(target_pos) = move_target {
                let move_speed = entity.movement_speed();

                // Apply waypoint pathfinding for ground units
                let waypoint = if entity.can_cross_river() {
                    target_pos
                } else {
                    get_movement_waypoint(&entity.position, &target_pos, &state.arena)
                };

                let (dir_x, dir_y) = entity.position.direction_to(&waypoint);

                velocity_updates.push((
                    *id,
                    Velocity::new(dir_x * move_speed, dir_y * move_speed),
                ));
            } else {
                // No enemies at all - stop
                velocity_updates.push((*id, Velocity::zero()));
            }
        }
    }

    // Apply velocity updates
    for (id, velocity) in velocity_updates {
        if let Some(entity) = state.entities.get_mut(&id) {
            entity.velocity = velocity;
        }
    }

    // Second pass: Apply velocities to positions with collision detection
    let mut position_updates: Vec<(EntityId, Position)> = Vec::new();

    for (id, entity) in &state.entities {
        // Skip if not moving
        if entity.velocity.x == 0.0 && entity.velocity.y == 0.0 {
            continue;
        }

        // Calculate new position
        let new_x = entity.position.x + entity.velocity.x * dt;
        let new_y = entity.position.y + entity.velocity.y * dt;
        let new_position = Position::new(new_x, new_y);

        // Check tile passability (river blocking)
        let tile_blocked = is_tile_blocked(state, entity, &new_position);

        // Check for collisions with other entities
        let would_collide = check_collision(state, *id, &new_position);

        if !tile_blocked && !would_collide {
            position_updates.push((*id, new_position));
        }
        // If tile blocked or collision detected, don't move (stay in current position)
    }

    // Apply position updates
    for (id, position) in position_updates {
        if let Some(entity) = state.entities.get_mut(&id) {
            entity.position = position;
        }
    }
}

/// Gets autonomous movement target when unit has no specific target.
/// Based on legacy engine autonomous pathfinding (lines 6286-6453).
///
/// Priority:
/// 1. Princess towers (left tower if X < 9, right tower if X >= 9)
/// 2. King tower (if both princess towers destroyed)
fn get_autonomous_movement_target(state: &GameState, entity: &crate::entities::Entity) -> Option<Position> {
    use crate::entities::EntityKind;
    use shared::PlayerId;

    // Find enemy towers
    let enemy_player = if entity.owner == PlayerId::Player1 {
        PlayerId::Player2
    } else {
        PlayerId::Player1
    };

    let mut princess_left: Option<&crate::entities::Entity> = None;
    let mut princess_right: Option<&crate::entities::Entity> = None;
    let mut king_tower: Option<&crate::entities::Entity> = None;

    // Scan for enemy towers
    // Our coordinates: Arena is 18 wide x 32 high
    // Princess towers at enemy side, king tower in back
    const ARENA_CENTER_X: f32 = 9.0;

    for other_entity in state.entities.values() {
        if other_entity.owner != enemy_player {
            continue;
        }

        if let EntityKind::Tower(_) = other_entity.kind {
            // Classify tower by position
            // Princess towers are closer to middle Y, king tower is at far Y
            // Legacy: Princess at Y=3.5/14.5, King at Y=9
            // In our coords (swap): Princess at X=3.5/14.5, King at X=9

            let x_dist_from_center = (other_entity.position.x - ARENA_CENTER_X).abs();

            if x_dist_from_center < 3.0 {
                // Near center X - this is the king tower
                king_tower = Some(other_entity);
            } else if other_entity.position.x < ARENA_CENTER_X {
                // Left side (low X) - left princess tower
                princess_left = Some(other_entity);
            } else {
                // Right side (high X) - right princess tower
                princess_right = Some(other_entity);
            }
        }
    }

    // Select target based on unit's X position (legacy Y position)
    // Legacy logic: if data[5] <= 9 target left princess, else target right princess
    let target_tower = if entity.position.x < ARENA_CENTER_X {
        // Unit on left side - target left princess tower
        princess_left.or(king_tower)
    } else {
        // Unit on right side - target right princess tower
        princess_right.or(king_tower)
    };

    target_tower.map(|tower| tower.position)
}

/// Gets the movement waypoint for a ground unit that may need to cross a bridge.
/// Based on legacy engine pathfinding (lines 6369-6382, 6489-6503).
///
/// Our coordinate system (portrait, 18 wide x 32 high):
/// - River at Y=15-17 (long axis)
/// - Bridges at X=4-6 and X=11-13 (short axis)
///
/// Legacy used X for long axis, Y for short axis (landscape, 32x18).
/// We swap: Legacy X → Our Y, Legacy Y → Our X
fn get_movement_waypoint(from: &Position, to: &Position, _arena: &crate::arena::Arena) -> Position {
    // River boundaries (our Y axis = legacy X axis)
    const RIVER_Y_START: f32 = 15.0;
    const RIVER_Y_END: f32 = 17.0;

    // Bridge waypoint coordinates (our X axis = legacy Y axis)
    // Bottom bridge: X = 3.0 (rendered at tile x=3)
    // Top bridge: X = 14.0 (rendered at tile x=14)
    const BOTTOM_BRIDGE_X: f32 = 3.0;
    const TOP_BRIDGE_X: f32 = 14.0;

    // Bridge entrance waypoints on X axis (legacy Y waypoints)
    const BOTTOM_BRIDGE_SOUTH: f32 = 2.9;  // Below bottom bridge
    const BOTTOM_BRIDGE_NORTH: f32 = 4.1;  // Above bottom bridge
    const TOP_BRIDGE_SOUTH: f32 = 13.9;    // Below top bridge
    const TOP_BRIDGE_NORTH: f32 = 15.1;    // Above top bridge

    const ARENA_CENTER_X: f32 = 9.0;  // Divides bottom/top bridge selection

    // Check if we're on opposite sides of the river
    let from_above_river = from.y < RIVER_Y_START;
    let to_above_river = to.y < RIVER_Y_START;

    let from_below_river = from.y >= RIVER_Y_END;
    let to_below_river = to.y >= RIVER_Y_END;

    // If both on same side, go direct
    if (from_above_river && to_above_river) || (from_below_river && to_below_river) {
        return *to;
    }

    // Check if we're on a bridge (close to bridge X coordinate)
    // Use wider tolerance for bridge detection
    let on_bottom_bridge = (from.x - BOTTOM_BRIDGE_X).abs() < 1.5;
    let on_top_bridge = (from.x - TOP_BRIDGE_X).abs() < 1.5;
    let on_bridge = on_bottom_bridge || on_top_bridge;

    // If unit is IN the river zone (Y between 15-17) AND on a bridge tile, continue straight
    let from_in_river = from.y >= RIVER_Y_START && from.y < RIVER_Y_END;

    if from_in_river && on_bridge {
        // Actually on a bridge, continue straight across
        return *to;
    }

    // If unit is approaching a bridge entrance (near Y=15 or Y=17) AND heading toward bridge X, continue straight
    let near_bridge_entrance = (from.y - RIVER_Y_START).abs() < 2.0 || (from.y - RIVER_Y_END).abs() < 2.0;

    if near_bridge_entrance && on_bridge {
        // Near bridge entrance and aligned with bridge - go straight to target
        return *to;
    }

    // If in river but NOT on bridge, they're stuck - route to nearest bridge
    if from_in_river {
        let dist_to_bottom = (from.x - BOTTOM_BRIDGE_X).abs();
        let dist_to_top = (from.x - TOP_BRIDGE_X).abs();
        let bridge_x = if dist_to_bottom < dist_to_top { BOTTOM_BRIDGE_X } else { TOP_BRIDGE_X };

        // Move to the nearest bridge first, then continue across
        return Position::new(bridge_x, if to.y < RIVER_Y_START { RIVER_Y_START } else { RIVER_Y_END });
    }

    // Need to cross river - route to bridge entrance on YOUR side of the river
    // Legacy: if unit at X > 17, route to X=17 (far edge before river)
    // Legacy: if unit at X < 15, route to X=15 (near edge before river)
    let waypoint_y = if from_below_river {
        // Coming from below (Y > 17), route to Y=17 (entrance to river from south)
        17.0
    } else {
        // Coming from above (Y < 15), route to Y=15 (entrance to river from north)
        15.0
    };

    // Select bridge based on which one is closer to the unit's X position
    // This keeps units in their lane when crossing
    let dist_to_bottom_bridge = (from.x - BOTTOM_BRIDGE_X).abs();
    let dist_to_top_bridge = (from.x - TOP_BRIDGE_X).abs();

    let waypoint_x = if dist_to_bottom_bridge < dist_to_top_bridge {
        // Closer to bottom bridge (x=3.5)
        BOTTOM_BRIDGE_X
    } else {
        // Closer to top bridge (x=14.0)
        TOP_BRIDGE_X
    };

    Position::new(waypoint_x, waypoint_y)
}

/// Checks if the tile at the new position is blocked for this entity.
/// Returns true if the entity cannot move to this tile.
fn is_tile_blocked(state: &GameState, _entity: &crate::entities::Entity, new_position: &Position) -> bool {
    // Only check if out of bounds
    // The legacy engine doesn't do tile-based river blocking - it relies on waypoint pathfinding
    // to guide units to bridges. Units that somehow get off-path can walk through rivers.
    if !state.arena.is_in_bounds(new_position) {
        return true;
    }

    false
}

/// Checks if moving an entity to a new position would cause a collision.
fn check_collision(state: &GameState, moving_entity_id: EntityId, new_position: &Position) -> bool {
    let moving_entity = &state.entities[&moving_entity_id];
    let moving_radius = moving_entity.radius();

    // Check against all other entities
    for (other_id, other_entity) in &state.entities {
        // Skip self
        if *other_id == moving_entity_id {
            continue;
        }

        // Skip entities with no collision radius
        let other_radius = other_entity.radius();
        if other_radius == 0.0 {
            continue;
        }

        // Calculate distance between centers
        let distance = new_position.distance_to(&other_entity.position);
        let min_distance = moving_radius + other_radius;

        // Collision if circles overlap
        if distance < min_distance {
            return true;
        }
    }

    false
}
