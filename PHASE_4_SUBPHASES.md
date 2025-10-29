# Phase 4 Sub-Phases: Combat Systems

Phase 4 was originally scoped as "Collision & Targeting" but includes several complex systems. Breaking it into sub-phases for incremental implementation.

---

## ✅ Phase 4.1: Basic Targeting & Combat (COMPLETE)

**Status:** Complete (commit 22eebd4)

**Implemented:**
- Entity attack cooldowns and target tracking
- Targeting logic (find nearest enemy)
- Damage application and attack execution
- Entity death and cleanup
- Target type filtering (ground/air/buildings)

**Testing:**
- Knight vs Archers combat verified
- Entities die and are removed correctly
- Attack cooldowns work properly

---

## Phase 4.2: Basic Movement AI

**Goal:** Make units move toward targets when out of range

**Tasks:**
1. Add velocity calculation toward target
2. Implement "move toward target if out of range" logic
3. Add movement speed from card data to entities
4. Stop moving when in attack range
5. Test Knight walking toward distant Archers

**Implementation Plan:**
- Add `set_velocity_toward()` helper to shared Position
- Update movement system to calculate direction to target
- Add `is_in_attack_range()` check to stop movement
- Units should:
  - Stand still if target in range (attack)
  - Move toward target if out of range
  - Stop when reaching attack range

**Testing Scenario:**
- Spawn Knight at (10, 10)
- Spawn Archers at (20, 10) - 10 tiles away
- Knight should walk toward Archers
- Knight should stop at distance 1.2 (attack range)
- Combat should begin once in range

**Deliverables:**
- Updated movement system with target-seeking
- CLI demo showing unit movement
- Deterministic movement (same seed = same paths)

**Success Criteria:**
- Knight walks toward distant target
- Knight stops when in attack range
- Movement speed matches card data
- No jittering or overshooting

---

## ✅ Phase 4.3: Circular Collision Detection (COMPLETE)

**Status:** Complete (commit 3792d16)

**Goal:** Prevent units from overlapping and walking through each other

**Implemented:**
- Added `radius()` method to Entity (Towers: 1.5, Troops: 0.4, Projectiles: 0.1, Spells: 0.0)
- Implemented circle-circle collision detection in movement system
- Three-pass movement: calculate velocities → check collisions → apply positions
- Simple collision response: don't move if collision would occur
- CLI test with 2 Knights side-by-side targeting same Archer

**Testing Results:**
- Knights started 0.9 tiles apart
- Converged to exactly 0.80 tiles (minimum safe distance)
- Maintained spacing throughout simulation
- No overlapping occurred

**Known Limitations & Deferred Features:**

1. **No pathfinding around obstacles**
   - Units stop if completely blocked by collision
   - Don't find alternate routes around obstacles
   - Works fine for simple scenarios (units side-by-side)
   - Advanced pathfinding deferred to future phase (outside Phase 4 scope)

2. **No pushing/sliding mechanics**
   - Units don't slide along collision surfaces
   - If diagonal movement blocked, unit stops completely
   - Could add tangent-space sliding in future (low priority)

3. **No separation force**
   - Units that somehow spawn overlapping won't push apart
   - Only prevents new overlaps, doesn't fix existing ones
   - Not an issue if spawn positions are validated

4. **Simple "stop moving" collision response**
   - Unit completely halts if collision detected
   - No partial movement or gradual navigation
   - Works well enough for current needs

5. **Performance: O(n²) collision checks**
   - Every moving entity checks against all others
   - Fine for <100 entities (typical match)
   - Will need spatial partitioning (quadtree/grid) for 100+ units
   - Optimization deferred to Phase 4.6 if needed

6. **No collision layers/groups**
   - All entities collide with all others (except radius 0.0)
   - Can't have "pass-through" relationships (e.g., air units over ground)
   - May need in future for air/ground separation

**Success Criteria Met:**
- ✅ Units don't overlap (circle collision works)
- ✅ Collision detection is deterministic
- ⚠️  Units can't navigate around obstacles (no pathfinding - deferred)
- ✅ Performance acceptable for typical match sizes

---

## ✅ Phase 4.4: Projectile System (COMPLETE)

**Status:** Complete

**Goal:** Ranged attacks spawn projectiles that travel to targets

**Implemented:**
- Added `is_ranged: bool` field to TroopData
- Added `is_ranged()` method to Entity (Troops check field, Towers always true)
- Modified combat system to spawn projectiles for ranged attacks instead of instant damage
- Created projectile system in `systems/projectile.rs` with:
  - Projectile movement toward target at 15 tiles/second
  - Collision detection when projectile reaches target (distance <= combined radii)
  - Damage application on hit
  - Projectile removal after hit or if target dies
