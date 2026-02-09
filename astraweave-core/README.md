# astraweave-core

Central shared types and simulation vocabulary for the AstraWeave engine.

## Overview

This crate defines the canonical data types used across all engine subsystems, ensuring a single source of truth for the AI-native perception → reasoning → planning → action loop.

## Key Types

| Type | Description |
|------|-------------|
| `WorldSnapshot` | Filtered world state for AI perception |
| `PlanIntent` | AI-generated action plan with steps |
| `ActionStep` | Individual validated action in a plan |
| `ToolRegistry` | Registry of permitted AI actions |
| `ToolSpec` | Specification for a single AI tool |

## Modules

- **`schema`** — `WorldSnapshot`, `PlayerState`, `CompanionState`, `EnemyState`, `PlanIntent`, `ActionStep`
- **`tool_sandbox` / `tool_vocabulary`** — AI action validation
- **`capture_replay`** — Deterministic capture and replay infrastructure
- **`perception`** — AI helpers (`astar_path`, `find_cover_positions`, `los_clear`)
- **`validation`** — Configuration validation traits
- **`metrics`** — Telemetry and performance counters

## Architecture

```text
Perception → Reasoning → Planning → Action
    ↓           ↓           ↓          ↓
WorldSnapshot  AI Model   PlanIntent  Tool Validation
```

## Usage

```rust
use astraweave_core::{WorldSnapshot, PlanIntent, ActionStep};
use astraweave_core::{default_tool_registry, ToolRegistry};
```

## License

MIT
