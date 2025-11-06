"""
Crust Gym - Gymnasium RL Environment for Clash Royale Simulator

This package provides a Gymnasium-compatible reinforcement learning environment
for training agents to play Clash Royale.

Example usage:
    >>> import gymnasium as gym
    >>> import crust_gym
    >>>
    >>> env = gym.make("ClashRoyale-v0", seed=42)
    >>> obs, info = env.reset()
    >>>
    >>> for _ in range(1000):
    >>>     action = env.action_space.sample()
    >>>     obs, reward, terminated, truncated, info = env.step(action)
    >>>     if terminated or truncated:
    >>>         break
"""

from gymnasium.envs.registration import register

# Import native Rust module
from crust_gym._native import ClashEnv, Observation, Action, StepResult, __version__

# Import Python wrapper
from crust_gym.envs import ClashRoyaleEnv

# Register with Gymnasium
register(
    id="ClashRoyale-v0",
    entry_point="crust_gym.envs:ClashRoyaleEnv",
    max_episode_steps=10800,  # 3 minutes @ 60 FPS
    reward_threshold=100.0,   # Win the game
)

__all__ = [
    "ClashRoyaleEnv",
    "ClashEnv",
    "Observation",
    "Action",
    "StepResult",
    "__version__",
]
