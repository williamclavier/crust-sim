# Crust Gym - Python RL Bindings

Python reinforcement learning environment for the Clash Royale simulator, built with PyO3 and compatible with Gymnasium.

## Features

- **Gymnasium API**: Standard interface compatible with all Python RL libraries
- **High Performance**: Direct Rust FFI (no network overhead)
- **Deterministic**: Reproducible simulations with seeded RNG
- **Configurable Rewards**: Multiple reward shaping schemes
- **Self-Play Ready**: Built-in support for training against past checkpoints

## Installation

### From Source (Development)

```bash
# Install maturin (Rust-Python build tool)
pip install maturin

# Build and install in development mode
cd py-gym
maturin develop --release

# Or build a wheel
maturin build --release
pip install target/wheels/crust_gym-*.whl
```

### From PyPI (Future)

```bash
pip install crust-gym
```

## Quick Start

### Basic Usage

```python
import gymnasium as gym
import crust_gym

# Create environment
env = gym.make("ClashRoyale-v0", seed=42)

# Run random policy
obs, info = env.reset()
for _ in range(1000):
    action = env.action_space.sample()
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        print(f"Episode finished. Winner: {info.get('winner', 'unknown')}")
        break
```

### Training with Stable-Baselines3

```python
from stable_baselines3 import PPO
import gymnasium as gym
import crust_gym

# Create environment
env = gym.make("ClashRoyale-v0", seed=42, reward_scheme="shaped")

# Train PPO agent
model = PPO(
    policy="MultiInputPolicy",
    env=env,
    verbose=1,
    tensorboard_log="./logs/ppo_clash",
)
model.learn(total_timesteps=1_000_000)

# Save and evaluate
model.save("models/ppo_clash_1m")
```

### Self-Play Training

```python
from crust_gym.utils import SelfPlayTrainer

trainer = SelfPlayTrainer(
    env_id="ClashRoyale-v0",
    algorithm="PPO",
    checkpoint_interval=50_000,
)

trainer.train(total_timesteps=10_000_000, log_dir="./logs/self_play")
```

## Observation Space

The environment returns a structured observation:

```python
{
    "arena_grid": np.ndarray,  # Shape: (32, 18, 6), dtype: uint8
    "player_state": np.ndarray,  # Shape: (2, 3), dtype: float32
    "hand": np.ndarray,  # Shape: (4,), dtype: int32
    "tick": np.ndarray,  # Shape: (1,), dtype: int32
}
```

### Arena Grid Channels

| Channel | Description |
|---------|-------------|
| 0 | Unit presence (0=empty, 1-99=card_id) |
| 1 | Unit HP (normalized 0-255) |
| 2 | Unit team (0=neutral, 1=player, 2=opponent) |
| 3 | Unit has target (0=no, 1=yes) |
| 4 | Projectile presence |
| 5 | Building/structure presence |

## Action Space

Hybrid discrete + continuous action space:

```python
Tuple((
    Discrete(4),        # Card index (which card to play from hand)
    Box(0, 31, (1,)),   # X position (continuous)
    Box(0, 17, (1,)),   # Y position (continuous)
))
```

## Reward Schemes

Configure via `reward_scheme` parameter:

| Scheme | Description | Use Case |
|--------|-------------|----------|
| `sparse` | Only tower kills and game outcome | Advanced agents |
| `shaped` | Dense rewards (tower damage + elixir) | Early training |
| `curriculum` | Starts shaped, anneals to sparse | Stable learning |

## Examples

See `examples/` directory for complete training scripts:

- `train_ppo.py` - Basic PPO training
- `train_sac.py` - SAC training (continuous actions)
- `self_play.py` - Self-play training loop
- `evaluate.py` - Agent evaluation with ELO ratings

## Performance

Expected performance on modern hardware:

| Metric | Target |
|--------|--------|
| Single env | 1,000+ steps/sec |
| 16 parallel envs | 10,000+ steps/sec |
| FFI overhead | <5% |
| Memory per env | <100 MB |

## Development

### Building

```bash
# Install development dependencies
pip install -e ".[dev]"

# Build Rust code
maturin develop

# Run tests
pytest tests/
cargo test

# Type checking
mypy crust_gym/

# Code formatting
black crust_gym/
cargo fmt
```

### Project Structure

```
py-gym/
├── src/              # Rust PyO3 bindings
│   ├── lib.rs        # Module definition
│   ├── env.rs        # ClashEnv implementation
│   ├── observation.rs
│   ├── action.rs
│   └── reward.rs
├── python/           # Python wrapper
│   └── crust_gym/
│       ├── __init__.py
│       └── envs.py
├── examples/         # Training scripts
├── tests/            # Python tests
└── Cargo.toml
```

## Documentation

See [`docs/PYTHON_RL_INTEGRATION.md`](../docs/PYTHON_RL_INTEGRATION.md) for detailed architecture documentation.

## Contributing

Contributions welcome! Please ensure:
- Code passes tests (`cargo test`, `pytest`)
- Code is formatted (`cargo fmt`, `black`)
- New features have tests

## License

MIT License - see LICENSE file for details.

## References

- [Gymnasium Documentation](https://gymnasium.farama.org/)
- [Stable-Baselines3](https://stable-baselines3.readthedocs.io/)
- [PyO3 Guide](https://pyo3.rs/)
- [Maturin Documentation](https://www.maturin.rs/)
