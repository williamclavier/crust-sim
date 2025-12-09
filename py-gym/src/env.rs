/*!
# ClashEnv - Main Environment Class

Core RL environment that wraps the game engine and provides the Gymnasium interface.
*/

use pyo3::prelude::*;
use pyo3::types::PyDict;
use engine::GameState;

use crate::{Observation, Action, StepResult, EnvError, RewardScheme};

/// Main RL environment exposed to Python
///
/// Example usage from Python:
/// ```python
/// env = ClashEnv(seed=42, reward_scheme="shaped")
/// obs = env.reset()
/// action = env.action_space.sample()
/// result = env.step(action)
/// ```
#[pyclass]
pub struct ClashEnv {
    /// Current game state
    game_state: Option<GameState>,

    /// RNG seed for determinism
    rng_seed: u64,

    /// Current tick count
    tick: u32,

    /// Maximum ticks per episode (3 minutes @ 60 FPS = 10800 ticks)
    max_ticks: u32,

    /// Reward computation scheme
    reward_scheme: RewardScheme,

    /// Previous game state (for reward computation)
    prev_state: Option<GameState>,
}

#[pymethods]
impl ClashEnv {
    /// Create a new environment
    ///
    /// Args:
    ///     seed: Optional RNG seed for deterministic simulation
    ///     reward_scheme: Reward function type ("sparse", "shaped", "curriculum")
    ///     max_ticks: Maximum ticks per episode (default: 10800 = 3 minutes)
    #[new]
    #[pyo3(signature = (seed=None, reward_scheme="shaped", max_ticks=10800))]
    pub fn new(
        seed: Option<u64>,
        reward_scheme: &str,
        max_ticks: u32,
    ) -> PyResult<Self> {
        let rng_seed = seed.unwrap_or_else(|| {
            use std::time::{SystemTime, UNIX_EPOCH};
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs()
        });

        let reward_scheme = reward_scheme.parse()
            .map_err(|e: String| PyErr::new::<pyo3::exceptions::PyValueError, _>(e))?;

        Ok(Self {
            game_state: None,
            rng_seed,
            tick: 0,
            max_ticks,
            reward_scheme,
            prev_state: None,
        })
    }

    /// Reset the environment to initial state
    ///
    /// Returns:
    ///     Initial observation
    pub fn reset(&mut self) -> PyResult<Observation> {
        // TODO: Initialize GameState with self.rng_seed
        // For now, return a placeholder
        self.tick = 0;
        self.prev_state = None;

        // Placeholder observation
        Ok(Observation::default())
    }

    /// Execute an action and step the simulation
    ///
    /// Args:
    ///     action: Action tuple (card_index, x, y)
    ///
    /// Returns:
    ///     StepResult containing (observation, reward, terminated, truncated, info)
    pub fn step(&mut self, py: Python, action: Action) -> PyResult<StepResult> {
        let game_state = self.game_state.as_ref()
            .ok_or_else(|| EnvError::NotInitialized)?;

        // TODO: Validate action
        // TODO: Execute action in game engine
        // TODO: Compute reward
        // TODO: Check termination conditions

        self.tick += 1;

        let terminated = false;  // TODO: Check if game is over
        let truncated = self.tick >= self.max_ticks;

        let info = PyDict::new(py);
        info.set_item("tick", self.tick)?;

        Ok(StepResult {
            observation: Observation::default(),
            reward: 0.0,
            terminated,
            truncated,
            info: info.into(),
        })
    }

    /// Render the current state (for debugging)
    ///
    /// Args:
    ///     mode: Render mode ("human", "ansi", "rgb_array")
    ///
    /// Returns:
    ///     Rendered state as string or array
    #[pyo3(signature = (mode="human"))]
    pub fn render(&self, mode: &str) -> PyResult<String> {
        match mode {
            "human" | "ansi" => {
                // TODO: Return ASCII representation
                Ok(format!("Tick: {}/{}", self.tick, self.max_ticks))
            }
            "rgb_array" => {
                // TODO: Return pixel array
                Err(PyErr::new::<pyo3::exceptions::PyNotImplementedError, _>(
                    "rgb_array rendering not implemented yet"
                ))
            }
            _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                format!("Invalid render mode: {}", mode)
            )),
        }
    }

    /// Update the RNG seed
    pub fn seed(&mut self, seed: u64) {
        self.rng_seed = seed;
    }

    /// Get action mask (valid actions for current state)
    ///
    /// Returns:
    ///     Boolean mask indicating which actions are valid
    pub fn get_action_mask(&self) -> PyResult<Vec<bool>> {
        // TODO: Return mask based on available elixir, card cooldowns, etc.
        Ok(vec![true; 4])  // Placeholder: all 4 cards available
    }

    /// Get current game state as JSON (for debugging)
    pub fn get_state_json(&self) -> PyResult<String> {
        match &self.game_state {
            Some(state) => {
                // TODO: Serialize game state
                Ok("{}".to_string())
            }
            None => Err(EnvError::NotInitialized.into()),
        }
    }

    fn __repr__(&self) -> String {
        format!(
            "ClashEnv(seed={}, tick={}/{}, scheme={:?})",
            self.rng_seed, self.tick, self.max_ticks, self.reward_scheme
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_env_creation() {
        pyo3::prepare_freethreaded_python();
        Python::with_gil(|py| {
            let env = ClashEnv::new(Some(42), "shaped", 10800).unwrap();
            assert_eq!(env.rng_seed, 42);
            assert_eq!(env.max_ticks, 10800);
        });
    }
}
