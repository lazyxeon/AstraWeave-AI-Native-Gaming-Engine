# Weaving + PCG Demo

Demonstrates **Weaving System** integration with **Procedural Content Generation (PCG)** for dynamic difficulty and content adaptation.

## Purpose

Shows how the Weaving system detects patterns in gameplay (tension, health, combat streaks), proposes intents (spawn aid, reduce tension, scale difficulty), and uses PCG to generate appropriate encounters dynamically.

## Features

- **Pattern Detection**: Analyzes player health, tension score, and combat streaks
- **Signal Generation**: Emits weaving signals based on detected patterns
- **Intent Proposal**: AI Director proposes content modifications
- **Adjudication**: Budget/cooldown system gates intent acceptance
- **Dynamic PCG**: Spawns encounters based on accepted intents
- **Deterministic**: Fixed seed (456) ensures reproducible sequences

## Build & Run

```powershell
# Standard run
cargo run -p weaving_pcg_demo

# Release mode (faster)
cargo run -p weaving_pcg_demo --release
```

## Controls

| Key | Action |
|-----|--------|
| `Space` | Generate new encounter |
| `N` | Skip to next encounter |
| `P` | Show detailed pattern analysis |
| `R` | Reseed and restart |
| `Q` | Quit demo |

## HUD Display

```
=== WEAVING + PCG DEMO ===
Seed: 456
Tick: 42
Status: RUNNING

📊 Player State:
  Health: 65/100 (Low)
  Tension: 4.5

🎲 Current Encounter: 3/8
  Type: Combat
  Difficulty: 6
  Position: (12, 8)
  Status: ✓ Completed

📈 Pattern Analysis:
  Combat Streak: 2
  Avg Difficulty: 4.5
  Tension Score: 4.5

⚡ Weaving Signals:
  - HighTension

🎯 Intents:
  Proposed: 1 intents
    - ReduceTension
  Accepted (this run): 3 intents

📝 Upcoming Encounters:
  [04] Rest (diff: 0)
  [05] Combat (diff: 7)
  [06] Resource (diff: 0)
```

## Determinism

- **Fixed Seed**: 456
- **Stable PCG**: Same seed → same encounter sequence
- **Reproducible Signals**: Deterministic pattern detection

### Verification

Run twice and compare encounter sequences:

```powershell
cargo run -p weaving_pcg_demo --release > run1.txt
cargo run -p weaving_pcg_demo --release > run2.txt
diff run1.txt run2.txt
# Should be identical
```

## Weaving Architecture

### Pipeline

```
Gameplay Events → Pattern Detection → Signal Generation → Intent Proposal → Adjudication → Content Spawn
```

### Pattern Detection

Analyzes:
- **Combat Streak**: Consecutive combat encounters
- **Average Difficulty**: Mean difficulty of completed encounters
- **Tension Score**: Accumulated tension from encounters
- **Health Trend**: Player health status (Healthy/Low/Critical)

### Signal Types

| Signal | Trigger Condition |
|--------|------------------|
| LowHealth | Player health < 40 |
| HighTension | Tension > 5.0 |
| ResourceScarcity | (Not implemented in demo) |
| Momentum | Combat streak ≥ 3 |

### Intent Types

| Intent | Purpose | Acceptance Rule |
|--------|---------|----------------|
| SpawnAid | Add healing resource | Tension < 8.0 |
| SpawnIntel | Reveal information | Tension < 10.0 |
| ReduceTension | Add rest area | Tension > 3.0 |
| ScaleUp | Increase difficulty | Tension < 7.0 |

### Adjudication

Simple budget system:
1. Check if tension budget allows intent
2. Accept first valid intent per cycle
3. Execute intent (spawn encounter via PCG)
4. Track accepted intents for telemetry

## Encounter Types

| Type | Effect | Difficulty Range |
|------|--------|------------------|
| Combat | -HP, +Tension | 1-10 |
| Resource | +HP | 0 |
| Event | +Tension | 1-5 |
| Rest | +HP, -Tension | 0 |

## Implementation Notes

- **Simplified Weaving**: Minimal implementation for clarity
- **Pattern Window**: Analyzes all completed encounters (no sliding window)
- **Single Intent**: Only first valid intent accepted per cycle
- **Tension Budget**: Simple threshold checks, no complex economy
- **Auto-Advance**: Demo automatically progresses through encounters

## Feature Flags

This demo does **not** require `weaving` or `pcg` feature flags. It implements minimal inline systems for demonstration purposes. For production systems, use:

```toml
[dependencies]
astraweave-weaving = { path = "../../astraweave-weaving" }
astraweave-pcg = { path = "../../astraweave-pcg" }
```

## Pattern Analysis Detail (P key)

Press `P` to see comprehensive breakdown:

```
╔═══════════════════════════════╗
║   PATTERN ANALYSIS DETAIL     ║
╚═══════════════════════════════╝

🔍 Encounter History:
  ✓01: Combat (diff: 5, pos: (12, 3))
  ✓02: Resource (diff: 0, pos: (7, 15))
  ✓03: Combat (diff: 6, pos: (14, 8))
  ○04: Rest (diff: 0, pos: (5, 10))

📊 Statistics:
  Total Encounters: 8
  Completed: 3
  Combat Encounters: 2
  Current Streak: 1
  Average Difficulty: 3.67

⚡ Signal Generation Rules:
  LowHealth: health < 40
  HighTension: tension > 5.0
  Momentum: combat_streak >= 3

🎯 Intent Adjudication:
  SpawnAid: accepted if tension < 8.0
  ReduceTension: accepted if tension > 3.0
  SpawnIntel: accepted if tension < 10.0
  ScaleUp: accepted if tension < 7.0
```

## Related

- **Core Loop Dispatcher**: `astraweave-ai/src/core_loop.rs`
- **BT Demo**: `examples/core_loop_bt_demo/`
- **GOAP Demo**: `examples/core_loop_goap_demo/`
- **Weaving Playground**: `examples/weaving_playground/`

## Troubleshooting

**Issue**: All encounters complete too quickly  
**Solution**: Increase initial encounter count or reduce time scale.

**Issue**: No intents proposed  
**Solution**: Player health may be too high. Wait for tension to accumulate through combat.

**Issue**: Tension never decreases  
**Solution**: Ensure Rest encounters are being accepted (check tension > 3.0 for ReduceTension intent).

## Extensions

Possible enhancements for learning:
- Implement sliding window pattern analysis (last N encounters only)
- Add multiple intent acceptance per cycle with priority ranking
- Implement complex tension economy with cooldowns and budgets
- Add encounter chains (prerequisite system)
- Implement spatial PCG (room/path generation, not just encounters)
- Add player actions that influence pattern detection
