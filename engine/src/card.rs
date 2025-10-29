//! Card definitions and behaviors.

use crate::entities::{Entity, EntityKind, TargetType, TroopData};
use crate::state::GameState;
use serde::{Deserialize, Serialize};
use shared::{PlayerId, Position, Result};

/// A card that can be played by a player.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Card {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub elixir_cost: f32,
    pub rarity: Rarity,
    #[serde(rename = "card_type")]
    pub type_name: String, // "troop", "spell", "building"

    // Card-level properties (constant across levels)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack_speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub first_hit_speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movement_speed: Option<String>, // "slow", "medium", "fast", "very_fast"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub movement_speed_value: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deploy_time: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub range: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projectile_speed: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub targets: Option<Vec<String>>, // ["air", "ground", "buildings"]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<String>, // "ground", "air"
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub radius: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<Vec<String>>, // ["freeze", "knockback", "spawn", etc.]

    // Level-based stats
    pub levels: Vec<CardLevelStats>,
}

/// Stats that vary by card level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardLevelStats {
    pub level: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hp: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub damage: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dps: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub area_damage: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spawn_damage: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shield_hp: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healing: Option<f32>,
}

impl Card {
    /// Spawns entities when this card is played at a specific level.
    pub fn spawn(&self, state: &mut GameState, owner: PlayerId, position: Position, level: u32) -> Result<()> {
        // Get stats for the requested level
        let level_stats = self.get_level_stats(level)?;

        match self.type_name.as_str() {
            "troop" | "tower troop" => {
                self.spawn_troop(state, owner, position, level_stats)?;
            }
            "spell" => {
                self.apply_spell(state, owner, position, level_stats)?;
            }
            "building" => {
                self.spawn_building(state, owner, position, level_stats)?;
            }
            _ => {
                return Err(shared::Error::InvalidAction(format!(
                    "Unknown card type: {}",
                    self.type_name
                )));
            }
        }
        Ok(())
    }

    /// Get stats for a specific card level.
    pub fn get_level_stats(&self, level: u32) -> Result<&CardLevelStats> {
        self.levels
            .iter()
            .find(|stats| stats.level == level)
            .ok_or_else(|| {
                shared::Error::InvalidAction(format!("Level {} not found for {}", level, self.name))
            })
    }

    /// Get the target type from the targets list.
    fn get_target_type(&self) -> TargetType {
        match &self.targets {
            Some(targets) => {
                let has_air = targets.iter().any(|t| t == "air");
                let has_ground = targets.iter().any(|t| t == "ground");
                let has_buildings = targets.iter().any(|t| t == "buildings");

                if has_buildings {
                    TargetType::Buildings
                } else if has_air && has_ground {
                    TargetType::Both
                } else if has_air {
                    TargetType::Air
                } else {
                    TargetType::Ground
                }
            }
            None => TargetType::Both, // Default
        }
    }

    fn spawn_troop(
        &self,
        state: &mut GameState,
        owner: PlayerId,
        position: Position,
        level_stats: &CardLevelStats,
    ) -> Result<()> {
        let count = self.count.unwrap_or(1);
        let hp = level_stats.hp.unwrap_or(100.0);
        let damage = level_stats.damage.unwrap_or(10.0);
        let range = self.range.unwrap_or(1.0);

        // Determine if this is a ranged unit based on attack range
        // Melee units have range <= 2.0, ranged units have range > 2.0
        let is_ranged = range > 2.0;

        // Determine if this unit can cross rivers (air units, jumping units)
        // Check transport type: "air" = can cross, "ground" = cannot cross
        let can_cross_river = self.transport.as_ref().map(|t| t == "air").unwrap_or(false);

        for _ in 0..count {
            let entity = Entity::new_with_card_info(
                owner,
                position,
                EntityKind::Troop(TroopData {
                    base_hp: hp,
                    damage,
                    range,
                    attack_speed: self.attack_speed.unwrap_or(1.0),
                    movement_speed: self.movement_speed_value.unwrap_or(60.0),
                    target_type: self.get_target_type(),
                    is_ranged,
                    can_cross_river,
                }),
                self.name.clone(),
                level_stats.level,
            );
            state.add_entity(entity);
        }
        Ok(())
    }

    fn spawn_building(
        &self,
        state: &mut GameState,
        owner: PlayerId,
        position: Position,
        _level_stats: &CardLevelStats,
    ) -> Result<()> {
        // TODO: Implement building spawning
        let _ = (state, owner, position);
        Ok(())
    }

    fn apply_spell(
        &self,
        state: &mut GameState,
        owner: PlayerId,
        position: Position,
        level_stats: &CardLevelStats,
    ) -> Result<()> {
        // TODO: Implement spell effects using level_stats.area_damage or .damage
        let _ = (state, owner, position, level_stats);
        Ok(())
    }
}

/// Card rarity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Rarity {
    Common,
    Rare,
    Epic,
    Legendary,
}

