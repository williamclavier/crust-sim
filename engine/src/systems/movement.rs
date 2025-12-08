//! Movement system for entities.

use crate::state::{EntityId, GameState};
use shared::Position;

/// Updates entity movement using steering behavior system.
/// Based on Clash Royale's force-weighted steering with turn smoothing.
///
/// Steering forces:
/// - Intent vector (toward target): weight 1.0
/// - Separation force (avoid unit overlaps): weight 0.4
/// - Wall avoid force: weight 0.5
/// - Tower avoid force: weight 0.7 (scaled high for signature "slide" effect)
///
/// Turn smoothing: lerp(previous_direction, total_direction, 0.18)
pub fn update(state: &mut GameState, dt: f32) {
    // Collect all moving entities and their steering data
    let entity_ids: Vec<EntityId> = state.entities.keys().copied().collect();
    let mut movement_updates: Vec<(EntityId, (f32, f32), Position)> = Vec::new();

    for id in &entity_ids {
        let entity = &state.entities[id];

        // Only move troops (not towers)
        if !entity.can_move() {
            continue;
        }

        // Calculate intent vector (direction toward goal)
        let intent_vector = calculate_intent_vector(state, entity);

        // If no intent (no target, nowhere to go), stop
        let (intent_x, intent_y) = match intent_vector {
            Some((x, y)) => (x, y),
            None => {
                // No movement - update smoothed_direction to zero
                movement_updates.push((*id, (0.0, 0.0), entity.position));
                continue;
            }
        };

        // Calculate steering forces
        let separation_force = calculate_separation_force(state, entity, &entity_ids);
        let tower_avoid_force = calculate_tower_avoidance_force(state, entity);
        let wall_avoid_force = calculate_wall_avoidance_force(state, entity);

        // Combine forces with weights
        let total_x = intent_x * 1.0
            + separation_force.0 * 0.4
            + wall_avoid_force.0 * 0.5
            + tower_avoid_force.0 * 0.7;
        let total_y = intent_y * 1.0
            + separation_force.1 * 0.4
            + wall_avoid_force.1 * 0.5
            + tower_avoid_force.1 * 0.7;

        // Normalize total direction
        let total_magnitude = (total_x * total_x + total_y * total_y).sqrt();
        let (desired_dir_x, desired_dir_y) = if total_magnitude > 0.001 {
            (total_x / total_magnitude, total_y / total_magnitude)
        } else {
            (intent_x, intent_y) // Fallback to intent if forces cancel out
        };

        // Apply turn smoothing: lerp(previous_direction, desired_direction, 0.18)
        let prev_dir = entity.smoothed_direction;
        let smoothing_factor = 0.18;

        let smoothed_x = prev_dir.0 + (desired_dir_x - prev_dir.0) * smoothing_factor;
        let smoothed_y = prev_dir.1 + (desired_dir_y - prev_dir.1) * smoothing_factor;

        // Re-normalize smoothed direction
        let smoothed_magnitude = (smoothed_x * smoothed_x + smoothed_y * smoothed_y).sqrt();
        let (final_dir_x, final_dir_y) = if smoothed_magnitude > 0.001 {
            (smoothed_x / smoothed_magnitude, smoothed_y / smoothed_magnitude)
        } else {
            (desired_dir_x, desired_dir_y)
        };

        // Calculate new position: new_position = old_position + (direction * speed * dt)
        let move_speed = entity.movement_speed();
        let new_x = entity.position.x + final_dir_x * move_speed * dt;
        let new_y = entity.position.y + final_dir_y * move_speed * dt;
        let new_position = Position::new(new_x, new_y);

        // Check tile passability (out of bounds)
        let tile_blocked = is_tile_blocked(state, entity, &new_position);

        if !tile_blocked {
            movement_updates.push((*id, (final_dir_x, final_dir_y), new_position));
        } else {
            // Blocked - keep current position but update smoothed direction
            movement_updates.push((*id, (final_dir_x, final_dir_y), entity.position));
        }
    }

    // Apply all movement updates
    for (id, smoothed_direction, new_position) in movement_updates {
        if let Some(entity) = state.entities.get_mut(&id) {
            entity.smoothed_direction = smoothed_direction;
            entity.position = new_position;
        }
    }

    // Apply simple overlap separation (no mass-based physics)
    apply_overlap_separation(state);
}

