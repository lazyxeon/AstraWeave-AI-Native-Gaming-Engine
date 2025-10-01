# BT Patrol Demo

Demonstrates **Behavior Tree AI** with a patrol → detect → chase → attack pattern.

## Purpose

Shows how a simple BT agent transitions between states based on line-of-sight (LOS) detection and proximity to target. Demonstrates deterministic AI behavior with fixed seeding.

## Features

- **Patrol State**: Agent follows waypoints in a square pattern
- **Detect State**: Transition when target enters LOS range (6 tiles)
- **Chase State**: Pursue target while maintaining LOS
- **Attack State**: Deal damage when within melee range (2 tiles)
- **Deterministic**: Fixed seed (42) ensures reproducible behavior

## Build & Run

```powershell
# Standard run
cargo run -p core_loop_bt_demo

# Release mode (faster)
cargo run -p core_loop_bt_demo --release
```

## Controls

| Key | Action |
|-----|--------|
| `Space` | Play/Pause simulation |
| `[` | Slow down time (0.5x) |
| `]` | Speed up time (2x) |
| `R` | Reset to initial seed |
| `Q` | Quit demo |

## HUD Display

```
=== BT PATROL DEMO ===
Mode: BehaviorTree
Tick: 42
Time: 4.20s (scale: 1.0x)
Current Node: Chase
Status: RUNNING

Agent: pos=(12, 10), hp=80
Target: pos=(10, 11), hp=75
Distance: 3, LOS: true
Next Waypoint: (15, 15)

Controls: [Space] Play/Pause | [/] Speed | [R] Reset | [Q] Quit
```

## Determinism

- **Fixed Seed**: 42
- **Stable Iteration**: World state uses BTreeMap for deterministic entity ordering
- **Reproducible**: Running the demo multiple times produces identical behavior traces

### Verification

Run twice and compare tick counts to target defeat:

```powershell
cargo run -p core_loop_bt_demo --release | findstr "DEFEATED"
# Should show same tick count each time
```

## Architecture

### State Machine

```
Patrol ←→ Detect ←→ Chase ←→ Attack
   ↑                            ↓
   └────────── (LOS lost) ──────┘
```

### Transition Logic

- **Patrol → Detect**: Target enters LOS range (distance ≤ 6)
- **Detect → Chase**: Target in LOS and distance > 2
- **Chase → Attack**: Distance ≤ 2
- **Any → Patrol**: LOS lost (distance > 6)

### Implementation Notes

- Uses simple Manhattan distance for LOS checks
- Movement is grid-based (one tile per step)
- Obstacles block movement but not LOS (simplified)
- Attack deals 5 damage per tick

## Feature Flags

This demo does **not** require the `ai-bt` feature flag. It implements a minimal inline BT for demonstration purposes. For production BT systems, use:

```toml
[dependencies]
astraweave-ai = { path = "../../astraweave-ai", features = ["ai-bt"] }
```

## Related

- **Core Loop Dispatcher**: `astraweave-ai/src/core_loop.rs`
- **Integration Tests**: `astraweave-ai/tests/core_loop_rule_integration.rs`
- **GOAP Demo**: `examples/core_loop_goap_demo/`
- **Weaving Demo**: `examples/weaving_pcg_demo/`

## Troubleshooting

**Issue**: Demo exits immediately  
**Solution**: Terminal may be in line-buffered mode. Run in release mode or use a different terminal.

**Issue**: Non-deterministic behavior  
**Solution**: Verify seed is fixed (42) and no external randomness sources used.

**Issue**: Agent gets stuck  
**Solution**: Obstacles may block path. Reset with `R` key to regenerate layout.
