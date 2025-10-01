# GOAP Craft Demo

Demonstrates **Goal-Oriented Action Planning (GOAP)** with resource gathering, crafting, and consumption.

## Purpose

Shows how a GOAP agent plans and executes a sequence of actions to achieve the goal "has_food = true". Demonstrates dynamic planning based on world state and inventory management.

## Features

- **Dynamic Planning**: Agent generates plans based on current inventory and goal state
- **Resource Management**: Tracks wood, berries, and cooked food
- **Multi-Step Goals**: Gather → Craft → Consume pipeline
- **Deterministic**: Fixed seed (123) ensures reproducible behavior

## Build & Run

```powershell
# Standard run
cargo run -p core_loop_goap_demo

# Release mode (faster)
cargo run -p core_loop_goap_demo --release
```

## Controls

| Key | Action |
|-----|--------|
| `Space` | Play/Pause simulation |
| `[` | Slow down time (0.5x) |
| `]` | Speed up time (2x) |
| `R` | Reset to initial seed |
| `G` | Spawn additional resource node |
| `Q` | Quit demo |

## HUD Display

```
=== GOAP CRAFT DEMO ===
Mode: GOAP (Goal-Oriented Action Planning)
Tick: 42
Time: 4.20s (scale: 1.0x)
Status: RUNNING

Goal: HasFood
Current Action: ChopWood
Plan Length: 3 steps
Next Actions: [ChopWood, GoToBerries, GatherBerries]

Agent: pos=(5, 5)
Hunger: 95/100

Inventory:
  Wood: 1
  Berries: 0
  Cooked Food: 0

World Resources:
  Wood at (5, 5): 9 remaining
  Berries at (15, 5): 10 remaining
  Wood at (5, 15): 10 remaining
  Berries at (15, 15): 10 remaining
```

## Determinism

- **Fixed Seed**: 123
- **Stable Iteration**: Resource nodes placed deterministically
- **Reproducible**: Same sequence of actions each run

### Verification

Run twice and compare final tick counts:

```powershell
cargo run -p core_loop_goap_demo --release | findstr "SATISFIED"
# Should show same tick count each time
```

## GOAP Architecture

### Goal

```
has_food = true
```

### Actions (with preconditions and effects)

| Action | Preconditions | Effects |
|--------|--------------|---------|
| GoToTree | wood_node exists | at_tree = true |
| ChopWood | at_tree = true | wood += 1 |
| GoToBerries | berry_node exists | at_berries = true |
| GatherBerries | at_berries = true | berries += 1 |
| GoToCampfire | - | at_campfire = true |
| CookFood | wood ≥ 2, berries ≥ 2, at_campfire | cooked_food += 1 |
| ConsumeFood | cooked_food > 0 | hunger = 100, has_food = true |

### Planning Algorithm

Simple forward chaining:
1. Check current goal state
2. If resources insufficient, gather wood
3. If wood sufficient but berries insufficient, gather berries
4. If both sufficient, go to campfire and cook
5. If cooked food available, consume

### Execution Flow

```
Start (Goal: HasFood)
  ↓
[GoToTree] → [ChopWood] (x2)
  ↓
[GoToBerries] → [GatherBerries] (x2)
  ↓
[GoToCampfire] → [CookFood]
  ↓
[ConsumeFood]
  ↓
Goal Satisfied ✓
```

## Implementation Notes

- **Simplified GOAP**: This demo uses a minimal GOAP implementation for clarity
- **No A* Planning**: Uses simple forward chaining instead of full A* search
- **Resource Nodes**: Static nodes that deplete when harvested
- **Crafting**: Requires 2 wood + 2 berries at campfire
- **Hunger System**: Decreases over time, restored by consuming food

## Feature Flags

This demo does **not** require the `ai-goap` feature flag. It implements a minimal inline GOAP for demonstration purposes. For production GOAP systems, use:

```toml
[dependencies]
astraweave-ai = { path = "../../astraweave-ai", features = ["ai-goap"] }
```

## Related

- **Core Loop Dispatcher**: `astraweave-ai/src/core_loop.rs`
- **Integration Tests**: `astraweave-ai/tests/core_loop_goap_integration.rs`
- **BT Demo**: `examples/core_loop_bt_demo/`
- **Weaving Demo**: `examples/weaving_pcg_demo/`

## Troubleshooting

**Issue**: Agent doesn't move  
**Solution**: Check that resource nodes haven't been depleted. Press `G` to spawn more or `R` to reset.

**Issue**: Non-deterministic resource spawn with `G` key  
**Solution**: Expected behavior. `G` key uses tick count as additional seed entropy for demonstration purposes.

**Issue**: Agent stuck at campfire  
**Solution**: Inventory may be insufficient. Verify wood ≥ 2 and berries ≥ 2 in HUD.

## Extensions

Possible enhancements for learning:
- Add full A* planning with action costs
- Implement multiple concurrent goals (food, shelter, defense)
- Add tool crafting (axe improves wood gathering rate)
- Implement action failure handling and replanning