/// Calculates intent vector (direction toward target or autonomous goal).
/// Returns normalized (unit vector) direction, or None if no goal.
fn calculate_intent_vector(state: &GameState, entity: &crate::entities::Entity) -> Option<(f32, f32)> {
    // Check if entity has a specific target
    if let Some(target_id) = entity.target {
        let target_entity_id = EntityId::from_u32(target_id);

        // Get target position (if target still exists)
        if let Some(target) = state.entities.get(&target_entity_id) {
            let distance = entity.position.distance_to(&target.position);
            let attack_range = entity.attack_range();

            // If target is out of range, move toward it (possibly via bridge waypoint)
            if distance > attack_range {
                // Check if we need to path through a bridge
                let move_target = if entity.can_cross_river() {
                    // Air units / jumping units: go direct
                    target.position
                } else {
                    // Ground units: check if target is across river
                    get_movement_waypoint(&entity.position, &target.position, &state.arena)
                };

                let (dir_x, dir_y) = entity.position.direction_to(&move_target);
                return Some((dir_x, dir_y));
            } else {
                // Target in range - no movement intent
                return None;
            }
        }
    }

    // No specific target - move autonomously toward enemy towers
    let move_target = get_autonomous_movement_target(state, entity);

    if let Some(target_pos) = move_target {
        // Apply waypoint pathfinding for ground units
        let waypoint = if entity.can_cross_river() {
            target_pos
        } else {
            get_movement_waypoint(&entity.position, &target_pos, &state.arena)
        };

        let (dir_x, dir_y) = entity.position.direction_to(&waypoint);
        Some((dir_x, dir_y))
    } else {
        // No enemies at all - no movement
        None
    }
}

/// Calculates separation force to avoid overlapping with nearby units.
/// Returns a normalized force vector pushing away from nearby entities.
fn calculate_separation_force(
    state: &GameState,
    entity: &crate::entities::Entity,
    all_ids: &[EntityId],
) -> (f32, f32) {
    let mut total_force_x = 0.0;
    let mut total_force_y = 0.0;
    let mut neighbor_count = 0;

    const SEPARATION_RADIUS: f32 = 2.0; // Only consider nearby units within 2 tiles

    for other_id in all_ids {
        let other = &state.entities[other_id];

        // Don't separate from self (check by position) or from towers
        let is_self = (entity.position.x - other.position.x).abs() < 0.001
            && (entity.position.y - other.position.y).abs() < 0.001;

        if is_self || !other.can_move() {
            continue;
        }

        let dx = entity.position.x - other.position.x;
        let dy = entity.position.y - other.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Only apply separation to nearby units
        if distance < SEPARATION_RADIUS && distance > 0.001 {
            // Force strength inversely proportional to distance
            let force_magnitude = 1.0 / distance;
            total_force_x += (dx / distance) * force_magnitude;
            total_force_y += (dy / distance) * force_magnitude;
            neighbor_count += 1;
        }
    }

    // Average and normalize the separation force
    if neighbor_count > 0 {
        total_force_x /= neighbor_count as f32;
        total_force_y /= neighbor_count as f32;

        let magnitude = (total_force_x * total_force_x + total_force_y * total_force_y).sqrt();
        if magnitude > 0.001 {
            return (total_force_x / magnitude, total_force_y / magnitude);
        }
    }

    (0.0, 0.0)
}