/// Load cards from JSON file.
pub fn load_cards_from_json(path: &str) -> Result<Vec<Card>> {
    let data = std::fs::read_to_string(path)
        .map_err(|e| shared::Error::InvalidAction(format!("Failed to read cards file: {}", e)))?;

    let cards: Vec<Card> = serde_json::from_str(&data)
        .map_err(|e| shared::Error::InvalidAction(format!("Failed to parse cards JSON: {}", e)))?;

    Ok(cards)
}

/// Get basic test cards for development.
pub fn get_test_cards() -> Vec<Card> {
    vec![
        // Knight - 3 elixir melee tank
        Card {
            name: "Knight".to_string(),
            url: None,
            elixir_cost: 3.0,
            rarity: Rarity::Common,
            type_name: "troop".to_string(),
            attack_speed: Some(1.2),
            first_hit_speed: None,
            movement_speed: Some("medium".to_string()),
            movement_speed_value: Some(1.0),
            deploy_time: Some(1.0),
            range: Some(1.2),
            projectile_speed: None,
            targets: Some(vec!["ground".to_string()]),
            count: Some(1),
            transport: Some("ground".to_string()),
            duration: None,
            radius: None,
            effects: None,
            levels: vec![
                CardLevelStats {
                    level: 11,
                    hp: Some(1452.0),
                    damage: Some(167.0),
                    dps: None,
                    area_damage: None,
                    spawn_damage: None,
                    shield_hp: None,
                    healing: None,
                }
            ],
        },
        // Archers - 3 elixir ranged duo
        Card {
            name: "Archers".to_string(),
            url: None,
            elixir_cost: 3.0,
            rarity: Rarity::Common,
            type_name: "troop".to_string(),
            attack_speed: Some(1.2),
            first_hit_speed: None,
            movement_speed: Some("medium".to_string()),
            movement_speed_value: Some(1.0),
            deploy_time: Some(1.0),
            range: Some(5.0),
            projectile_speed: None,
            targets: Some(vec!["air".to_string(), "ground".to_string()]),
            count: Some(2),
            transport: Some("ground".to_string()),
            duration: None,
            radius: None,
            effects: None,
            levels: vec![
                CardLevelStats {
                    level: 11,
                    hp: Some(252.0),
                    damage: Some(100.0),
                    dps: None,
                    area_damage: None,
                    spawn_damage: None,
                    shield_hp: None,
                    healing: None,
                }
            ],
        },
        // Giant - 5 elixir tank (targets buildings)
        Card {
            name: "Giant".to_string(),
            url: None,
            elixir_cost: 5.0,
            rarity: Rarity::Rare,
            type_name: "troop".to_string(),
            attack_speed: Some(1.5),
            first_hit_speed: None,
            movement_speed: Some("slow".to_string()),
            movement_speed_value: Some(0.75),
            deploy_time: Some(1.0),
            range: Some(1.2),
            projectile_speed: None,
            targets: Some(vec!["buildings".to_string()]),
            count: Some(1),
            transport: Some("ground".to_string()),
            duration: None,
            radius: None,
            effects: None,
            levels: vec![
                CardLevelStats {
                    level: 11,
                    hp: Some(3275.0),
                    damage: Some(211.0),
                    dps: None,
                    area_damage: None,
                    spawn_damage: None,
                    shield_hp: None,
                    healing: None,
                }
            ],
        },
        // Fireball - 4 elixir damage spell
        Card {
            name: "Fireball".to_string(),
            url: None,
            elixir_cost: 4.0,
            rarity: Rarity::Rare,
            type_name: "spell".to_string(),
            attack_speed: None,
            first_hit_speed: None,
            movement_speed: None,
            movement_speed_value: None,
            deploy_time: Some(0.0),
            range: None,
            projectile_speed: None,
            targets: Some(vec!["air".to_string(), "ground".to_string()]),
            count: None,
            transport: None,
            duration: None,
            radius: Some(2.5),
            effects: Some(vec!["damage".to_string()]),
            levels: vec![
                CardLevelStats {
                    level: 11,
                    hp: Some(0.0),
                    damage: Some(572.0),
                    dps: None,
                    area_damage: None,
                    spawn_damage: None,
                    shield_hp: None,
                    healing: None,
                }
            ],
        },
        // Arrows - 3 elixir area damage spell
        Card {
            name: "Arrows".to_string(),
            url: None,
            elixir_cost: 3.0,
            rarity: Rarity::Common,
            type_name: "spell".to_string(),
            attack_speed: None,
            first_hit_speed: None,
            movement_speed: None,
            movement_speed_value: None,
            deploy_time: Some(0.0),
            range: None,
            projectile_speed: None,
            targets: Some(vec!["air".to_string(), "ground".to_string()]),
            count: None,
            transport: None,
            duration: None,
            radius: Some(4.0),
            effects: Some(vec!["damage".to_string()]),
            levels: vec![
                CardLevelStats {
                    level: 11,
                    hp: Some(0.0),
                    damage: Some(144.0),
                    dps: None,
                    area_damage: None,
                    spawn_damage: None,
                    shield_hp: None,
                    healing: None,
                }
            ],
        },
    ]
}