- Updated card spawning to auto-detect ranged units (range > 2.0)

**Testing Results:**
- Archers spawn 2 projectiles when attacking (one per archer)
- Projectiles travel toward Knight target
- Knight took 800 damage over 5 seconds (4 successful arrow hits)
- Projectiles removed after hitting target
- System integrated into main game loop (runs after movement, before lifecycle)

**Success Criteria Met:**
- ✅ Projectiles spawn on ranged attack
- ✅ Projectiles travel to target
- ✅ Damage applied on impact
- ✅ Projectiles removed after hit
- ✅ Melee attacks still instant (no projectiles)
- ✅ Circle-to-rectangle collision for towers

**Future Enhancements (Deferred):**

Based on analysis of the original clash-royale-engine implementation, the following features are not yet implemented but could be added in future phases:

1. **Pass-Through Mechanics** - Projectiles that hit multiple targets (Magic Archer)
   - Track already-hit targets per projectile
   - Distance-based pass-through limits
   - Apply damage to all units in path

2. **Special Effects System** - Status effects on projectile hit
   - Stun effects (duration-based immobilization)
   - Slow effects (movement speed reduction)
   - Knockback effects (push entities with direction/distance)
   - Snare effects (prevent movement but allow attacks)
   - Rage/healing buffs

3. **Spread/Shotgun Patterns** - Multiple projectiles with angle variance
   - Firecracker-style spreading shrapnel
   - Hunter-style shotgun spread
   - Configurable spread angle and count

4. **Returning Projectiles** - Boomerang-style mechanics
   - Executioner's axe returns to thrower
   - Damage on both outgoing and return path
   - Track return state and distance

5. **Tower Damage Modifiers** - Different damage for crown towers
   - Crown towers take 30% spell damage
   - Regular towers take full damage
   - Non-spell projectiles unaffected

6. **Projectile Retargeting** - Auto-retarget if original target dies
   - Find new target mid-flight
   - Optional per-card behavior

7. **Wait Timers** - Delayed damage application
   - Travel time before damage activates
   - Used for spell delays

8. **Area Damage on Impact** - Splash damage around hit point
   - Radius-based damage application
   - Damage falloff curves

These features are well-documented in `/Users/will/Documents/Projects/clash-royale-engine/code.txt` lines 2009-8300 and can be referenced when implementing advanced card mechanics in future phases.

---

## ✅ Phase 4.5: Advanced Targeting (Tower Priority) (COMPLETE)

**Status:** Complete (already implemented in Phase 4.1)

**Goal:** Implement proper targeting priorities (buildings > troops)

**Analysis:**
Upon review, this functionality was already correctly implemented in Phase 4.1's targeting system. The implementation works as follows:

**Existing Implementation:**
1. Entity types are identified via `EntityKind::Tower` vs `EntityKind::Troop`
2. Target filtering is implemented via `TargetType` enum:
   - `TargetType::Buildings`: Only targets towers (Giant behavior)
   - `TargetType::Ground`: Only targets ground units
   - `TargetType::Air`: Only targets air units (not yet implemented)
   - `TargetType::Both`: Targets any enemy unit
3. `is_valid_target_type()` function filters entities by type:
   - For `TargetType::Buildings`: Returns `true` only for `EntityKind::Tower(_)`
   - Other types filter accordingly
4. `find_target()` returns nearest valid enemy matching the target type filter

**How It Works:**
- Giant has `targets: ["buildings"]` → `TargetType::Buildings`
- When Giant looks for target, `is_valid_target_type()` rejects all non-tower entities
- Result: Giant walks past troops and only attacks towers
- Troops with `TargetType::Both` or `TargetType::Ground` attack nearest enemy of matching type

**Success Criteria Met:**
- ✅ Giants only target towers (troops filtered out)
- ✅ Troops attack nearest valid enemy based on their target type
- ✅ Targeting is deterministic (always nearest matching target)
- ✅ Units retain target until it dies or becomes invalid (no unnecessary retargeting)

**Note:** This is target **filtering**, not target **prioritization**. Units don't "prefer" buildings over troops - they either can or cannot target specific entity types. This matches Clash Royale's actual behavior where Giants literally cannot target troops at all.

---

## Phase 4.6: Integration Testing

**Goal:** Verify all combat systems work together

**Tasks:**
1. Full battle test: 3v3 units with mixed types
2. Tower destruction test
3. Movement + collision + projectiles + targeting
4. Performance test (100+ entities)
5. Document all known issues

**Testing Scenarios:**
1. **Mixed Combat:**
   - P1: Knight, Archers, Giant
   - P2: Knight, Archers, Giant
   - All spawn near each other
   - Verify correct targeting, movement, projectiles

