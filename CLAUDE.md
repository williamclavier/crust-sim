# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**crust-sim** is a Rust-based rewrite of a legacy JavaScript Clash Royale engine (circa 2020). The goal is to create a modular, deterministic, high-performance simulation engine with:

- **Configuration-driven design**: No hardcoded game values; all mechanics, cards, and stats defined in versioned config files
- **Deterministic simulation**: Reproducible matches with seeded RNG
- **AI-ready tick-based logic**: Fixed timestep updates for training and automation
- **Multiple interfaces**: CLI simulation, WASM browser UI, and WebSocket API
- **Replay system**: Full serialization and playback of matches
- **Patch versioning**: Support for different game versions via config/patches/

## Repository Context

### Working Directories
- `/Users/will/Documents/Projects/crust-sim/` - Main Rust project (this repository)
- `/Users/will/Documents/Projects/clash-royale-engine/` - Legacy JavaScript engine reference

### Legacy Engine
The original engine is a monolithic Processing.js file (`code.txt`, 1.1MB) with:
- 99+ cards with hardcoded stats
- 32x18 tile-based arena
- 60 FPS game loop
- Tightly coupled physics, combat, and rendering systems

**Key reference documents** (in this repo):
- `ROADMAP.md` - 8-phase migration plan
- `CLASH_ROYALE_ENGINE_ANALYSIS.md` - Detailed technical analysis (630 lines)
- `CLASH_ROYALE_ENGINE_EXTRACTION_GUIDE.md` - Extraction templates and checklists
- `EXPLORATION_SUMMARY.txt` - Quick reference with line numbers

## Architecture (Planned)

The project will follow a multi-crate workspace structure:

```
crust-sim/
├── engine/           # Pure Rust logic (no I/O, no rendering)
│   ├── entities/     # Troops, towers, spells, projectiles
│   ├── systems/      # Movement, combat, collisions, elixir
│   ├── arena.rs      # Field geometry and tile system
│   ├── state.rs      # GameState serialization
│   └── action.rs     # Player actions (PlayCard, Spawn)
├── py-gym/           # Python RL bindings (PyO3 + Gymnasium)
│   ├── src/          # Rust PyO3 bindings
│   ├── python/       # Python wrapper package
│   └── examples/     # Training scripts
├── viewer/           # WASM/Bevy browser visualizer
├── bridge/           # WebSocket/gRPC control layer
├── shared/           # Shared data structures
└── config/           # Configuration files
    ├── patches/      # Version-specific game data
    │   ├── v2020_06/ # Legacy baseline
    │   └── v2025_10/ # Current patch
    └── schemas/      # JSON validation schemas
```

## Development Workflow

### Current Phase: Pre-Phase 1 (Project Setup)

The Rust project structure does not exist yet. Before starting Phase 1:

1. **Initialize Rust workspace**: Create `Cargo.toml` with workspace members
2. **Set up engine crate**: Basic skeleton with `GameState`, `Entity`, and `step()` function
3. **Add deterministic RNG**: Use `oorandom` or similar for reproducible simulation

### Phase 1: Legacy Analysis & Data Extraction

**Goal**: Extract game data from the JavaScript engine into structured JSON.

**Key tasks**:
1. Parse `/Users/will/Documents/Projects/clash-royale-engine/code.txt`
2. Extract card definitions, stats, and behaviors → `config/patches/v2020_06/cards.json`
3. Extract arena geometry and tile types → `config/patches/v2020_06/arena.json`
4. Document tick rules and time delta → `config/patches/v2020_06/mechanics.json`

**Reference**: Lines of interest in `code.txt` are documented in `EXPLORATION_SUMMARY.txt`

### Commands (Future - Not Yet Implemented)

Once the Rust project is set up:

```bash
# Build all crates
cargo build --release

# Run CLI simulation
cargo run --bin engine-cli

# Run tests
cargo test

# Run specific test
cargo test --package engine --test test_name

# Run with deterministic seed
cargo run --bin engine-cli -- --seed 12345

# Replay a match
cargo run --bin engine-cli -- replay replays/match1.json

# Start WASM viewer (later phases)
cd viewer && trunk serve

# Start WebSocket bridge (later phases)
cargo run --bin bridge -- --port 8080
```

## Critical Design Principles