/// Calculates tower avoidance force to make units "slide" around towers.
/// Simple repulsion force - just pushes away from tower center.
/// Combined with intent vector, this creates natural curved paths around obstacles.
/// Weighted HIGH (0.7) to create Clash Royale's signature curved path.
fn calculate_tower_avoidance_force(
    state: &GameState,
    entity: &crate::entities::Entity,
) -> (f32, f32) {
    use crate::entities::EntityKind;

    let mut total_force_x = 0.0;
    let mut total_force_y = 0.0;
    let mut tower_count = 0;

    // Tight avoidance radius - only repel when actually close to tower
    const AVOIDANCE_RADIUS: f32 = 2.2;

    for other in state.entities.values() {
        // Only avoid towers
        if !matches!(other.kind, EntityKind::Tower(_)) {
            continue;
        }

        let dx = entity.position.x - other.position.x;
        let dy = entity.position.y - other.position.y;
        let distance = (dx * dx + dy * dy).sqrt();

        // Only apply avoidance when close to tower
        if distance < AVOIDANCE_RADIUS && distance > 0.001 {
            // Simple repulsion - push directly away from tower center
            // Strength increases as distance decreases (inverse relationship)
            let strength = (AVOIDANCE_RADIUS - distance) / AVOIDANCE_RADIUS;

            total_force_x += (dx / distance) * strength;
            total_force_y += (dy / distance) * strength;
            tower_count += 1;
        }
    }

    // Average and normalize the tower avoidance force
    if tower_count > 0 {
        total_force_x /= tower_count as f32;
        total_force_y /= tower_count as f32;

        let magnitude = (total_force_x * total_force_x + total_force_y * total_force_y).sqrt();
        if magnitude > 0.001 {
            return (total_force_x / magnitude, total_force_y / magnitude);
        }
    }

    (0.0, 0.0)
}

/// Calculates wall avoidance force to prevent units from walking into arena boundaries.
/// Returns a normalized force vector pushing away from nearby walls.
fn calculate_wall_avoidance_force(
    state: &GameState,
    entity: &crate::entities::Entity,
) -> (f32, f32) {
    let mut force_x = 0.0;
    let mut force_y = 0.0;

    const WALL_AVOID_DISTANCE: f32 = 1.5; // Start avoiding walls from 1.5 tiles away

    // Check distance to each boundary
    let left_dist = entity.position.x;
    let right_dist = state.arena.width as f32 - entity.position.x;
    let top_dist = entity.position.y;
    let bottom_dist = state.arena.height as f32 - entity.position.y;

    // Left wall
    if left_dist < WALL_AVOID_DISTANCE {
        force_x += (WALL_AVOID_DISTANCE - left_dist) / WALL_AVOID_DISTANCE;
    }

    // Right wall
    if right_dist < WALL_AVOID_DISTANCE {
        force_x -= (WALL_AVOID_DISTANCE - right_dist) / WALL_AVOID_DISTANCE;
    }

    // Top wall
    if top_dist < WALL_AVOID_DISTANCE {
        force_y += (WALL_AVOID_DISTANCE - top_dist) / WALL_AVOID_DISTANCE;
    }

    // Bottom wall
    if bottom_dist < WALL_AVOID_DISTANCE {
        force_y -= (WALL_AVOID_DISTANCE - bottom_dist) / WALL_AVOID_DISTANCE;
    }

    // Normalize
    let magnitude = (force_x * force_x + force_y * force_y).sqrt();
    if magnitude > 0.001 {
        (force_x / magnitude, force_y / magnitude)
    } else {
        (0.0, 0.0)
    }
}

