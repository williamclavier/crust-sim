/*!
# Action Space

Defines the action structure and validation logic.

Action space: Tuple(Discrete(4), Box(0-31), Box(0-17))
- Card index (0-3): Which card in hand to play
- X position (0-31): Continuous x coordinate
- Y position (0-17): Continuous y coordinate
*/

use pyo3::prelude::*;
use engine::{GameState, Position};

/// Action for the RL agent
///
/// Corresponds to Gymnasium action space:
/// ```python
/// Tuple((
///     Discrete(4),        # card_index
///     Box(0, 31, (1,)),   # x position
///     Box(0, 17, (1,)),   # y position
/// ))
/// ```
#[pyclass]
#[derive(Debug, Clone)]
pub struct Action {
    /// Which card in hand to play (0-3)
    #[pyo3(get)]
    pub card_index: u8,

    /// X position (continuous, will be clamped to [0, 31])
    #[pyo3(get)]
    pub x: f32,

    /// Y position (continuous, will be clamped to [0, 17])
    #[pyo3(get)]
    pub y: f32,
}

#[pymethods]
impl Action {
    #[new]
    pub fn new(card_index: u8, x: f32, y: f32) -> Self {
        Self {
            card_index,
            x: x.clamp(0.0, 31.0),
            y: y.clamp(0.0, 17.0),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "Action(card_index={}, x={:.2}, y={:.2})",
            self.card_index, self.x, self.y
        )
    }
}

impl Action {
    /// Validate action against current game state
    ///
    /// Checks:
    /// - Card index is valid (0-3)
    /// - Player has enough elixir
    /// - Position is valid for card type
    /// - Position is on player's side of arena
    pub fn is_valid(&self, _game_state: &GameState) -> bool {
        // Basic validation
        if self.card_index >= 4 {
            return false;
        }

        // TODO: Check elixir cost
        // TODO: Check if position is on player's side
        // TODO: Check if card can be played at position (e.g., buildings only in certain areas)

        true
    }

    /// Convert to game engine action
    ///
    /// Clamps position to player's side (bottom half: y ∈ [9, 17])
    pub fn to_game_action(&self, _game_state: &GameState) -> Option<GameAction> {
        // TODO: Implement conversion to engine's action type
        // - Get card from hand
        // - Clamp y to player's side
        // - Create GameAction::PlayCard

        None
    }
}

/// Action space description
#[pyclass]
#[derive(Debug, Clone)]
pub struct ActionSpace {
    /// Number of cards in hand
    #[pyo3(get)]
    pub num_cards: usize,

    /// Arena width
    #[pyo3(get)]
    pub arena_width: usize,

    /// Arena height
    #[pyo3(get)]
    pub arena_height: usize,
}

#[pymethods]
impl ActionSpace {
    #[new]
    pub fn new() -> Self {
        Self {
            num_cards: 4,
            arena_width: 32,
            arena_height: 18,
        }
    }

    /// Sample a random action (for testing)
    pub fn sample(&self) -> Action {
        use oorandom::Rand32;
        let mut rng = Rand32::new(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        );

        Action {
            card_index: (rng.rand_u32() % self.num_cards as u32) as u8,
            x: (rng.rand_float() * self.arena_width as f32),
            y: (rng.rand_float() * self.arena_height as f32),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ActionSpace(Tuple(Discrete({}), Box([0, {}]), Box([0, {}])))",
            self.num_cards, self.arena_width - 1, self.arena_height - 1
        )
    }
}

/// Placeholder for game action (TODO: use actual engine type)
#[allow(dead_code)]
pub enum GameAction {
    PlayCard {
        card_id: u32,
        position: Position,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_creation() {
        let action = Action::new(2, 15.5, 10.2);
        assert_eq!(action.card_index, 2);
        assert!((action.x - 15.5).abs() < 0.01);
        assert!((action.y - 10.2).abs() < 0.01);
    }

    #[test]
    fn test_action_clamping() {
        let action = Action::new(0, -5.0, 100.0);
        assert_eq!(action.x, 0.0);
        assert_eq!(action.y, 17.0);
    }

    #[test]
    fn test_action_space_sample() {
        pyo3::prepare_freethreaded_python();
        let space = ActionSpace::new();
        let action = space.sample();
        assert!(action.card_index < 4);
        assert!(action.x >= 0.0 && action.x <= 31.0);
        assert!(action.y >= 0.0 && action.y <= 17.0);
    }
}
