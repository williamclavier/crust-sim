# Clash Royale Engine Modernization Roadmap (Rust)

## Overview
This document defines the full roadmap to rebuild the legacy JavaScript Clash Royale engine into a modular, deterministic, and high-performance Rust simulation engine.

The final product should support:
- AI-ready tick-based logic
- Human-playable browser UI (WASM)
- Replay serialization and playback
- External WebSocket API for AI training or bots
- Deterministic simulation with reproducible seeds

---

## Repository Scaffold

clash_royale_rust/
├─ engine/ # Pure Rust logic crate
│ ├─ src/
│ │ ├─ entities/ # structs for troops, towers, spells
│ │ ├─ systems/ # movement, attacks, collisions, elixir
│ │ ├─ arena.rs # field layout + geometry
│ │ ├─ state.rs # GameState and serialization
│ │ ├─ action.rs # PlayCard, Spawn, etc.
│ │ └─ lib.rs
│ ├─ Cargo.toml
│
├─ viewer/ # WASM/Bevy visualizer (later phase)
│ ├─ src/
│ ├─ index.html
│ ├─ Cargo.toml
│
├─ bridge/ # WebSocket or gRPC control layer
│ ├─ src/
│ ├─ Cargo.toml
│
├─ shared/ # shared data structures
│ ├─ src/lib.rs
│ ├─ Cargo.toml
│
├─ scripts/ # JS data extraction and tools
└─ Cargo.toml


---

## Phase 1: Legacy Analysis & Data Extraction
**Goal:** Extract game logic and card data from the old JS engine.

**Tasks**
1. Parse the old `index.js` and extract:
   - Card types, stats, and behaviors
   - Arena geometry
   - Tick/update rules and time delta
2. Export data as JSON (`scripts/cards.json`)
3. Define a schema for card definitions.

**Deliverables**
- `scripts/extract_cards.js` generates `cards.json`
- Markdown summary of extracted rules

**Success Criteria**
- `cards.json` accurately reflects legacy card stats

---

## Phase 2: Core Engine Skeleton
**Goal:** Build the Rust foundation for the simulation.

