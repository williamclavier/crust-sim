# Python RL Integration Architecture

**Status**: Phase 7 - Planned
**Last Updated**: 2025-11-06
**Dependencies**: Phase 5 (Replay & Serialization)

---

## Overview

This document describes the architecture for exposing the Rust game engine to Python via PyO3, enabling reinforcement learning (RL) research using the Gymnasium API.

**Key Goals:**
- **Performance**: Direct FFI bindings (no network overhead)
- **Compatibility**: Standard Gymnasium interface for all Python RL libraries
- **Determinism**: Maintain reproducible simulations for research
- **Scalability**: Support parallel environment rollouts

---

## Architecture Diagram

```
┌─────────────────────────────────────────────────────────────┐
│  Python Training Script (user code)                         │
│  - Stable-Baselines3 / Ray RLlib / CleanRL                  │
└──────────────────────┬──────────────────────────────────────┘
                       │ Gymnasium API
┌──────────────────────▼──────────────────────────────────────┐
│  crust_gym.ClashRoyaleEnv (Python)                          │
│  - Observation/action space definitions                     │
│  - Reward shaping logic                                     │
│  - State formatting utilities                               │
└──────────────────────┬──────────────────────────────────────┘
                       │ PyO3 FFI
┌──────────────────────▼──────────────────────────────────────┐
│  py-gym crate (Rust + PyO3)                                 │
│  - ClashEnv struct (Python-exposed game wrapper)            │
│  - Observation/action serialization                         │
│  - Episode management                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │ Native Rust calls
┌──────────────────────▼──────────────────────────────────────┐
│  engine crate (Pure Rust)                                   │
│  - GameState, step(), action execution                      │
│  - Deterministic simulation logic                           │
└─────────────────────────────────────────────────────────────┘
```

---

## Workspace Structure

```
crust-sim/
├── py-gym/                      # NEW: Python bindings crate
│   ├── Cargo.toml               # Maturin-compatible config
│   ├── pyproject.toml           # Python package metadata
│   ├── src/
│   │   ├── lib.rs               # PyO3 module definition
│   │   ├── env.rs               # ClashEnv implementation
│   │   ├── observation.rs       # Observation space logic
│   │   ├── action.rs            # Action space logic
│   │   └── reward.rs            # Reward shaping functions
│   ├── python/
│   │   └── crust_gym/           # Python wrapper package
│   │       ├── __init__.py      # Gymnasium registration
│   │       ├── envs.py          # ClashRoyaleEnv class
│   │       └── utils.py         # Helper functions
│   └── examples/
│       ├── train_ppo.py         # PPO training example
│       ├── train_sac.py         # SAC training example
│       ├── self_play.py         # Self-play training
│       └── evaluate.py          # Evaluation script
├── engine/                      # (existing)
├── shared/                      # (existing)
└── ...
```

---

## Observation Space Design

**Type**: `gym.spaces.Dict` (structured state)

### Components

```python
observation_space = gym.spaces.Dict({
    # Arena state (32x18 grid, multi-channel)
    "arena_grid": gym.spaces.Box(
        low=0, high=255,
        shape=(32, 18, 6),  # 6 channels (see below)
        dtype=np.uint8
    ),

    # Player state (both players)
    "player_state": gym.spaces.Box(
        low=0, high=10,
        shape=(2, 3),  # [elixir, tower_hp_left, tower_hp_right]
        dtype=np.float32
    ),

    # Current hand (4 cards per player)
    "hand": gym.spaces.MultiDiscrete([99, 99, 99, 99]),

    # Game metadata
    "tick": gym.spaces.Box(low=0, high=10800, shape=(1,), dtype=np.int32),  # 3 min @ 60 FPS
})
```

### Arena Grid Channels

