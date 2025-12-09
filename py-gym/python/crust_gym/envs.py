"""
Gymnasium Environment Wrapper

Provides a Gymnasium-compatible interface to the Rust simulation engine.
"""

from typing import Any, Dict, Optional, Tuple, Union

import gymnasium as gym
import numpy as np
from gymnasium import spaces

from crust_gym._native import ClashEnv


class ClashRoyaleEnv(gym.Env):
    """
    Clash Royale RL Environment

    Observation Space:
        Dict({
            "arena_grid": Box(0, 255, shape=(32, 18, 6), dtype=uint8),
            "player_state": Box(0, 10, shape=(2, 3), dtype=float32),
            "hand": MultiDiscrete([99, 99, 99, 99]),
            "tick": Box(0, 10800, shape=(1,), dtype=int32),
        })

    Action Space:
        Tuple((
            Discrete(4),        # Card index (0-3)
            Box(0, 31, (1,)),   # X position (continuous)
            Box(0, 17, (1,)),   # Y position (continuous)
        ))

    Reward:
        Configurable via reward_scheme parameter:
        - "sparse": Only tower kills and game outcome
        - "shaped": Dense rewards (tower damage + elixir efficiency)
        - "curriculum": Starts shaped, anneals to sparse

    Args:
        seed: Random seed for deterministic simulation
        reward_scheme: Reward function type ("sparse", "shaped", "curriculum")
        max_ticks: Maximum ticks per episode (default: 10800 = 3 minutes @ 60 FPS)
        render_mode: Rendering mode ("human", "ansi", "rgb_array")
    """

    metadata = {"render_modes": ["human", "ansi", "rgb_array"], "render_fps": 60}

    def __init__(
        self,
        seed: Optional[int] = None,
        reward_scheme: str = "shaped",
        max_ticks: int = 10800,
        render_mode: Optional[str] = None,
    ):
        super().__init__()

        # Initialize Rust environment
        self._env = ClashEnv(seed=seed, reward_scheme=reward_scheme, max_ticks=max_ticks)
        self.render_mode = render_mode

        # Define observation space
        self.observation_space = spaces.Dict(
            {
                "arena_grid": spaces.Box(
                    low=0, high=255, shape=(32, 18, 6), dtype=np.uint8
                ),
                "player_state": spaces.Box(
                    low=0.0, high=10.0, shape=(2, 3), dtype=np.float32
                ),
                "hand": spaces.MultiDiscrete([99, 99, 99, 99]),
                "tick": spaces.Box(low=0, high=max_ticks, shape=(1,), dtype=np.int32),
            }
        )

        # Define action space
        self.action_space = spaces.Tuple(
            (
                spaces.Discrete(4),  # Card index
                spaces.Box(low=0.0, high=31.0, shape=(1,), dtype=np.float32),  # X
                spaces.Box(low=0.0, high=17.0, shape=(1,), dtype=np.float32),  # Y
            )
        )

    def reset(
        self,
        seed: Optional[int] = None,
        options: Optional[Dict[str, Any]] = None,
    ) -> Tuple[Dict[str, np.ndarray], Dict[str, Any]]:
        """
        Reset the environment to initial state.

        Args:
            seed: Optional seed for reproducibility
            options: Additional reset options

        Returns:
            observation: Initial observation
            info: Additional information
        """
        super().reset(seed=seed)

        if seed is not None:
            self._env.seed(seed)

        # Get initial observation from Rust
        obs = self._env.reset()

        # Convert to dict format
        observation = self._format_observation(obs)

        info = {}
        return observation, info

    def step(
        self, action: Tuple[int, np.ndarray, np.ndarray]
    ) -> Tuple[Dict[str, np.ndarray], float, bool, bool, Dict[str, Any]]:
        """
        Execute an action in the environment.

        Args:
            action: Tuple of (card_index, x_position, y_position)

        Returns:
            observation: Next observation
            reward: Reward for this transition
            terminated: Whether episode ended naturally (tower destroyed)
            truncated: Whether episode was cut off (max ticks reached)
            info: Additional information
        """
        # Unpack action
        card_index, x, y = action
        x = float(x[0]) if isinstance(x, np.ndarray) else float(x)
        y = float(y[0]) if isinstance(y, np.ndarray) else float(y)

        # Execute action in Rust environment
        from crust_gym._native import Action

        rust_action = Action(card_index, x, y)
        result = self._env.step(rust_action)

        # Format observation
        observation = self._format_observation(result.observation)

        # Extract info dict from PyO3
        info = dict(result.info) if hasattr(result, "info") else {}

        return observation, result.reward, result.terminated, result.truncated, info

    def render(self) -> Optional[Union[np.ndarray, str]]:
        """
        Render the environment.

        Returns:
            Rendered frame (format depends on render_mode)
        """
        if self.render_mode is None:
            return None

        return self._env.render(self.render_mode)

    def close(self):
        """Clean up resources."""
        pass

    def _format_observation(self, obs) -> Dict[str, np.ndarray]:
        """
        Convert Rust observation to Gymnasium dict format.

        Args:
            obs: Observation from Rust

        Returns:
            Formatted observation dict
        """
        return {
            "arena_grid": np.array(obs.arena_grid, dtype=np.uint8),
            "player_state": np.array(obs.player_state, dtype=np.float32),
            "hand": np.array(obs.hand, dtype=np.int32),
            "tick": np.array([obs.tick], dtype=np.int32),
        }

    def get_action_mask(self) -> np.ndarray:
        """
        Get mask of valid actions for current state.

        Returns:
            Boolean array indicating which actions are valid
        """
        return np.array(self._env.get_action_mask(), dtype=bool)
