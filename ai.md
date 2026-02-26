---
layout: default
title: AI Subsystem
---

# AI Orchestration (astraweave-ai)

AstraWeave's AI system supports multiple reasoning backends — from nanosecond classical AI to creative LLM-driven behavior — unified behind a single `Orchestrator` trait.

## Architecture

```
WorldSnapshot ──► Orchestrator ──► PlanIntent ──► ToolRegistry ──► Validated Actions
                   ▲
        ┌──────────┼──────────────┐
        │          │              │
  BehaviorTree   GOAP     Hermes 2 Pro LLM
  (57-253 ns)  (1-47 µs)     (2-8 s)
```

## AI Modes

| Mode | Backend | Latency | Use Case |
|------|---------|---------|----------|
| Classical | `BehaviorGraph` | 57–253 ns | Patrol, guard, combat |
| GOAP | `GOAPPlanner` | 1–47 µs | Strategic goal planning |
| LLM | Hermes 2 Pro (Ollama) | 2–8 s | Creative, emergent |
| Hybrid | Arbiter (GOAP + LLM) | 314 ns–8 s | GOAP for tactics, LLM for strategy |
| Utility | Utility curves | 460 ns | Dynamic priority scoring |
| Ensemble | All modes voting | 2.4 s | Maximum accuracy |

## Core Types

### WorldSnapshot

```rust
pub struct WorldSnapshot {
    pub t: f32,
    pub player: PlayerState,
    pub me: CompanionState,        // snap.me.pos, snap.me.ammo
    pub enemies: Vec<EnemyState>,
    pub pois: Vec<Poi>,
    pub obstacles: Vec<IVec2>,
    pub objective: Option<String>,
}
```

### PlanIntent

```rust
pub struct PlanIntent {
    pub plan_id: String,
    pub steps: Vec<ActionStep>,
}
```

### AIArbiter (Hybrid Mode)

The arbiter manages GOAP ↔ LLM transitions with configurable cooldowns:

```rust
let arbiter = AIArbiter::new(llm_executor)
    .with_llm_cooldown(Duration::from_secs(10));

match arbiter.mode() {
    AIControlMode::GOAP => { /* instant tactical decision */ }
    AIControlMode::ExecutingLLM { step_index } => { /* strategic LLM plan */ }
    AIControlMode::BehaviorTree => { /* fallback */ }
}
```

### 4-Tier Fallback

1. **Full LLM** → structured JSON plan
2. **Simplified LLM** → reduced context
3. **Heuristic** → rule-based backup
4. **Emergency** → safe default action

## Tool Sandbox

All AI actions are validated by the `ToolRegistry`:

- 37 registered tools across 6 categories
- Movement, Combat, Tactical, Utility, Support, Special
- Cooldown enforcement, range validation
- **No AI can cheat** — all actions go through validation

## Performance

- **12,700+ agents** at 60 FPS validated
- **6.48M** anti-cheat checks per second
- **100% deterministic** replay across runs
- 1,000 agents with hybrid AI = 0.6% frame budget

[← Back to Home](index.html) · [Architecture](architecture.html) · [ECS](ecs.html)