| Channel | Encoding |
|---------|----------|
| 0 | Unit presence (0=empty, 1-99=card_id) |
| 1 | Unit HP (normalized 0-255) |
| 2 | Unit team (0=neutral, 1=player, 2=opponent) |
| 3 | Unit target presence (0=none, 1=has target) |
| 4 | Projectile presence (0=none, 1-99=projectile type) |
| 5 | Building/structure presence (0=none, 1-99=structure_id) |

**Rationale**:
- Convolutional networks can learn spatial patterns
- Multi-channel encoding preserves game semantics
- Fixed-size grid supports batch processing

---

## Action Space Design

**Type**: `gym.spaces.Tuple` (hybrid discrete + continuous)

### Structure

```python
action_space = gym.spaces.Tuple((
    gym.spaces.Discrete(4),        # Card slot (0-3 in hand)
    gym.spaces.Box(0, 31, shape=(1,), dtype=np.float32),  # X position (continuous)
    gym.spaces.Box(0, 17, shape=(1,), dtype=np.float32),  # Y position (continuous)
))
```

### Mapping to Engine Actions

```rust
pub struct RLAction {
    pub card_index: u8,    // Which card in hand (0-3)
    pub x: f32,            // Continuous x ∈ [0, 31]
    pub y: f32,            // Continuous y ∈ [0, 17]
}

impl RLAction {
    pub fn to_game_action(&self, player: &PlayerState) -> Option<GameAction> {
        let card = player.hand.get(self.card_index as usize)?;

        // Clip to player's side (prevent invalid placements)
        let clamped_y = if player.side == Side::Bottom {
            self.y.clamp(9.0, 17.0)  // Bottom half only
        } else {
            self.y.clamp(0.0, 8.0)   // Top half only
        };

        Some(GameAction::PlayCard {
            card: card.id,
            position: Position {
                x: self.x.clamp(0.0, 31.0),
                y: clamped_y,
            },
        })
    }
}
```

**Rationale**:
- Continuous positioning matches human gameplay granularity
- Easier for policy networks to learn (vs. 32x18=576 discrete positions)
- Action masking prevents invalid moves (e.g., insufficient elixir)

---

## Reward Shaping

**Primary Objective**: Win the game (destroy opponent towers)

### Reward Function

```python
def compute_reward(prev_state, action, next_state):
    reward = 0.0

    # 1. Tower damage (main signal)
    tower_damage_dealt = (
        prev_state.opponent_tower_hp - next_state.opponent_tower_hp
    )
    tower_damage_taken = (
        prev_state.player_tower_hp - next_state.player_tower_hp
    )
    reward += tower_damage_dealt * 0.01   # +0.01 per HP dealt
    reward -= tower_damage_taken * 0.01   # -0.01 per HP taken

    # 2. Tower destruction (sparse bonus)
    if next_state.opponent_towers_destroyed > prev_state.opponent_towers_destroyed:
        reward += 5.0  # Large bonus for tower kill
    if next_state.player_towers_destroyed > prev_state.player_towers_destroyed:
        reward -= 5.0  # Large penalty for tower loss

    # 3. Elixir efficiency (encourage good trades)
    elixir_advantage = (
        next_state.player_elixir - next_state.opponent_elixir
    )
    reward += elixir_advantage * 0.001  # Small shaping term

    # 4. Victory condition
    if next_state.game_over:
        if next_state.winner == Player.Self:
            reward += 100.0   # Win bonus
        elif next_state.winner == Player.Opponent:
            reward -= 100.0   # Loss penalty
        else:
            reward += 0.0     # Draw (no bonus)

    # 5. Invalid action penalty
    if not action.is_valid:
        reward -= 1.0  # Discourage invalid moves

    return reward
```

### Alternative Reward Schemes (Configurable)

| Scheme | Description | Use Case |
|--------|-------------|----------|
| `sparse` | Only tower kills/game outcome | Advanced agents |
| `shaped` | Dense + tower damage + elixir | Early training |
| `curriculum` | Start shaped, anneal to sparse | Stable learning |

---

## Training Schemes

### 1. Self-Play (Primary)

