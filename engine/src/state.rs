//! Game state management and serialization.

use crate::action::Action;
use crate::arena::Arena;
use crate::card::Card;
use crate::entities::Entity;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use shared::{PlayerId, Result};
use std::collections::HashMap;

/// The complete state of a game simulation.
///
/// This struct contains everything needed to:
/// - Run the simulation forward
/// - Serialize/deserialize for replays
/// - Restore to a previous state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    /// Current simulation tick (increments each step).
    pub tick: u64,

    /// Deterministic RNG for all randomness.
    pub rng: Rng,

    /// All entities currently in the game (troops, towers, projectiles).
    pub entities: HashMap<EntityId, Entity>,

    /// Player-specific state (elixir, deck, etc.).
    pub players: HashMap<PlayerId, PlayerState>,

    /// Available cards (loaded at game start, indexed by card name).
    cards: HashMap<String, Card>,

    /// Arena geometry and tile system.
    pub arena: Arena,

    /// Next entity ID to assign.
    next_entity_id: u32,

    /// Game match time in seconds.
    pub match_time: f32,

    /// Maximum match duration (in seconds).
    pub max_match_time: f32,
}

impl GameState {
    /// Creates a new game state with the given RNG seed.
    pub fn new(seed: u64) -> Self {
        let mut players = HashMap::new();
        players.insert(PlayerId::Player1, PlayerState::new(PlayerId::Player1));
        players.insert(PlayerId::Player2, PlayerState::new(PlayerId::Player2));

        // Load test cards
        let mut cards = HashMap::new();
        for card in crate::card::get_test_cards() {
            cards.insert(card.name.clone(), card);
        }

        let mut state = Self {
            tick: 0,
            rng: Rng::new(seed),
            entities: HashMap::new(),
            players,
            cards,
            arena: Arena::new(),
            next_entity_id: 1,
            match_time: 0.0,
            max_match_time: 180.0, // 3 minutes (will be configurable)
        };

        // Spawn towers for both players
        state.spawn_towers();

        state
    }

    /// Spawns tower entities for both players at their starting positions.
    fn spawn_towers(&mut self) {
        use crate::entities::{EntityKind, TowerData};
        use shared::Position;

        // Player 1 towers (bottom side)
        // King tower: center of 4x4 zone (x=7-10, y=27-30) = (8.5, 28.5)
        let p1_king = Entity::new(
            PlayerId::Player1,
            Position::new(8.5, 28.5),
            EntityKind::Tower(TowerData {
                base_hp: 2400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 1.0,
            }),
        );
        self.add_entity(p1_king);

        // Left Princess tower: center of 3x3 zone (x=2-4, y=24-26) = (3.0, 25.0)
        let p1_left = Entity::new(
            PlayerId::Player1,
            Position::new(3.0, 25.0),
            EntityKind::Tower(TowerData {
                base_hp: 1400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 0.8,
            }),
        );
        self.add_entity(p1_left);

        // Right Princess tower: center of 3x3 zone (x=13-15, y=24-26) = (14.0, 25.0)
        let p1_right = Entity::new(
            PlayerId::Player1,
            Position::new(14.0, 25.0),
            EntityKind::Tower(TowerData {
                base_hp: 1400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 0.8,
            }),
        );
        self.add_entity(p1_right);

        // Player 2 towers (top side)
        // King tower: center of 4x4 zone (x=7-10, y=1-4) = (8.5, 2.5)
        let p2_king = Entity::new(
            PlayerId::Player2,
            Position::new(8.5, 2.5),
            EntityKind::Tower(TowerData {
                base_hp: 2400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 1.0,
            }),
        );
        self.add_entity(p2_king);

        // Left Princess tower: center of 3x3 zone (x=2-4, y=5-7) = (3.0, 6.0)
        let p2_left = Entity::new(
            PlayerId::Player2,
            Position::new(3.0, 6.0),
            EntityKind::Tower(TowerData {
                base_hp: 1400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 0.8,
            }),
        );
        self.add_entity(p2_left);

        // Right Princess tower: center of 3x3 zone (x=13-15, y=5-7) = (14.0, 6.0)
        let p2_right = Entity::new(
            PlayerId::Player2,
            Position::new(14.0, 6.0),
            EntityKind::Tower(TowerData {
                base_hp: 1400.0,
                damage: 50.0,
                range: 7.0,
                attack_speed: 0.8,
            }),
        );
        self.add_entity(p2_right);
    }

    /// Loads cards from a JSON file.
    pub fn load_cards(&mut self, cards: Vec<Card>) {
        self.cards.clear();
        for card in cards {
            self.cards.insert(card.name.clone(), card);
        }
    }

    /// Gets a card by name.
    pub fn get_card_by_name(&self, name: &str) -> Option<&Card> {
        self.cards.get(name)
    }