2. **Tower Siege:**
   - Spawn Giant targeting tower
   - Spawn defending troops
   - Verify Giant reaches and destroys tower

3. **Ranged vs Melee:**
   - Archers vs Knight
   - Knight should walk toward Archers
   - Archers should shoot while staying still (future)
   - For now: just verify projectiles and damage

**Deliverables:**
- Comprehensive combat test suite
- Performance benchmarks
- Known issues documented
- Phase 4 partial summary

**Success Criteria:**
- All sub-phases working together
- No crashes or infinite loops
- Deterministic results
- Ready for Phase 4.7 (Arena Navigation)

---

## ✅ Phase 4.7: Arena Navigation (COMPLETE)

**Status:** Complete

**Goal:** Implement river/bridge mechanics and tile-based movement constraints

**Background:**
Currently units can walk through rivers and ignore terrain. The legacy engine has:
- River tiles (impassable water at x=15-16)
- Bridge tiles (crossings at y=4-6)
- "jumping" special ability (allows some units to cross rivers)

**Tasks:**
1. Add tile type checking to Arena module
2. Implement river blocking in movement system
3. Add bridge passability at specific coordinates
4. Create "can_cross_river" flag for jumping units
5. Test units navigating around/over river
6. Add visual indication of blocked paths (optional)

**Implementation Plan:**
- Update Arena to expose `get_tile_type(x, y)` function
- Tile types: Normal, River, Bridge, Crown, Princess, Banned
- Modify movement system:
  - Before moving, check if destination tile is passable
  - River tiles blocked unless:
    - Tile is a bridge (y=4-6 in river zone)
    - Unit has "jumping" ability
  - Stop movement if blocked (similar to collision)
- Add jumping ability to entity special effects (future)

**Testing Scenario:**
- Spawn Knight at (10, 10) - Blue side
- Spawn Archers at (20, 10) - Red side (across river)
- Knight should path toward bridge (y=4-6) to cross river
- Knight should NOT walk directly through river at y=9 or y=11
- (Advanced) Units with jumping ability ignore river blocking

**Deliverables:**
- Tile-based movement validation
- River/bridge mechanics working
- CLI demo showing units navigating to bridges
- Documentation of tile types in arena.json

**Success Criteria:**
- Units cannot walk through river (except at bridges)
- Units path correctly toward bridge crossings
- Jumping units (if implemented) can cross anywhere
- Movement is still deterministic
- Ready for Phase 5 (Replay & Serialization)

**Implemented:**
- Added `can_cross_river` flag to TroopData (air units, jumping units)
- Arena initialized with river at y=15-16, bridges at x=4-6 and x=11-13
- Tile passability checking in movement system (blocks river for ground units)
- **Bridge waypoint pathfinding** (matching legacy engine approach):
  - Detects when target is across river
  - Calculates nearest bridge (left bridge at x=5, right bridge at x=12)
  - Sets intermediate waypoint at bridge center
  - Units path to bridge first, then to target
- Air units and jumping units ignore river blocking

**Testing Results:**
- Ground units correctly blocked from walking through river
- Bridge pathfinding automatically routes units to nearest bridge
- Air units can cross river directly
- No regression in existing combat/projectile tests

**Implementation Notes:**
- Uses **simple waypoint pathfinding** (not A* or flow fields)
- Matches legacy engine's approach: `if (target across river) { route_to_bridge() }`
- Bridge selection based on horizontal distance (closest bridge)
- Once unit crosses river, moves directly toward target
- Deterministic and performant (no expensive pathfinding calculations)
- **River crossing logic**: Units can ENTER river zone only at bridge X coordinates, but once inside the river zone (from crossing a bridge), they can exit freely to prevent diagonal movement blocking

**Known Limitations (Acceptable for Phase 4):**
- No dynamic obstacle avoidance around buildings/troops (handled by collision system)
- No lane-based steering or formation behavior
- Units pick closest bridge by X-distance, not path distance
- Advanced pathfinding features (A*, flow fields, lane graphs) deferred to future phase

---

## Timeline Estimate

- **Phase 4.1:** ✅ Complete (2-3 hours)
- **Phase 4.2:** ✅ Complete (2-3 hours)
- **Phase 4.3:** ✅ Complete (3-4 hours)
- **Phase 4.4:** ✅ Complete (3-4 hours)
- **Phase 4.5:** ✅ Complete (0 hours - already implemented)
- **Phase 4.6:** 2-3 hours (integration testing and documentation)
- **Phase 4.7:** 2-3 hours (river/bridge navigation constraints)

**Total:** ~13-19 hours for complete Phase 4

---

## Next Step

Currently on: **Phase 4.7 - Arena Navigation**

Note: Implementing 4.7 before 4.6 so that integration testing (4.6) includes river/bridge mechanics.

Ready to start when you are!