```python
# Pseudocode
def self_play_training():
    agent = PPO(policy="MultiInputPolicy", env=ClashRoyaleEnv())
    checkpoint_opponents = []

    for iteration in range(num_iterations):
        # Train against latest checkpoint
        opponent = checkpoint_opponents[-1] if checkpoint_opponents else random_bot
        env.set_opponent(opponent)

        agent.learn(total_timesteps=100_000)

        # Save checkpoint every N iterations
        if iteration % checkpoint_interval == 0:
            checkpoint_opponents.append(agent.copy())

        # Evaluate against checkpoint pool
        if iteration % eval_interval == 0:
            elo_ratings = evaluate_vs_pool(agent, checkpoint_opponents)
            log_metrics(elo_ratings)
```

**Advantages**:
- No need for labeled data
- Naturally handles strategy evolution
- Proven in AlphaGo, OpenAI Five, AlphaStar

### 2. Imitation Learning (Optional)

```python
# Train on replays from strong players
def imitation_learning():
    dataset = load_replays("data/expert_replays/")
    agent = BC(policy="MultiInputPolicy")  # Behavioral Cloning
    agent.learn(dataset)

    # Fine-tune with RL
    agent = PPO.load("bc_checkpoint")
    agent.learn(total_timesteps=1_000_000)
```

**Use Case**: Bootstrapping early policy

---

## Algorithm Recommendations

### Phase 1: Proximal Policy Optimization (PPO)

**Rationale**:
- Sample-efficient for on-policy learning
- Robust to hyperparameter tuning
- Works well with self-play
- Proven in multi-agent games (Dota 2, StarCraft)

```python
from stable_baselines3 import PPO

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
    verbose=1,
    tensorboard_log="./logs/ppo",
)
```

### Phase 2: Soft Actor-Critic (SAC)

**Rationale**:
- Off-policy (more sample-efficient)
- Better for continuous action spaces
- Automatic entropy tuning

**When to switch**: After initial PPO convergence (>60% win rate vs. scripted bot)

### Phase 3: MuZero (Aspirational)

**Rationale**:
- Model-based RL (learns game dynamics)
- Sample-efficient
- State-of-the-art for board games

**Requirements**: Significant compute (GPUs, distributed training)

---

## Implementation Roadmap

### Milestone 1: Core Bindings (Week 1)

**Tasks**:
1. Set up `py-gym` crate with Maturin
2. Implement basic `ClashEnv` struct with PyO3
3. Expose `reset()` and `step()` to Python
4. Add unit tests for Rust-Python data serialization

**Deliverable**: `import crust_gym; env = crust_gym.ClashEnv()` works

### Milestone 2: Gymnasium Wrapper (Week 1-2)

**Tasks**:
1. Implement `ClashRoyaleEnv(gym.Env)` in Python
2. Define observation/action spaces
3. Add action validation and masking
4. Register environment with Gymnasium

**Deliverable**: `gym.make("ClashRoyale-v0")` works

### Milestone 3: Reward Shaping (Week 2)

**Tasks**:
1. Implement reward function in Rust
2. Add configurable reward schemes
3. Test reward signal with random policy
4. Tune reward coefficients

**Deliverable**: Agent receives meaningful reward signal

### Milestone 4: Training Infrastructure (Week 2-3)

**Tasks**:
1. Create PPO training script
2. Add TensorBoard logging
3. Implement checkpoint saving/loading
4. Create evaluation script (ELO rating)

**Deliverable**: `python examples/train_ppo.py` trains a functional agent

### Milestone 5: Self-Play (Week 3-4)

**Tasks**:
1. Implement opponent management system
2. Add checkpoint pool
3. Create self-play training loop
4. Benchmark performance

**Deliverable**: Agent improves via self-play to >70% win rate vs. scripted bot

### Milestone 6: Optimization (Week 4+)

**Tasks**:
1. Add vectorized environments (parallel rollouts)
2. Optimize observation encoding
3. Profile and optimize Python-Rust FFI calls
4. Add GPU support for neural network inference

