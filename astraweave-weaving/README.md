# astraweave-weaving

**Emergent behavior layer** for AstraWeave: pattern detection → intent proposal → budget/cooldown adjudication.

## Overview

The weaving system detects world patterns (e.g., "low health cluster", "resource scarcity") and proposes emergent gameplay events (e.g., "spawn wandering healer", "send supply drop") that feel reactive and organic without being scripted. It enforces budget constraints and cooldowns to prevent spamming.

## Architecture

```
WorldMetrics → PatternDetector → Patterns
                                    ↓
                            IntentProposer → Intents
                                               ↓
                                     WeaveAdjudicator → Approved Intents
```

### 1. Pattern Detection

**Trait**: `PatternDetector`

Scans `WorldMetrics` (aggregated world state) and returns detected patterns with strength values (0.0 to 1.0).

**Concrete Detectors**:
- `LowHealthClusterDetector`: Detects groups of critically injured entities
- `ResourceScarcityDetector`: Detects low resource availability per type
- `FactionConflictDetector`: Detects high tension between factions
- `CombatIntensityDetector`: Detects high combat activity over time

**Example**:
```rust
use astraweave_weaving::patterns::{LowHealthClusterDetector, PatternDetector, WorldMetrics};

let detector = LowHealthClusterDetector { min_cluster_size: 3 };
let metrics = WorldMetrics {
    critical_health_count: 5,
    avg_health: 0.3,
    ..Default::default()
};

let patterns = detector.detect(&metrics);
// Returns: [("low_health_cluster", 0.5)]
```

### 2. Intent Proposal

**Trait**: `IntentProposer`

Converts detected patterns into actionable `WeaveIntent` structs with priority, cost, and cooldown.

**Concrete Proposers**:
- `AidEventProposer`: Spawns healer NPC when health is low
- `SupplyDropProposer`: Sends resource crates when scarce
- `MediatorProposer`: Dispatches mediator NPC during faction conflict
- `ScavengerPatrolProposer`: Spawns looters during high combat

**Example**:
```rust
use astraweave_weaving::intents::{AidEventProposer, IntentProposer};
use std::collections::BTreeMap;

let proposer = AidEventProposer { strength_threshold: 0.5 };
let mut patterns = BTreeMap::new();
patterns.insert("low_health_cluster".to_string(), 0.8);

let intents = proposer.propose(&patterns, 12345);
// Returns: [WeaveIntent { kind: "spawn_aid_event", priority: 0.8, cost: 10, ... }]
```

### 3. Adjudication

**Struct**: `WeaveAdjudicator`

Enforces budget (per-tick action limit) and cooldowns (minimum time between same events). Returns only approved intents.

**Features**:
- Budget allocation per tick (default: 20 points)
- Cooldown tracking (e.g., aid events only every 5 seconds)
- Priority sorting with deterministic tie-breaking
- Minimum priority filtering

**Example**:
```rust
use astraweave_weaving::adjudicator::WeaveAdjudicator;
use astraweave_weaving::intents::WeaveIntent;

let mut adjudicator = WeaveAdjudicator::new();
adjudicator.begin_tick(); // Reset budget, decrement cooldowns

let intents = vec![
    WeaveIntent::new("spawn_healer").with_priority(0.9).with_cost(10).with_cooldown("aid_event"),
    WeaveIntent::new("spawn_supply").with_priority(0.7).with_cost(8).with_cooldown("supply_drop"),
];

let approved = adjudicator.adjudicate(intents);
// Returns: intents that fit budget and aren't on cooldown, sorted by priority
```

## Configuration

Adjudicator behavior is controlled via `WeaveConfig`:

```toml
# weave_config.toml
budget_per_tick = 20
min_priority = 0.3

[cooldowns]
aid_event = 300              # 5 seconds at 60Hz
supply_drop_food = 600       # 10 seconds
mediator = 900               # 15 seconds
scavenger_patrol = 450       # 7.5 seconds
```

Load from TOML:
```rust
use astraweave_weaving::adjudicator::WeaveConfig;

let config = WeaveConfig::from_toml(include_str!("weave_config.toml"))?;
let adjudicator = WeaveAdjudicator::with_config(config);
```

## Determinism

All intent proposal uses explicit seeds for deterministic behavior:

```rust
// Same patterns + same seed = same intents
let intents1 = proposer.propose(&patterns, 42);
let intents2 = proposer.propose(&patterns, 42);
assert_eq!(intents1, intents2); // Deterministic
```

## Integration Pattern

```rust
use astraweave_weaving::patterns::{LowHealthClusterDetector, PatternDetector, WorldMetrics};
use astraweave_weaving::intents::{AidEventProposer, IntentProposer};
use astraweave_weaving::adjudicator::WeaveAdjudicator;

// 1. Build world metrics from ECS
let metrics = WorldMetrics {
    avg_health: compute_avg_health(&world),
    critical_health_count: count_critical_health(&world),
    resource_scarcity: compute_resource_scarcity(&world),
    ..Default::default()
};

// 2. Detect patterns
let detectors: Vec<Box<dyn PatternDetector>> = vec![
    Box::new(LowHealthClusterDetector { min_cluster_size: 3 }),
];
let mut patterns = BTreeMap::new();
for detector in &detectors {
    patterns.extend(detector.detect(&metrics));
}

// 3. Propose intents
let proposers: Vec<Box<dyn IntentProposer>> = vec![
    Box::new(AidEventProposer { strength_threshold: 0.5 }),
];
let mut intents = Vec::new();
for proposer in &proposers {
    intents.extend(proposer.propose(&patterns, current_seed));
}

// 4. Adjudicate
let mut adjudicator = WeaveAdjudicator::new();
adjudicator.begin_tick();
let approved = adjudicator.adjudicate(intents);

// 5. Execute approved intents (spawn entities, trigger events)
for intent in approved {
    execute_weave_intent(&intent, &mut world);
}
```

## Testing

Run all tests:
```bash
cargo test -p astraweave-weaving --lib
```

**Test Coverage**: 21 unit tests
- Patterns: 7 tests (detector logic, threshold checks)
- Intents: 7 tests (proposal logic, determinism, multi-proposer)
- Adjudicator: 7 tests (budget, cooldowns, priority, config)

## Design Goals

1. **Emergent, Not Scripted**: System reacts to world state, not fixed triggers
2. **Budget-Controlled**: Prevents event spam via per-tick budget
3. **Cooldown-Protected**: Enforces minimum time between similar events
4. **Deterministic**: Same world state + seed = same intents
5. **Composable**: Easy to add new detectors and proposers
6. **Testable**: Pure functions, no global state

## Performance Notes

- Pattern detection runs per-tick (60Hz), keep detectors lightweight
- Use aggregated metrics (`WorldMetrics`) instead of scanning all entities in detectors
- Adjudicator sorting is O(n log n) where n = number of proposed intents (typically < 20)
- Cooldown tracking uses `BTreeMap` for deterministic ordering

## Future Enhancements

- [x] Persistent weave signals (events that span multiple ticks) - `CWeaveSignal` component
- [ ] Multi-tick intent chains (e.g., "spawn patrol, then ambush")
- [ ] Pattern history tracking (detect trends, not just current state)
- [ ] Weave configuration hot-reloading
- [ ] Visualization/debugging tools for pattern strength over time

## License

Part of the AstraWeave AI-Native Game Engine.