/// Applies simple overlap separation to prevent units from stacking.
/// No mass-based physics - just equal push-apart along separation vector.
fn apply_overlap_separation(state: &mut GameState) {
    let entity_ids: Vec<EntityId> = state.entities.keys().copied().collect();
    let mut separation_updates: Vec<(EntityId, Position)> = Vec::new();

    // Check all pairs for overlap
    for i in 0..entity_ids.len() {
        for j in (i + 1)..entity_ids.len() {
            let id1 = entity_ids[i];
            let id2 = entity_ids[j];

            let (pos1, radius1, can_move1, pos2, radius2, can_move2) = {
                let e1 = &state.entities[&id1];
                let e2 = &state.entities[&id2];
                (
                    e1.position,
                    e1.radius(),
                    e1.can_move(),
                    e2.position,
                    e2.radius(),
                    e2.can_move(),
                )
            };

            // Skip if either has no collision radius
            if radius1 == 0.0 || radius2 == 0.0 {
                continue;
            }

            // Calculate overlap
            let dx = pos2.x - pos1.x;
            let dy = pos2.y - pos1.y;
            let distance = (dx * dx + dy * dy).sqrt();
            let min_distance = radius1 + radius2;
            let overlap = min_distance - distance;

            // Only apply hard separation if ACTUALLY overlapping (not just close)
            // The steering separation force already handles "staying apart"
            if overlap > 0.2 && distance > 0.001 {
                // Gentle push-apart - much smaller than before
                // This is just to fix actual penetration, not to keep them apart
                let push_amount = overlap * 0.3; // Only push 30% of the overlap per frame (gradual)

                // Normalize direction
                let dir_x = dx / distance;
                let dir_y = dy / distance;

                // Push entity 1 away
                if can_move1 {
                    let new_pos1 = Position::new(
                        pos1.x - dir_x * push_amount,
                        pos1.y - dir_y * push_amount,
                    );
                    separation_updates.push((id1, new_pos1));
                }

                // Push entity 2 away
                if can_move2 {
                    let new_pos2 = Position::new(
                        pos2.x + dir_x * push_amount,
                        pos2.y + dir_y * push_amount,
                    );
                    separation_updates.push((id2, new_pos2));
                }
            }
        }
    }

    // Apply separation updates
    for (id, new_position) in separation_updates {
        if let Some(entity) = state.entities.get_mut(&id) {
            entity.position = new_position;
        }
    }
}

/// Finds the initial target tower for a newly spawned unit based on its position.
/// Returns the entity ID of the target tower.
/// This locks in targeting at spawn to prevent units from switching lanes mid-path.
pub fn find_initial_target(state: &GameState, entity_id: EntityId) -> Option<u32> {
    let entity = state.entities.get(&entity_id)?;

    use crate::entities::EntityKind;
    use shared::PlayerId;

    // Find enemy towers
    let enemy_player = if entity.owner == PlayerId::Player1 {
        PlayerId::Player2
    } else {
        PlayerId::Player1
    };

    let mut princess_left: Option<(EntityId, &crate::entities::Entity)> = None;
    let mut princess_right: Option<(EntityId, &crate::entities::Entity)> = None;
    let mut king_tower: Option<(EntityId, &crate::entities::Entity)> = None;

    const ARENA_CENTER_X: f32 = 9.0;

    for (id, other_entity) in &state.entities {
        if other_entity.owner != enemy_player {
            continue;
        }

        if let EntityKind::Tower(_) = other_entity.kind {
            let x_dist_from_center = (other_entity.position.x - ARENA_CENTER_X).abs();

            if x_dist_from_center < 3.0 {
                king_tower = Some((*id, other_entity));
            } else if other_entity.position.x < ARENA_CENTER_X {
                princess_left = Some((*id, other_entity));
            } else {
                princess_right = Some((*id, other_entity));
            }
        }
    }

    // Select target based on unit's SPAWN position (not current position)
    // This ensures targeting is locked in and doesn't change as unit moves
    let target_tower = if entity.position.x < ARENA_CENTER_X {
        princess_left.or(king_tower)
    } else {
        princess_right.or(king_tower)
    };

    target_tower.map(|(id, _)| id.as_u32())
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

    // If unit is aligned with a bridge (X position matches), they should cross straight through
    // Don't wait until they reach the entrance - start crossing as soon as they're aligned
    if on_bridge {
        // On bridge lane - go straight to target, crossing the river
        return *to;
    }

    // If unit is IN the river but NOT on a bridge, they're stuck - route to nearest bridge
    let from_in_river = from.y >= RIVER_Y_START && from.y < RIVER_Y_END;

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

    // Select bridge based on which one is closer to the TARGET's X position
    // This ensures units route through the correct bridge to reach their destination
    let dist_to_bottom_bridge = (to.x - BOTTOM_BRIDGE_X).abs();
    let dist_to_top_bridge = (to.x - TOP_BRIDGE_X).abs();

    let waypoint_x = if dist_to_bottom_bridge < dist_to_top_bridge {
        // Target closer to bottom bridge (X=3.0) - use left lane
        BOTTOM_BRIDGE_X
    } else {
        // Target closer to top bridge (X=14.0) - use right lane
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
