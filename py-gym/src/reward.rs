/*!
# Reward Shaping

Implements configurable reward functions for training RL agents.

Reward schemes:
- `sparse`: Only terminal rewards (tower kills, game outcome)
- `shaped`: Dense rewards (tower damage, elixir efficiency)
- `curriculum`: Starts shaped, anneals to sparse over time
*/

use std::str::FromStr;
use engine::GameState;

/// Reward computation scheme
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RewardScheme {
    /// Sparse rewards: only tower kills and game outcome
    Sparse,

    /// Dense shaped rewards: tower damage + elixir efficiency
    Shaped,

    /// Curriculum learning: starts shaped, anneals to sparse
    Curriculum,
}

impl FromStr for RewardScheme {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "sparse" => Ok(Self::Sparse),
            "shaped" => Ok(Self::Shaped),
            "curriculum" => Ok(Self::Curriculum),
            _ => Err(format!("Invalid reward scheme: {}", s)),
        }
    }
}

/// Compute reward for a transition
///
/// Args:
///     prev_state: Previous game state
///     next_state: Current game state
///     scheme: Reward computation scheme
///     training_progress: Progress through training [0, 1] (for curriculum)
///
/// Returns:
///     Scalar reward value
pub fn compute_reward(
    prev_state: &GameState,
    next_state: &GameState,
    scheme: RewardScheme,
    training_progress: f32,
) -> f32 {
    match scheme {
        RewardScheme::Sparse => compute_sparse_reward(prev_state, next_state),
        RewardScheme::Shaped => compute_shaped_reward(prev_state, next_state),
        RewardScheme::Curriculum => {
            // Interpolate between shaped and sparse
            let shaped = compute_shaped_reward(prev_state, next_state);
            let sparse = compute_sparse_reward(prev_state, next_state);
            shaped * (1.0 - training_progress) + sparse * training_progress
        }
    }
}

/// Sparse reward: only terminal events
fn compute_sparse_reward(prev_state: &GameState, next_state: &GameState) -> f32 {
    let mut reward = 0.0;

    // TODO: Implement sparse reward logic
    // - Tower destruction: ±5.0
    // - Game outcome: ±100.0 (win/loss)

    reward
}

/// Dense shaped reward: includes intermediate signals
fn compute_shaped_reward(prev_state: &GameState, next_state: &GameState) -> f32 {
    let mut reward = 0.0;

    // TODO: Implement shaped reward logic
    // 1. Tower damage: ±0.01 per HP
    // 2. Tower destruction: ±5.0
    // 3. Elixir advantage: ±0.001
    // 4. Game outcome: ±100.0

    reward
}

/// Reward weights (tunable hyperparameters)
#[allow(dead_code)]
pub struct RewardWeights {
    /// Reward per point of tower damage dealt
    pub tower_damage: f32,

    /// Bonus for destroying a tower
    pub tower_kill: f32,

    /// Reward for winning the game
    pub victory: f32,

    /// Reward per point of elixir advantage
    pub elixir_efficiency: f32,

    /// Penalty for invalid actions
    pub invalid_action: f32,
}

impl Default for RewardWeights {
    fn default() -> Self {
        Self {
            tower_damage: 0.01,
            tower_kill: 5.0,
            victory: 100.0,
            elixir_efficiency: 0.001,
            invalid_action: -1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reward_scheme_parsing() {
        assert_eq!("sparse".parse::<RewardScheme>().unwrap(), RewardScheme::Sparse);
        assert_eq!("shaped".parse::<RewardScheme>().unwrap(), RewardScheme::Shaped);
        assert_eq!("curriculum".parse::<RewardScheme>().unwrap(), RewardScheme::Curriculum);
        assert!("invalid".parse::<RewardScheme>().is_err());
    }

    #[test]
    fn test_reward_weights_default() {
        let weights = RewardWeights::default();
        assert_eq!(weights.tower_damage, 0.01);
        assert_eq!(weights.tower_kill, 5.0);
        assert_eq!(weights.victory, 100.0);
    }
}