**Deliverable**: 1000+ steps/second training throughput

---

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| **Steps/sec (single env)** | 1,000+ | Faster than real-time (60 FPS game) |
| **Steps/sec (16 parallel envs)** | 10,000+ | Efficient vectorized rollouts |
| **FFI overhead** | <5% | Minimal Python-Rust serialization cost |
| **Memory per env** | <100 MB | Support 100+ parallel envs on single GPU |
| **Training time to competent bot** | <24 hours | Practical research iteration |

---

## API Reference

### Rust (PyO3)

```rust
#[pyclass]
pub struct ClashEnv {
    game_state: GameState,
    rng_seed: u64,
    tick: u32,
    reward_scheme: RewardScheme,
}

#[pymethods]
impl ClashEnv {
    #[new]
    fn new(seed: Option<u64>, reward_scheme: Option<String>) -> PyResult<Self>;

    fn reset(&mut self) -> PyResult<Observation>;

    fn step(&mut self, action: Action) -> PyResult<StepResult>;

    fn render(&self, mode: &str) -> PyResult<String>;

    fn seed(&mut self, seed: u64);

    fn get_action_mask(&self) -> PyResult<Vec<bool>>;
}

#[pyclass]
pub struct Observation {
    #[pyo3(get)]
    pub arena_grid: Vec<Vec<Vec<u8>>>,  // 32x18x6

    #[pyo3(get)]
    pub player_state: Vec<Vec<f32>>,    // 2x3

    #[pyo3(get)]
    pub hand: Vec<u8>,                  // 4 card IDs

    #[pyo3(get)]
    pub tick: u32,
}

#[pyclass]
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
    pub info: HashMap<String, serde_json::Value>,
}
```

### Python (Gymnasium)

```python
class ClashRoyaleEnv(gym.Env):
    """Clash Royale RL environment."""

    metadata = {"render_modes": ["human", "rgb_array", "ansi"]}

    def __init__(
        self,
        seed: Optional[int] = None,
        reward_scheme: str = "shaped",
        opponent: Optional[Policy] = None,
        render_mode: Optional[str] = None,
    ):
        """Initialize environment."""

    def reset(
        self,
        seed: Optional[int] = None,
        options: Optional[dict] = None
    ) -> Tuple[ObsType, dict]:
        """Reset environment to initial state."""

    def step(self, action: ActType) -> Tuple[ObsType, float, bool, bool, dict]:
        """Execute action and return next state."""

    def render(self) -> Optional[Union[RenderFrame, List[RenderFrame]]]:
        """Render environment state."""

    def close(self):
        """Clean up resources."""
```

---

## Dependencies

### Rust Crates

```toml
[dependencies]
pyo3 = { version = "0.22", features = ["extension-module"] }
numpy = "0.22"  # NumPy array support
serde_json = "1.0"
engine = { path = "../engine" }
shared = { path = "../shared" }

[build-dependencies]
maturin = "1.7"
```

### Python Packages

```toml
[project]
name = "crust-gym"
version = "0.1.0"
requires-python = ">=3.8"
dependencies = [
    "gymnasium>=0.29.0",
    "numpy>=1.24.0",
    "stable-baselines3>=2.0.0",
    "tensorboard>=2.15.0",
]

[project.optional-dependencies]
dev = [
    "pytest>=7.4.0",
    "black>=23.0.0",
    "mypy>=1.5.0",
]
advanced = [
    "ray[rllib]>=2.8.0",  # For distributed training
    "torch>=2.0.0",       # For custom policies
]
```

---

## Testing Strategy

