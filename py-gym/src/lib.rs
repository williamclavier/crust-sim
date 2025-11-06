/*!
# Crust Gym - Python RL Bindings

PyO3 bindings for the Clash Royale simulation engine, exposing a Gymnasium-compatible
reinforcement learning environment.

## Architecture

This crate provides:
- `ClashEnv`: Main environment class exposed to Python
- Observation/action space definitions
- Reward computation functions
- Episode management

See `docs/PYTHON_RL_INTEGRATION.md` for detailed architecture documentation.
*/

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;

// Re-export engine types
use engine::{GameState, Position};

mod env;
mod observation;
mod action;
mod reward;

pub use env::ClashEnv;
pub use observation::Observation;
pub use action::{Action, ActionSpace};
pub use reward::{RewardScheme, compute_reward};

/// Python module definition
#[pymodule]
fn _native(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<ClashEnv>()?;
    m.add_class::<Observation>()?;
    m.add_class::<Action>()?;
    m.add_class::<StepResult>()?;

    // Add version info
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;

    Ok(())
}

/// Result returned from env.step()
/// Follows Gymnasium API: (observation, reward, terminated, truncated, info)
#[pyclass]
#[derive(Debug, Clone)]
pub struct StepResult {
    #[pyo3(get)]
    pub observation: Observation,

    #[pyo3(get)]
    pub reward: f32,

    #[pyo3(get)]
    pub terminated: bool,

    #[pyo3(get)]
    pub truncated: bool,

    #[pyo3(get)]
    pub info: Py<PyDict>,
}

#[pymethods]
impl StepResult {
    fn __repr__(&self) -> String {
        format!(
            "StepResult(reward={:.2}, terminated={}, truncated={})",
            self.reward, self.terminated, self.truncated
        )
    }
}

/// Error types for the environment
#[derive(Debug, thiserror::Error)]
pub enum EnvError {
    #[error("Invalid action: {0}")]
    InvalidAction(String),

    #[error("Environment not initialized")]
    NotInitialized,

    #[error("Episode already finished")]
    EpisodeFinished,

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

impl From<EnvError> for PyErr {
    fn from(err: EnvError) -> PyErr {
        pyo3::exceptions::PyRuntimeError::new_err(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_version() {
        assert!(!env!("CARGO_PKG_VERSION").is_empty());
    }
}
