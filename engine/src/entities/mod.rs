//! Entity definitions (troops, towers, projectiles, spells).

use serde::{Deserialize, Serialize};
use shared::{PlayerId, Position, Velocity};

/// An entity in the game world.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub owner: PlayerId,
    pub position: Position,
    pub velocity: Velocity,
    pub hp: f32,
    pub max_hp: f32,
    pub kind: EntityKind,

    /// Time until next attack (in seconds). 0 = ready to attack.
    pub attack_cooldown: f32,

    /// Current target entity ID (if any).
    pub target: Option<u32>,

    /// Card name that spawned this entity (for display purposes).
    pub card_name: Option<String>,

    /// Level of the card that spawned this entity.
    pub level: Option<u32>,
}

impl Entity {
    pub fn new(owner: PlayerId, position: Position, kind: EntityKind) -> Self {
        let max_hp = kind.base_hp();
        Self {
            owner,
            position,
            velocity: Velocity::zero(),
            hp: max_hp,
            max_hp,
            kind,
            attack_cooldown: 0.0,
            target: None,
            card_name: None,
            level: None,
        }
    }

    pub fn new_with_card_info(owner: PlayerId, position: Position, kind: EntityKind, card_name: String, level: u32) -> Self {
        let max_hp = kind.base_hp();
        Self {
            owner,
            position,
            velocity: Velocity::zero(),
            hp: max_hp,
            max_hp,
            kind,
            attack_cooldown: 0.0,
            target: None,
            card_name: Some(card_name),
            level: Some(level),
        }
    }

    pub fn is_alive(&self) -> bool {
        self.hp > 0.0
    }

    pub fn take_damage(&mut self, amount: f32) {
        self.hp = (self.hp - amount).max(0.0);
    }

    /// Returns the attack range for this entity.
    pub fn attack_range(&self) -> f32 {
        match &self.kind {
            EntityKind::Tower(data) => data.range,
            EntityKind::Troop(data) => data.range,
            _ => 0.0,
        }
    }

    /// Returns the damage this entity deals.
    pub fn damage(&self) -> f32 {
        match &self.kind {
            EntityKind::Tower(data) => data.damage,
            EntityKind::Troop(data) => data.damage,
            EntityKind::Projectile(data) => data.damage,
            EntityKind::Spell(data) => data.damage,
        }
    }

    /// Returns the attack speed (seconds between attacks).
    pub fn attack_speed(&self) -> f32 {
        match &self.kind {
            EntityKind::Tower(data) => data.attack_speed,
            EntityKind::Troop(data) => data.attack_speed,
            _ => 1.0,
        }
    }

    /// Returns true if this entity can attack (troops and towers).
    pub fn can_attack(&self) -> bool {
        matches!(self.kind, EntityKind::Tower(_) | EntityKind::Troop(_))
    }

    /// Returns the target type for this entity.
    pub fn target_type(&self) -> Option<TargetType> {
        match &self.kind {
            EntityKind::Troop(data) => Some(data.target_type),
            _ => None,
        }
    }

    /// Returns the movement speed (tiles per second).
    pub fn movement_speed(&self) -> f32 {
        match &self.kind {
            EntityKind::Troop(data) => data.movement_speed,
            _ => 0.0, // Towers and projectiles don't move
        }
    }

    /// Returns true if this entity can move.
    pub fn can_move(&self) -> bool {
        matches!(self.kind, EntityKind::Troop(_))
    }

    /// Returns the collision radius for this entity (in tiles).
    pub fn radius(&self) -> f32 {
        match &self.kind {
            EntityKind::Tower(_) => 1.5,      // Towers are large
            EntityKind::Troop(_) => 0.4,       // Troops are medium (about 1 tile wide for 2 side-by-side)
            EntityKind::Projectile(_) => 0.1,  // Projectiles are small
            EntityKind::Spell(_) => 0.0,       // Spells have no collision
        }
    }

    /// Returns the collision shape for this entity.
    /// Towers use rectangular hitboxes, everything else uses circular.
    pub fn collision_shape(&self) -> CollisionShape {
        match &self.kind {
            EntityKind::Tower(_) => CollisionShape::Rectangle { half_width: 2.0, half_height: 2.0 },
            EntityKind::Troop(_) => CollisionShape::Circle { radius: 0.4 },
            EntityKind::Projectile(_) => CollisionShape::Circle { radius: 0.1 },
            EntityKind::Spell(_) => CollisionShape::None,
        }
    }

    /// Returns true if this entity uses ranged attacks (spawns projectiles).
    pub fn is_ranged(&self) -> bool {
        match &self.kind {
            EntityKind::Troop(data) => data.is_ranged,
            EntityKind::Tower(_) => true, // Towers always shoot projectiles
            _ => false,
        }
    }

    /// Returns true if this entity can cross river tiles (air units, jumping units).
    pub fn can_cross_river(&self) -> bool {
        match &self.kind {
            EntityKind::Troop(data) => data.can_cross_river,
            _ => true, // Projectiles, spells, and towers don't interact with rivers
        }
    }
}

/// Collision shape for entities.
#[derive(Debug, Clone, Copy)]
pub enum CollisionShape {
    Circle { radius: f32 },
    Rectangle { half_width: f32, half_height: f32 },
    None,
}

/// Different types of entities in the game.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EntityKind {
    /// Player towers (King, Princess).
    Tower(TowerData),

    /// Ground or air troops.
    Troop(TroopData),

    /// Projectiles (arrows, fireballs, etc.).
    Projectile(ProjectileData),

    /// Spell effects (area damage, etc.).
    Spell(SpellData),
}

impl EntityKind {
    fn base_hp(&self) -> f32 {
        match self {
            EntityKind::Tower(data) => data.base_hp,
            EntityKind::Troop(data) => data.base_hp,
            EntityKind::Projectile(_) => 1.0,
            EntityKind::Spell(_) => 1.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TowerData {
    pub base_hp: f32,
    pub damage: f32,
    pub range: f32,
    pub attack_speed: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TroopData {
    pub base_hp: f32,
    pub damage: f32,
    pub range: f32,
    pub attack_speed: f32,
    pub movement_speed: f32,
    pub target_type: TargetType,
    pub is_ranged: bool, // true = spawns projectiles, false = instant melee damage
    pub can_cross_river: bool, // true = can cross river (air units, Hog Rider, etc.)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectileData {
    pub damage: f32,
    pub speed: f32,
    pub target_id: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpellData {
    pub damage: f32,
    pub radius: f32,
    pub duration: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum TargetType {
    Ground,
    Air,
    Both,
    Buildings,
}
