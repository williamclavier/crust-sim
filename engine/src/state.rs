//! Game state management and serialization.
use crate::action::Action;
use crate::card::Card;
use crate::entities::Entity;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use shared::{PlayerId, Result, CRState, Tower as CRTower, Unit as CRUnit, LegalMasks};

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

    /// Next entity ID to assign.
    next_entity_id: u32,

    /// Game match time in seconds.
    pub match_time: f32,

    /// Maximum match duration (in seconds).
    pub max_match_time: f32,
}

fn extract_entity_info(e: &Entity) -> Option<(PlayerId, (f32, f32), (f32, f32))> {
    // Extract owner, position (x, y), and velocity (vx, vy) from Entity
    // Only include movable troop entities (not towers or projectiles)
    match &e.kind {
        crate::entities::EntityKind::Troop(_) => {
            Some((
                e.owner,
                (e.position.x, e.position.y),
                (e.velocity.x, e.velocity.y),
            ))
        }
        _ => None, // Skip towers, projectiles, and spells
    }
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

        Self {
            tick: 0,
            rng: Rng::new(seed),
            entities: HashMap::new(),
            players,
            cards,
            next_entity_id: 1,
            match_time: 0.0,
            max_match_time: 180.0, // 3 minutes (will be configurable)
        }
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

impl GameState {
    /// Export a snapshot of the game for RL / external control.
    /// `pov` = which player is considered "ALLY" (usually Player1).
    pub fn export_cr_state(&self, pov: PlayerId) -> CRState {
        // Decide who is ally vs enemy from the POV
        let (ally_id, enemy_id) = match pov {
            PlayerId::Player1 => (PlayerId::Player1, PlayerId::Player2),
            PlayerId::Player2 => (PlayerId::Player2, PlayerId::Player1),
        };

        let ally_player = self.players.get(&ally_id).expect("ally player missing");
        let enemy_player = self.players.get(&enemy_id).expect("enemy player missing");

        // === Tower snapshots ===

        const KING_MAX_HP: f32 = 2400.0;
        const PRINCESS_MAX_HP: f32 = 1400.0;

        // TEMP: positions are rough placeholders; tweak later.
        fn tower_pos_for(player: PlayerId, tt: TowerType) -> (f32, f32) {
            match (player, tt) {
                // Player1 bottom, Player2 top (arbitrary grid coords)
                (PlayerId::Player1, TowerType::King)          => (16.0,  2.0),
                (PlayerId::Player1, TowerType::LeftPrincess)  => (8.0,   4.0),
                (PlayerId::Player1, TowerType::RightPrincess) => (24.0,  4.0),
                (PlayerId::Player2, TowerType::King)          => (16.0, 30.0),
                (PlayerId::Player2, TowerType::LeftPrincess)  => (8.0,  28.0),
                (PlayerId::Player2, TowerType::RightPrincess) => (24.0, 28.0),
            }
        }

        let mut ally_towers = Vec::new();
        let mut enemy_towers = Vec::new();

        for (&tt, &hp) in &ally_player.tower_hp {
            let max_hp = match tt {
                TowerType::King => KING_MAX_HP,
                TowerType::LeftPrincess | TowerType::RightPrincess => PRINCESS_MAX_HP,
            };
            let (x, y) = tower_pos_for(ally_id, tt);
            ally_towers.push(CRTower {
                owner: "ALLY".to_string(),
                x,
                y,
                hp_frac: (hp / max_hp).clamp(0.0, 1.0),
            });
        }

        for (&tt, &hp) in &enemy_player.tower_hp {
            let max_hp = match tt {
                TowerType::King => KING_MAX_HP,
                TowerType::LeftPrincess | TowerType::RightPrincess => PRINCESS_MAX_HP,
            };
            let (x, y) = tower_pos_for(enemy_id, tt);
            enemy_towers.push(CRTower {
                owner: "ENEMY".to_string(),
                x,
                y,
                hp_frac: (hp / max_hp).clamp(0.0, 1.0),
            });
        }

        // === Units (currently empty; we’ll fill later) ===

        let mut ally_units: Vec<CRUnit> = Vec::new();
        let mut enemy_units: Vec<CRUnit> = Vec::new();

        for entity in self.entities.values() {
            if let Some((owner_id, (x, y), (vx, vy))) = extract_entity_info(entity) {
                let owner_str = if owner_id == ally_id { "ALLY" } else { "ENEMY" }.to_string();
                let unit = CRUnit {
                    owner: owner_str,
                    x,
                    y,
                    vx,
                    vy,
                };
                if owner_id == ally_id {
                    ally_units.push(unit);
                } else {
                    enemy_units.push(unit);
                }
            }
        }

        // === Legal masks (placeholder; everything allowed for now) ===

        let legal = LegalMasks {
            cards: vec![true; 8],        // 8 hand slots
            tiles_flat: vec![true; 16 * 9], // 16x9 placement grid
        };

        // === Damage-based helpers ===

        let ally_total_hp: f32  = ally_player.tower_hp.values().sum();
        let enemy_total_hp: f32 = enemy_player.tower_hp.values().sum();
        let ally_max_total  = KING_MAX_HP + 2.0 * PRINCESS_MAX_HP;
        let enemy_max_total = ally_max_total;

        let ally_tower_hp_drop  = (ally_max_total  - ally_total_hp).max(0.0);
        let enemy_tower_hp_drop = (enemy_max_total - enemy_total_hp).max(0.0);

        // === Win / lose flags ===

        let win  = enemy_player.is_defeated()
            || (self.is_match_over() && ally_total_hp > enemy_total_hp);
        let lose = ally_player.is_defeated()
            || (self.is_match_over() && enemy_total_hp > ally_total_hp);

        CRState {
            t_ms: (self.match_time * 1000.0) as u64,
            ally_elixir: ally_player.elixir,
            time_left: (self.max_match_time - self.match_time).max(0.0),
            overtime: self.match_time > self.max_match_time,

            ally_towers,
            enemy_towers,
            ally_units,
            enemy_units,

            legal,

            win,
            lose,

            enemy_tower_hp_drop,
            ally_tower_hp_drop,
        }
    }
}


pub fn step_with_action(
    game: &mut GameState,
    pov: PlayerId,
    card_idx: usize,
    tile_idx: usize,
) {
    eprintln!(
        "step_with_action: pov={:?}, card_idx={}, tile_idx={}, match_time={}",
        pov, card_idx, tile_idx, game.match_time
    );

    // 1) Choose which player is "us"
    let player_id = pov;

    // 2) Get mutable reference to that player's state
    let player_state = match game.players.get_mut(&player_id) {
        Some(p) => p,
        None => {
            eprintln!("step_with_action: player {:?} not found", player_id);
            return;
        }
    };

    // Track elixir and entity count before action
    let elixir_before = player_state.elixir;
    let entity_count_before = game.entities.len();

    // 3) Validate card_idx (0–3 for the 4-card hand)
    if card_idx >= player_state.hand.len() {
        eprintln!("step_with_action: invalid card_idx {}", card_idx);
        return;
    }

    // Which card in the deck does this hand slot point to?
    let deck_index = player_state.hand[card_idx];
    let maybe_card_name = player_state.deck.get(deck_index).cloned();
    let card_name = match maybe_card_name {
        Some(name) => name,
        None => {
            eprintln!(
                "step_with_action: no card at deck index {} for player {:?}",
                deck_index, player_id
            );
            return;
        }
    };

    // 4) Convert tile_idx into an (x, y) placement in a 16x9 grid
    let grid_w = 16;
    let grid_h = 9;
    if tile_idx >= grid_w * grid_h {
        eprintln!("step_with_action: invalid tile_idx {}", tile_idx);
        return;
    }
    let gx = (tile_idx % grid_w) as f32;
    let gy = (tile_idx / grid_w) as f32;

    // TODO: if you want world coords, convert (gx, gy) via your Arena
    let x = gx;
    let y = gy;

    // 5) Build an Action that your engine understands
    // Action::PlayCard expects: player, card_name, level, position
    // Use level 11 as default (matches test cards)
    let position = shared::Position::new(x, y);
    let action = Action::PlayCard {
        player: player_id,
        card_name: card_name.clone(),
        level: 11,
        position,
    };

    eprintln!(
        "step_with_action: applying PlayCard(player={:?}, card={}, level=11, position=({}, {}))",
        player_id, card_name, x, y
    );

    // 6) Apply the action
    if let Err(e) = game.apply_action(&action) {
        eprintln!("step_with_action: apply_action error: {:?}", e);
        // Still advance time even if action fails
    } else {
        // Log elixir and entity changes after successful action
        let elixir_after = game.players.get(&player_id).map(|p| p.elixir).unwrap_or(0.0);
        let entity_count_after = game.entities.len();
        eprintln!(
            "step_with_action: card '{}' played. elixir: {} -> {}, entities: {} -> {}",
            card_name, elixir_before, elixir_after, entity_count_before, entity_count_after
        );
    }

    // 7) Advance the simulation by Δt
    let delta_t = 1.0;
    game.advance_time(delta_t);

    eprintln!(
        "step_with_action: finished, new match_time={}, ally elixir={}",
        game.match_time,
        game.players
            .get(&player_id)
            .map(|p| p.elixir)
            .unwrap_or(-1.0)
    );
}