### 1. Determinism First
- **Always use seeded RNG**: Every random event must use the engine's RNG state
- **Fixed timestep**: Use consistent delta time (e.g., 0.016s) regardless of frame rate
- **No system time dependencies**: Never use `std::time::SystemTime` for game logic
- **Replay validation**: Every match must be reproducible from action log + seed

### 2. Configuration Over Code
- **Never hardcode game values**: All stats, mechanics, and constants go in JSON configs
- **Patch-based versioning**: Organize configs by game version (e.g., `v2020_06/`, `v2025_10/`)
- **Schema validation**: Use JSON schemas to validate config files at load time
- **Hot-reloadable**: Design systems to reload configs without recompilation

### 3. Separation of Concerns
- **Pure engine logic**: `engine/` crate has zero dependencies on I/O, rendering, or networking
- **No coupling**: Combat system shouldn't know about rendering, networking shouldn't know about physics
- **Trait-based systems**: Use `UpdateSystem`, `RenderSystem` traits for modularity

### 4. Legacy Engine as Oracle
- Keep the JavaScript engine running as a behavioral reference
- When in doubt about mechanics, test against the legacy implementation
- Document any intentional deviations from legacy behavior

## Migration Strategy

Follow the 8-phase roadmap in `ROADMAP.md`:

**Phase 1-2**: Foundation (data extraction + core skeleton)
**Phase 3-4**: Gameplay (cards, elixir, combat)
**Phase 5**: Replay system
**Phase 6**: Browser UI
**Phase 7**: Python RL Integration (PyO3 + Gymnasium)
**Phase 8**: Optimization

Each phase builds incrementally. Validate correctness at every step using deterministic replays.

## Python RL Integration

The project includes Python bindings for reinforcement learning research:

**Key Features:**
- **Gymnasium API**: Standard interface compatible with Stable-Baselines3, Ray RLlib, etc.
- **PyO3 Bindings**: Direct Rust-Python FFI (1000+ steps/sec, no network overhead)
- **Hybrid Action Space**: Discrete card selection + continuous (x,y) positioning
- **Self-Play Ready**: Built-in support for training against past checkpoints
- **Configurable Rewards**: Sparse, shaped, and curriculum learning schemes

**Quick Start:**
```python
import gymnasium as gym
import crust_gym

env = gym.make("ClashRoyale-v0", seed=42)
obs, info = env.reset()

from stable_baselines3 import PPO
model = PPO("MultiInputPolicy", env)
model.learn(total_timesteps=1_000_000)
```

See `docs/PYTHON_RL_INTEGRATION.md` for detailed architecture documentation.

## Testing Strategy

- **Unit tests**: Test individual systems in isolation (collision, targeting, damage)
- **Integration tests**: Test full matches with scripted actions
- **Deterministic tests**: Assert same seed produces identical outputs
- **Replay tests**: Verify recorded matches replay exactly
- **Legacy parity tests**: Compare outputs against JavaScript engine where possible

## Common Gotchas

### From Legacy Engine Analysis

1. **Collision detection**: Legacy uses 3 O(n²) passes - optimize this in Rust with spatial partitioning
2. **Effect system**: 20+ status effects (slow, stun, spawner, split, etc.) - needs careful state management
3. **Tower positioning**: Fixed positions in legacy - make configurable in arena.json
4. **Card versions**: Legacy has duplicate card definitions - consolidate in extraction
5. **Processing.js quirks**: Legacy uses Processing.js coordinate system and drawing - abstract this away

### Rust-Specific

1. **Floating point determinism**: Use consistent compiler flags and avoid platform-specific math
2. **Serialization**: Use `serde` with deterministic field ordering
3. **ECS choice**: Consider `hecs` or `bevy_ecs` for entity management (Phase 8)
4. **WASM target**: Ensure `engine/` crate compiles to `wasm32-unknown-unknown`

## Resources

- Legacy engine location: `/Users/will/Documents/Projects/clash-royale-engine/`
- Main game logic: `code.txt` (lines 2658-3024 for core loop and combat)
- Card database: `code.txt` (declarative arrays, extract with parsing script)
- Analysis docs: `CLASH_ROYALE_ENGINE_ANALYSIS.md` and `EXTRACTION_GUIDE.md`
