#!/usr/bin/env python3
"""
Basic PPO Training Example

Trains a PPO agent on the Clash Royale environment using Stable-Baselines3.

Usage:
    python examples/train_ppo.py --seed 42 --timesteps 1000000
"""

import argparse
from pathlib import Path

import gymnasium as gym
from stable_baselines3 import PPO
from stable_baselines3.common.callbacks import (
    CheckpointCallback,
    EvalCallback,
)
from stable_baselines3.common.vec_env import DummyVecEnv

import crust_gym


def make_env(seed: int, reward_scheme: str = "shaped"):
    """Create a single environment instance."""

    def _init():
        env = gym.make("ClashRoyale-v0", seed=seed, reward_scheme=reward_scheme)
        return env

    return _init


def train(
    total_timesteps: int = 1_000_000,
    seed: int = 42,
    reward_scheme: str = "shaped",
    log_dir: str = "./logs/ppo_clash",
    save_dir: str = "./models",
):
    """
    Train a PPO agent.

    Args:
        total_timesteps: Total training steps
        seed: Random seed
        reward_scheme: Reward function ("sparse", "shaped", "curriculum")
        log_dir: TensorBoard log directory
        save_dir: Model checkpoint directory
    """
    print("=" * 60)
    print("Clash Royale RL Training - PPO")
    print("=" * 60)
    print(f"Total timesteps: {total_timesteps:,}")
    print(f"Seed: {seed}")
    print(f"Reward scheme: {reward_scheme}")
    print(f"Log directory: {log_dir}")
    print(f"Save directory: {save_dir}")
    print("=" * 60)

    # Create directories
    Path(log_dir).mkdir(parents=True, exist_ok=True)
    Path(save_dir).mkdir(parents=True, exist_ok=True)

    # Create training environment
    env = DummyVecEnv([make_env(seed, reward_scheme)])

    # Create evaluation environment
    eval_env = DummyVecEnv([make_env(seed + 1000, reward_scheme)])

    # Define callbacks
    checkpoint_callback = CheckpointCallback(
        save_freq=50_000,
        save_path=save_dir,
        name_prefix="ppo_clash",
        save_replay_buffer=False,
        save_vecnormalize=False,
    )

    eval_callback = EvalCallback(
        eval_env,
        best_model_save_path=f"{save_dir}/best_model",
        log_path=f"{log_dir}/eval",
        eval_freq=10_000,
        deterministic=True,
        render=False,
        n_eval_episodes=10,
    )

    # Create PPO agent
    model = PPO(
        policy="MultiInputPolicy",
        env=env,
        learning_rate=3e-4,
        n_steps=2048,
        batch_size=64,
        n_epochs=10,
        gamma=0.99,
        gae_lambda=0.95,
        clip_range=0.2,
        ent_coef=0.01,  # Encourage exploration
        vf_coef=0.5,
        max_grad_norm=0.5,
        verbose=1,
        tensorboard_log=log_dir,
        seed=seed,
    )

    print("\n" + "=" * 60)
    print("Starting training...")
    print("=" * 60 + "\n")

    # Train the agent
    model.learn(
        total_timesteps=total_timesteps,
        callback=[checkpoint_callback, eval_callback],
        log_interval=10,
        progress_bar=True,
    )

    # Save final model
    final_path = f"{save_dir}/ppo_clash_final"
    model.save(final_path)
    print(f"\n✓ Final model saved to: {final_path}")

    # Evaluate final performance
    print("\n" + "=" * 60)
    print("Evaluating final model...")
    print("=" * 60)

    obs = eval_env.reset()
    wins = 0
    total_episodes = 10

    for ep in range(total_episodes):
        obs = eval_env.reset()
        done = False
        episode_reward = 0.0

        while not done:
            action, _ = model.predict(obs, deterministic=True)
            obs, reward, done, info = eval_env.step(action)
            episode_reward += reward[0]

        winner = info[0].get("winner", "unknown")
        wins += winner == "player"

        print(f"Episode {ep + 1}/{total_episodes}: Reward={episode_reward:.2f}, Winner={winner}")

    win_rate = wins / total_episodes
    print(f"\n✓ Final win rate: {win_rate:.1%} ({wins}/{total_episodes})")
    print("=" * 60)


def main():
    parser = argparse.ArgumentParser(description="Train PPO agent on Clash Royale")
    parser.add_argument(
        "--timesteps",
        type=int,
        default=1_000_000,
        help="Total training timesteps (default: 1M)",
    )
    parser.add_argument(
        "--seed",
        type=int,
        default=42,
        help="Random seed (default: 42)",
    )
    parser.add_argument(
        "--reward-scheme",
        type=str,
        default="shaped",
        choices=["sparse", "shaped", "curriculum"],
        help="Reward function (default: shaped)",
    )
    parser.add_argument(
        "--log-dir",
        type=str,
        default="./logs/ppo_clash",
        help="TensorBoard log directory",
    )
    parser.add_argument(
        "--save-dir",
        type=str,
        default="./models",
        help="Model save directory",
    )

    args = parser.parse_args()

    train(
        total_timesteps=args.timesteps,
        seed=args.seed,
        reward_scheme=args.reward_scheme,
        log_dir=args.log_dir,
        save_dir=args.save_dir,
    )


if __name__ == "__main__":
    main()