    /// Initializes a player's deck with the given card names.
    /// The deck will be shuffled deterministically using the game's RNG.
    pub fn set_player_deck(&mut self, player_id: PlayerId, deck: Vec<String>) -> Result<()> {
        let player = self
            .players
            .get_mut(&player_id)
            .ok_or_else(|| shared::Error::InvalidAction("Player not found".to_string()))?;

        // Validate that all cards exist
        for card_name in &deck {
            if !self.cards.contains_key(card_name) {
                return Err(shared::Error::InvalidAction(format!(
                    "Card '{}' not found in available cards",
                    card_name
                )));
            }
        }

        player.set_deck(deck, &mut self.rng);
        Ok(())
    }

    /// Applies a player action to the game state.
    pub fn apply_action(&mut self, action: &Action) -> Result<()> {
        action.apply(self)
    }

    /// Allocates a new entity ID.
    pub fn allocate_entity_id(&mut self) -> EntityId {
        let id = self.next_entity_id;
        self.next_entity_id += 1;
        EntityId(id)
    }

    /// Adds an entity to the game.
    pub fn add_entity(&mut self, entity: Entity) -> EntityId {
        let id = self.allocate_entity_id();
        self.entities.insert(id, entity);
        id
    }

    /// Removes an entity from the game.
    pub fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        self.entities.remove(&id)
    }

    /// Checks if the match has ended.
    pub fn is_match_over(&self) -> bool {
        self.match_time >= self.max_match_time
            || self.players.values().any(|p| p.is_defeated())
    }

    /// Advances match time by delta.
    pub fn advance_time(&mut self, delta: f32) {
        self.match_time += delta;
    }
}

/// Unique identifier for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EntityId(u32);

impl EntityId {
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    pub fn from_u32(id: u32) -> Self {
        EntityId(id)
    }
}

/// Player-specific state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub id: PlayerId,
    pub elixir: f32,
    pub max_elixir: f32,
    pub elixir_regen_rate: f32,
    pub tower_hp: HashMap<TowerType, f32>,

    /// The player's 8-card deck (card names).
    pub deck: Vec<String>,

    /// Current hand (4 card indices into the cycle).
    pub hand: Vec<usize>,

    /// Current position in the deck cycle (0-7, wraps around).
    pub next_card_index: usize,
}

impl PlayerState {
    /// Creates a new player state with an empty deck.
    /// Use `set_deck()` to initialize the deck and hand.
    pub fn new(id: PlayerId) -> Self {
        let mut tower_hp = HashMap::new();
        tower_hp.insert(TowerType::King, 2400.0);
        tower_hp.insert(TowerType::LeftPrincess, 1400.0);
        tower_hp.insert(TowerType::RightPrincess, 1400.0);

        Self {
            id,
            elixir: 5.0,
            max_elixir: 10.0,
            elixir_regen_rate: 1.0, // 1 elixir per second (will be configurable)
            tower_hp,
            deck: Vec::new(),
            hand: Vec::new(),
            next_card_index: 0,
        }
    }

    /// Sets the player's deck and initializes the hand with the first 4 cards.
    /// The deck should contain exactly 8 card names.
    pub fn set_deck(&mut self, deck: Vec<String>, rng: &mut crate::rng::Rng) {
        assert_eq!(deck.len(), 8, "Deck must contain exactly 8 cards");
        self.deck = deck;

        // Shuffle the initial deck order using RNG for determinism
        for i in (1..self.deck.len()).rev() {
            let j = rng.rand_int_range(0, i as i32 + 1) as usize;
            self.deck.swap(i, j);
        }

        // Initialize hand with first 4 cards (indices 0-3)
        self.hand = vec![0, 1, 2, 3];
        self.next_card_index = 4; // Next card to draw is at index 4
    }

    /// Gets the card name at the given hand index (0-3).
    pub fn get_hand_card(&self, hand_index: usize) -> Option<&String> {
        self.hand.get(hand_index).and_then(|&deck_index| self.deck.get(deck_index))
    }

    /// Plays a card from the hand and cycles in the next card.
    /// Returns the card name that was played.
    pub fn play_card_from_hand(&mut self, hand_index: usize) -> Option<String> {
        if hand_index >= self.hand.len() {
            return None;
        }

        // Get the card name before removing it
        let deck_index = self.hand[hand_index];
        let card_name = self.deck.get(deck_index)?.clone();

        // Replace this hand slot with the next card in the cycle
        self.hand[hand_index] = self.next_card_index;

        // Advance the cycle (wraps around to 0 after 7)
        self.next_card_index = (self.next_card_index + 1) % 8;

        Some(card_name)
    }

    /// Checks if this player has been defeated (King tower destroyed).
    pub fn is_defeated(&self) -> bool {
        self.tower_hp.get(&TowerType::King).copied().unwrap_or(0.0) <= 0.0
    }

    /// Adds elixir, capped at max.
    pub fn add_elixir(&mut self, amount: f32) {
        self.elixir = (self.elixir + amount).min(self.max_elixir);
    }

    /// Attempts to spend elixir. Returns true if successful.
    pub fn spend_elixir(&mut self, cost: f32) -> bool {
        if self.elixir >= cost {
            self.elixir -= cost;
            true
        } else {
            false
        }
    }
}

/// Tower types in the arena.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TowerType {
    King,
    LeftPrincess,
    RightPrincess,
}