**Tasks**
1. Create the `engine` crate with:
   ```rust
   pub struct GameState;
   pub struct Entity;
   pub fn step(&mut self, dt: f32);

Add deterministic RNG (e.g., oorandom)

Implement tick loop that updates entities

Deliverables

CLI simulation printing entity states

Deterministic output using fixed seed

Success Criteria

Same seed produces identical tick logs

Phase 3: Cards and Elixir System

Goal: Implement basic player loop, deck, and elixir regeneration.

Tasks

Add PlayerState with hand, deck, elixir

Define Card struct with metadata and costs

Implement card deployment and elixir usage

Add 5+ test cards (Knight, Archers, Giant, Fireball, Arrows)

Deliverables

CLI sim with two scripted players

JSON output showing card plays and elixir consumption

Success Criteria

Deterministic duel output consistent across runs

Phase 4: Collision & Targeting

Goal: Enable combat and targeting systems.

Tasks

Add collision, attack, and death systems

Use circular or grid-based collision detection

Implement targeting logic (nearest, tower priority)

Add HP, damage, and projectile systems

Deliverables

CLI output includes attacks and HP updates

Unit test verifying tower kill within expected ticks

Success Criteria

Unit deaths and tower destruction behave as expected

Phase 5: Replay & Serialization

Goal: Make every match fully replayable.

Tasks

Add serde serialization to GameState and Action

Record all actions + states per tick

Implement deterministic replay loading from JSON

Deliverables

cargo run -- replay replays/match1.json reproduces match

Recorded JSON logs match deterministic checksum

Success Criteria

Replayed matches produce identical outputs

Phase 6: Browser/WASM Viewer

Goal: Add an interactive or visual UI similar to the old JS version.

Tasks

Add viewer crate using wasm-bindgen + Trunk

Render arena + entities on an HTML5 canvas

Add mouse/tap input for manual play

Support replay visualization

Deliverables

trunk serve runs playable browser demo

Visual playback for saved replays

Success Criteria

Browser viewer renders and updates at 60 FPS

Phase 7: AI & Python RL Integration

Goal: Expose engine to Python ML ecosystem via PyO3 bindings and Gymnasium API for reinforcement learning research.

Architecture

Create py-gym crate with Maturin + PyO3 bindings

Implement Gymnasium-compatible environment

Design observation/action spaces for RL

Add configurable reward shaping

Support self-play training with checkpoint opponents

Tasks

Milestone 1: Core PyO3 Bindings
- Set up py-gym crate with Maturin build system
- Implement ClashEnv struct exposed to Python
- Add reset() and step() methods with proper serialization
- Unit tests for Rust-Python data conversion

Milestone 2: Gymnasium Wrapper
- Create ClashRoyaleEnv(gym.Env) Python class
- Define observation space: Dict space with arena grid (32x18x6), player state, hand
- Define action space: Tuple(Discrete(4), Box(0-31), Box(0-17)) for hybrid card+position
- Implement action validation and masking
- Register with Gymnasium: gym.make("ClashRoyale-v0")

Milestone 3: Reward Shaping
- Implement configurable reward function:
  - Tower damage: ±0.01 per HP
  - Tower kill bonus: ±5.0
  - Victory: ±100.0
  - Elixir efficiency: small shaping term
- Add reward schemes: sparse, shaped, curriculum
- Test reward signal with random policy

Milestone 4: Training Infrastructure
- Create examples/train_ppo.py using Stable-Baselines3
- Add TensorBoard logging for metrics
- Implement checkpoint saving/loading
- Create evaluation script with ELO rating system
- Verify deterministic training (same seed = same results)

Milestone 5: Self-Play System
- Implement opponent pool management
- Add periodic checkpoint saving during training
- Create self-play training loop (train vs. past versions)
- Benchmark: agent reaches >70% win rate vs. scripted bot

Milestone 6: Optimization
- Add vectorized environments (16+ parallel rollouts)
- Profile and optimize observation encoding
- Reduce FFI overhead to <5%
- Target: 1,000+ steps/sec (single env), 10,000+ (16 parallel)

Deliverables

pip install crust-gym works from maturin build

python examples/train_ppo.py trains a functional agent

Agent improves via self-play to superhuman performance

Comprehensive documentation in docs/PYTHON_RL_INTEGRATION.md

Success Criteria

Environment passes Gymnasium API compliance tests (check_env)

Same seed produces deterministic trajectories

Trained PPO agent beats random policy within 50k timesteps

Self-play training converges to strategic behavior

Technical Specifications

Observation Space: Structured Dict with multi-channel arena grid, player state vectors, card hand

Action Space: Hybrid (Discrete card selection + Continuous x,y positioning)

Training Scheme: Self-play with periodic checkpoint opponents (proven in AlphaGo/OpenAI Five)

Algorithm: Start with PPO (robust, proven), migrate to SAC (sample-efficient) or MuZero (model-based)

Reward Function: Shaped dense rewards (tower damage + elixir efficiency) with configurable schemes

Performance: 1000+ steps/sec single env, 10000+ vectorized, <100 MB memory per env

Dependencies

Python: gymnasium>=0.29, stable-baselines3>=2.0, numpy, tensorboard

Rust: pyo3 = "0.22", numpy = "0.22", maturin = "1.7"

Build: Maturin for creating Python wheels from Rust+PyO3 code

Future Extensions

Multi-agent RL (separate policies, 2v2 modes)

Imitation learning from human replays

Distributed training (Ray RLlib, multi-GPU)

Curriculum learning (start simple, increase complexity)

Explainability tools (attention maps, strategy extraction)

Phase 8: Optimization & Extensibility

Goal: Improve performance and modularity.

Tasks

Add ECS or parallel update (e.g., hecs or rayon)

Configurable JSON card/arena definitions

Abstract systems into traits (UpdateSystem, RenderSystem)

Document architecture

Deliverables

Benchmarks showing 1000+ matches/sec

ENGINE_ARCHITECTURE.md describing modules

Success Criteria

Engine modular, deterministic, and performant

Best Practices

Always use seeded RNG for determinism

Fixed timestep simulation (e.g., 0.016s)

Validate correctness via deterministic replays

Keep JS engine as behavioral oracle

Incrementally migrate systems, test at each phase

Log state transitions for debugging

End State

When complete, this Rust project should provide:

A deterministic, reproducible Clash Royale-style simulator

Modular systems for expansion

Browser-based UI for visualization

External control via WebSocket API

Replay serialization for training and analysis

Clear architecture documentation for future contributors