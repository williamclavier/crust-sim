/*!
# Observation Space

Defines the observation structure returned to RL agents.

Observation components:
- `arena_grid`: 32x18x6 multi-channel grid (unit positions, HP, teams, etc.)
- `player_state`: 2x3 matrix (elixir, tower HP for both players)
- `hand`: 4-element vector of card IDs
- `tick`: Current game tick
*/

use pyo3::prelude::*;
use numpy::{PyArray3, PyArray2, PyArray1};

/// Observation returned from the environment
///
/// Corresponds to Gymnasium Dict space:
/// ```python
/// {
///     "arena_grid": Box(0, 255, shape=(32, 18, 6), dtype=uint8),
///     "player_state": Box(0, 10, shape=(2, 3), dtype=float32),
///     "hand": MultiDiscrete([99, 99, 99, 99]),
///     "tick": Box(0, 10800, shape=(1,), dtype=int32),
/// }
/// ```
#[pyclass]
#[derive(Debug, Clone)]
pub struct Observation {
    /// Arena grid: 32x18x6 (width, height, channels)
    ///
    /// Channels:
    /// 0: Unit presence (0=empty, 1-99=card_id)
    /// 1: Unit HP (normalized 0-255)
    /// 2: Unit team (0=neutral, 1=player, 2=opponent)
    /// 3: Unit target presence (0=none, 1=has target)
    /// 4: Projectile presence (0=none, 1-99=projectile type)
    /// 5: Building/structure presence (0=none, 1-99=structure_id)
    #[pyo3(get)]
    pub arena_grid: Py<PyArray3<u8>>,

    /// Player state: 2x3 matrix
    /// Row 0: [player_elixir, player_left_tower_hp, player_right_tower_hp]
    /// Row 1: [opponent_elixir, opponent_left_tower_hp, opponent_right_tower_hp]
    #[pyo3(get)]
    pub player_state: Py<PyArray2<f32>>,

    /// Current hand: 4 card IDs (0-98)
    #[pyo3(get)]
    pub hand: Py<PyArray1<u8>>,

    /// Current game tick
    #[pyo3(get)]
    pub tick: u32,
}

impl Default for Observation {
    fn default() -> Self {
        Python::with_gil(|py| Self::zeros(py))
    }
}

impl Observation {
    /// Create a zero-initialized observation (for testing)
    pub fn zeros(py: Python) -> Self {
        Self {
            arena_grid: PyArray3::<u8>::zeros(py, [32, 18, 6], false).into(),
            player_state: PyArray2::<f32>::zeros(py, [2, 3], false).into(),
            hand: PyArray1::<u8>::zeros(py, 4, false).into(),
            tick: 0,
        }
    }

    /// Create observation from game state (TODO: implement)
    pub fn from_game_state(py: Python, _game_state: &engine::GameState) -> Self {
        // TODO: Extract observation from GameState
        // - Encode units into arena_grid
        // - Extract player stats
        // - Get current hand
        Self::zeros(py)
    }
}

#[pymethods]
impl Observation {
    fn __repr__(&self) -> String {
        format!(
            "Observation(arena_grid={}x{}x6, player_state=2x3, hand=[...], tick={})",
            32, 18, self.tick
        )
    }

    /// Get observation as dictionary (for debugging)
    fn to_dict(&self, py: Python) -> PyResult<Py<pyo3::types::PyDict>> {
        let dict = pyo3::types::PyDict::new(py);
        dict.set_item("arena_grid", self.arena_grid.clone_ref(py))?;
        dict.set_item("player_state", self.player_state.clone_ref(py))?;
        dict.set_item("hand", self.hand.clone_ref(py))?;
        dict.set_item("tick", self.tick)?;
        Ok(dict.into())
    }
}

/// Arena grid channel indices
#[allow(dead_code)]
pub mod channels {
    pub const UNIT_PRESENCE: usize = 0;
    pub const UNIT_HP: usize = 1;
    pub const UNIT_TEAM: usize = 2;
    pub const UNIT_TARGET: usize = 3;
    pub const PROJECTILE: usize = 4;
    pub const BUILDING: usize = 5;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_creation() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let obs = Observation::zeros(py);
            assert_eq!(obs.tick, 0);
        });
    }
}