### Unit Tests (Rust)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_observation_serialization() {
        let obs = Observation::from_game_state(&game_state);
        // Assert shapes and value ranges
    }

    #[test]
    fn test_action_validation() {
        let action = Action { card_index: 0, x: 15.5, y: 10.0 };
        assert!(action.is_valid(&game_state));
    }

    #[test]
    fn test_reward_computation() {
        let reward = compute_reward(&prev_state, &next_state, &action);
        assert!(reward > 0.0);  // Positive for tower damage
    }
}
```

### Integration Tests (Python)

```python
def test_gymnasium_api_compliance():
    """Test environment follows Gymnasium API."""
    env = gym.make("ClashRoyale-v0")
    check_env(env)  # Gymnasium's built-in validator

def test_determinism():
    """Test same seed produces same trajectory."""
    env1 = gym.make("ClashRoyale-v0", seed=42)
    env2 = gym.make("ClashRoyale-v0", seed=42)

    for _ in range(100):
        action = env1.action_space.sample()
        obs1, *_ = env1.step(action)
        obs2, *_ = env2.step(action)
        assert np.allclose(obs1["arena_grid"], obs2["arena_grid"])

def test_training_convergence():
    """Test agent can learn basic strategy."""
    env = gym.make("ClashRoyale-v0")
    model = PPO("MultiInputPolicy", env)
    model.learn(total_timesteps=50_000)

    # Evaluate
    wins = 0
    for _ in range(10):
        obs, _ = env.reset()
        done = False
        while not done:
            action, _ = model.predict(obs)
            obs, _, terminated, truncated, info = env.step(action)
            done = terminated or truncated
        wins += info.get("winner") == "player"

    assert wins >= 3  # Better than random (expect ~30% win rate)
```

---

## Example Usage

### Basic Training

```python
import gymnasium as gym
import crust_gym
from stable_baselines3 import PPO

# Create environment
env = gym.make("ClashRoyale-v0", seed=42)

# Train agent
model = PPO(
    policy="MultiInputPolicy",
    env=env,
    verbose=1,
    tensorboard_log="./logs/ppo_clash",
)
model.learn(total_timesteps=1_000_000)

# Save model
model.save("models/ppo_clash_1m")

# Evaluate
obs, info = env.reset()
for _ in range(1000):
    action, _ = model.predict(obs, deterministic=True)
    obs, reward, terminated, truncated, info = env.step(action)
    if terminated or truncated:
        print(f"Game over. Winner: {info['winner']}")
        break
```

### Self-Play Training

```python
from crust_gym import SelfPlayTrainer

trainer = SelfPlayTrainer(
    env_id="ClashRoyale-v0",
    algorithm="PPO",
    checkpoint_interval=50_000,
    opponent_pool_size=10,
)

trainer.train(
    total_timesteps=10_000_000,
    log_dir="./logs/self_play",
)

# Best agent saved to trainer.best_model_path
```

---

## Future Extensions

### 1. Multi-Agent RL
- Separate policies for each player
- Cooperative modes (2v2)
- Communication channels

### 2. Curriculum Learning
- Start with simplified scenarios (1 card type)
- Gradually increase complexity
- Automatic difficulty adjustment

### 3. Imitation Learning
- Record human gameplay
- Pre-train with behavioral cloning
- Fine-tune with RL

### 4. Distributed Training
- Ray RLlib integration
- Multi-GPU support
- Cluster deployment

### 5. Explainability
- Attention visualizations
- Strategy extraction
- Replay analysis tools

---

## References

- [Gymnasium Documentation](https://gymnasium.farama.org/)
- [Stable-Baselines3](https://stable-baselines3.readthedocs.io/)
- [PyO3 User Guide](https://pyo3.rs/)
- [Maturin Documentation](https://www.maturin.rs/)
- [OpenAI Five](https://openai.com/research/openai-five)
- [AlphaStar](https://www.deepmind.com/blog/alphastar-mastering-the-real-time-strategy-game-starcraft-ii)

---

## Contact

For questions about the RL integration, see:
- Architecture discussions: `docs/PYTHON_RL_INTEGRATION.md` (this file)
- Implementation progress: `ROADMAP.md` Phase 7
- API examples: `py-gym/examples/`
