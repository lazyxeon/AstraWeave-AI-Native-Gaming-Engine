# AstraWeave Architecture Reference

> **Extended reference** for code patterns, API deep dives, performance data, and testing patterns.
> The condensed versions live in `.github/copilot-instructions.md`. Read this file when you need full detail.

---

## Table of Contents

1. [AI-First Architecture](#ai-first-architecture)
2. [ECS System Stages](#ecs-system-stages)
3. [Rendering & Materials](#rendering--materials)
4. [Performance Optimization](#performance-optimization)
5. [API Deep Dives](#api-deep-dives)
6. [Testing Patterns](#testing-patterns)
7. [Workspace Structure (Detailed)](#workspace-structure-detailed)
8. [Performance Baselines](#performance-baselines)
9. [Formal Verification (Miri & Kani)](#formal-verification-miri--kani)

---

## AI-First Architecture

### Core Loop

```
Perception → Reasoning → Planning → Action
    ↓           ↓            ↓          ↓
WorldSnapshot  AI Model   PlanIntent  Tool Validation
```

**Key Concepts**:

- `WorldSnapshot`: Filtered world state for AI perception (see `astraweave-ai/src/core_loop.rs`)
- `PlanIntent` + `ActionStep`: AI decisions as validated action sequences
- `Orchestrator` trait: Abstracts AI planning (rule-based vs LLM)
- **Tool Sandbox**: All AI actions validated by engine (no cheating possible)

### GOAP+Hermes Hybrid Arbiter (Full Patterns)

```rust
// See astraweave-ai/src/arbiter.rs
use astraweave_ai::arbiter::{AIArbiter, AIControlMode};
use astraweave_ai::llm_executor::LlmExecutor;

// Pattern 1: Basic agent with arbiter
struct Agent {
    arbiter: AIArbiter,
    llm_executor: Arc<LlmExecutor>,
}

impl Agent {
    fn new(llm_executor: Arc<LlmExecutor>) -> Self {
        Self {
            arbiter: AIArbiter::new(llm_executor.clone()),
            llm_executor,
        }
    }

    fn update(&mut self, world: &mut World, snap: WorldSnapshot) -> Result<PlanIntent> {
        self.arbiter.update(world, &snap)?;
        match self.arbiter.mode() {
            AIControlMode::GOAP => {
                let plan = goap_orchestrator.plan(world, &snap)?;
                Ok(plan)
            }
            AIControlMode::ExecutingLLM { step_index } => {
                let llm_plan = self.arbiter.current_llm_plan().unwrap();
                Ok(execute_step(&llm_plan, step_index))
            }
            AIControlMode::BehaviorTree => {
                let plan = bt_orchestrator.plan(world, &snap)?;
                Ok(plan)
            }
        }
    }
}

// Pattern 2: Shared LLM executor (efficient for many agents)
let llm_executor = Arc::new(LlmExecutor::new(
    hermes_client,  // OllamaClient with Hermes 2 Pro model
    tool_registry,
));
let agents: Vec<Agent> = (0..100)
    .map(|_| Agent::new(llm_executor.clone()))
    .collect();

// Pattern 3: Custom cooldown (adjust LLM request frequency)
let arbiter = AIArbiter::new(llm_executor)
    .with_llm_cooldown(Duration::from_secs(5));   // Aggressive (high LLM usage)
    // .with_llm_cooldown(Duration::from_secs(30)); // Passive
    // .with_llm_cooldown(Duration::ZERO);           // Immediate (testing only)

// Pattern 4: Metrics monitoring
let metrics = arbiter.metrics();
let success_rate = metrics.llm_successes as f32
    / (metrics.llm_successes + metrics.llm_failures) as f32;

// Pattern 5: Manual mode transitions (advanced usage)
if emergency_situation {
    arbiter.transition_to_goap();
}
```

**Performance Characteristics**:
- **GOAP Control**: 101.7 ns per update (982× faster than target)
- **LLM Polling**: 575.3 ns per update
- **Mode Transitions**: 221.9 ns
- **Full Cycle**: 313.7 ns (GOAP update + LLM poll + metrics)
- **Scalability**: 1,000 agents @ 60 FPS = 0.6% frame budget, 10,000 agents = 6.1%

### Testing Patterns for Arbiter

```rust
use astraweave_ai::test_utils::MockLlmOrch;

#[tokio::test]
async fn test_arbiter_with_mock() {
    let mock_llm = Arc::new(MockLlmOrch::new_with_delay(
        Duration::from_millis(100),
        Some(mock_plan()),
    ));
    let llm_executor = Arc::new(LlmExecutor::new(mock_llm, tool_registry));
    let mut arbiter = AIArbiter::new(llm_executor);

    arbiter.update(&world, &snap)?;
    tokio::time::sleep(Duration::from_millis(150)).await;
    arbiter.update(&world, &snap)?;

    assert!(matches!(arbiter.mode(), AIControlMode::ExecutingLLM { .. }));
}

// Benchmarking with criterion
fn bench_arbiter_goap_control(c: &mut Criterion) {
    let arbiter = setup_arbiter();
    c.bench_function("arbiter_goap_control", |b| {
        b.iter(|| arbiter.update(black_box(&mut world), black_box(&snap)))
    });
}
```

📚 **Docs**: `docs/archive/completion_reports/ARBITER_IMPLEMENTATION.md`, `ARBITER_QUICK_REFERENCE.md`

---

## ECS System Stages

Deterministic, ordered execution at **fixed 60Hz tick** with deterministic RNG:

| Order | Stage | Purpose |
|-------|-------|---------|
| 1 | `PRE_SIMULATION` | Setup, initialization |
| 2 | `PERCEPTION` | Build WorldSnapshots, update AI sensors |
| 3 | `SIMULATION` | Game logic, cooldowns, state updates |
| 4 | `AI_PLANNING` | Generate PlanIntents from orchestrators |
| 5 | `PHYSICS` | Apply forces, resolve collisions |
| 6 | `POST_SIMULATION` | Cleanup, constraint resolution |
| 7 | `PRESENTATION` | Rendering, audio, UI updates |

---

## Rendering & Materials

- **wgpu 25.0.2** backend (Vulkan/DX12/Metal)
- **Material System**: TOML → GPU D2 array textures with stable indices
  - Pattern: `assets/materials/<biome>/{materials.toml, arrays.toml}`
  - WGSL bindings (group=1): albedo (0), sampler (1), normal (2), linear sampler (3), MRA (4)
- **Shared Utilities**: `MaterialManager`, `IblManager`, `MeshRegistry`
- **Feature Flags**: `textures`, `assets` gate loaders
- **GPU Skinning**: Production-ready pipeline with dual bone influence
  - `astraweave-render/src/skinning_gpu.rs`
- **GPU Mesh Optimization**:
  - `vertex_compression.rs` (octahedral normals, half-float UVs, 37.5% memory reduction)
  - `lod_generator.rs` (quadric error metrics, 3-5 LOD levels)
  - `instancing.rs` (GPU batching, 10-100× draw call reduction)

---

## Performance Optimization

### Tracy Profiling (Week 8)

Tracy 0.11.1 integrated for zero-overhead profiling. See `examples/profiling_demo/`.

### Spatial Hash Collision

O(n log n) grid-based spatial partitioning in `astraweave-physics/src/spatial_hash.rs`. 99.96% collision check reduction, cache locality cascade benefits.

### SIMD Movement

Batch processing for 2.08× speedup in `astraweave-math/src/simd_movement.rs`. `BATCH_SIZE=4` loop unrolling, glam auto-vectorization. ECS batching pattern: `collect() → SIMD → writeback` (3-5× faster than scattered `get_mut()`).

### Key Lessons Learned

1. **Amdahl's Law**: Only 0.15-22.4% parallelizable work → max 1.24× speedup (59% ECS overhead is sequential)
2. **Batching > Scattering**: ECS collect/writeback 3-5× faster than scattered `get_mut()`
3. **Overhead Threshold**: Only parallelize >5 ms workloads (Rayon overhead ~50-100 µs)
4. **SIMD Auto-Vec**: glam achieves 80-85% of hand-written AVX2, trust auto-vectorization
5. **Cache Locality Cascades**: Spatial hash improved ALL systems 9-17%, not just collision
6. **API Verification First**: Always read actual struct definitions before generating code
7. **Comprehensive Fixing**: Creating one corrected file vs piecemeal edits is more reliable
8. **Case Sensitivity Matters**: snake_case vs PascalCase mismatch caused 100% false positives
9. **Debug Early**: One debug logging statement revealed critical validation bug
10. **Production First**: Focus on working demo over 100% test coverage

---

## API Deep Dives

### WorldSnapshot API

```rust
// CORRECT API (from astraweave-core/src/schema.rs):
pub struct WorldSnapshot {
    pub t: f32,
    pub player: PlayerState,
    pub me: CompanionState,
    pub enemies: Vec<EnemyState>,
    pub pois: Vec<Poi>,
    pub obstacles: Vec<IVec2>,
    pub objective: Option<String>,
}

pub struct CompanionState {
    pub ammo: i32,
    pub cooldowns: BTreeMap<String, f32>,
    pub morale: f32,
    pub pos: IVec2,
}

pub struct PlanIntent {
    pub plan_id: String,
    pub steps: Vec<ActionStep>,
}

// Usage:
let enemy_pos = snap.enemies[0].pos;           // ✅
let my_pos = snap.me.pos;                      // ✅
let my_ammo = snap.me.ammo;                    // ✅
let cooldown = snap.me.cooldowns.get("attack"); // ✅
let poi = snap.pois.first().map(|p| p.pos);    // ✅
```

### BehaviorGraph API

```rust
use astraweave_behavior::{BehaviorGraph, BehaviorNode, BehaviorContext, BehaviorStatus};

let combat_seq = BehaviorNode::Sequence(vec![
    BehaviorNode::Condition("check_threat".into()),
    BehaviorNode::Action("throw_smoke".into()),
]);
let move_seq = BehaviorNode::Sequence(vec![
    BehaviorNode::Action("move_to_objective".into()),
]);
let root = BehaviorNode::Selector(vec![combat_seq, move_seq]);
let graph = BehaviorGraph::new(root);

let context = BehaviorContext::new(snap);
let status = graph.tick(&context);
```

### hello_companion Performance (Phase 6)

| Mode | Latency |
|------|---------|
| Classical | 0.20 ms |
| BehaviorTree | 0.17 ms |
| Utility | 0.46 ms |
| LLM (Hermes 2 Pro) | 3,462 ms |
| Hybrid | 2,155 ms |
| Ensemble | 2,355 ms |

---

## Workspace Structure (Detailed)

### Core Engine Crates

| Crate | Purpose |
|-------|---------|
| `astraweave-ecs` | Archetype-based ECS, system stages, events |
| `astraweave-ai` | AI orchestrator, core loop, tool sandbox |
| `astraweave-sdk` | C ABI, header generation (SDK exports) |
| `astraweave-render` | wgpu 25 renderer, materials, IBL, GPU skinning, mesh optimization |
| `astraweave-physics` | Rapier3D wrapper, character controller, spatial hash |
| `astraweave-gameplay` | Combat physics, attack sweep |
| `astraweave-nav` | Navmesh, A*, portal graphs |
| `astraweave-audio` | Spatial audio, rodio backend |
| `astraweave-scene` | World partition, async cell streaming |
| `astraweave-terrain` | Voxel/polygon hybrid, marching cubes |
| `astraweave-cinematics` | Timeline, sequencer, camera/audio/FX tracks |
| `astraweave-math` | SIMD vector/matrix ops (glam-based), movement optimization |

### Gameplay & Tools

| Crate | Purpose |
|-------|---------|
| `astraweave-behavior` | Behavior trees, utility AI |
| `astraweave-weaving` | Fate-weaving system (Veilweaver game mechanic) |
| `astraweave-pcg` | Procedural content generation |
| `tools/aw_editor` | Level/encounter editor (GUI) |
| `tools/aw_asset_cli` | Asset pipeline tooling |

### Examples Status

- ✅ Working: `hello_companion`, `unified_showcase`, `core_loop_bt_demo`, `core_loop_goap_demo`, `weaving_pcg_demo`, `profiling_demo`
- ⚠️ API Drift: `ui_controls_demo`, `debug_overlay` (egui/winit version mismatches)
- ❌ Broken: `astraweave-author`, `rhai_authoring` (rhai sync trait issues)

---

## Performance Baselines

| System | Metric | Notes |
|--------|--------|-------|
| ECS | 25.8 ns world creation, 420 ns/entity spawn, <1 ns/entity tick | |
| AI Core Loop | 184 ns – 2.10 µs | 2500× faster than 5 ms target |
| GOAP | 1.01 µs cache hit (97.9% faster), 47.2 µs cache miss | |
| Behavior Trees | 57–253 ns | 66,000 agents @ 60 FPS possible |
| Terrain | 15.06 ms world chunk | 60 FPS budget achieved |
| Input | 4.67 ns binding creation | |
| Physics | 114 ns char move, 6.52 µs full tick, 2.97 µs rigid body step | |
| GPU Mesh | 21 ns vertex compression, 37.5% memory reduction | |
| SIMD Math | 2.08× speedup (20.588 µs → 9.879 µs @ 10k entities) | |
| Frame Time | 2.70 ms @ 1,000 entities, 370 FPS | Week 8 |
| AI-Native | 12,700+ agents @ 60 FPS, 6.48M checks/sec | 100% determinism |

**See also**: `docs/current/MASTER_BENCHMARK_REPORT.md`

---

## Formal Verification (Miri & Kani)

### Miri UB Detection

**Status**: All unsafe code validated — 977 tests, **ZERO undefined behavior**.

**Validated crates** (4):
- `astraweave-ecs` (386 tests): BlobVec, SparseSet, EntityAllocator, SystemParam
- `astraweave-math` (109 tests): SIMD vec/mat/quat, SSE2 scalar fallback
- `astraweave-core` (465 tests): Entity::from_raw, capture/replay
- `astraweave-sdk` (17 tests): C ABI FFI, raw pointer handling

**CI**: `.github/workflows/miri.yml` (weekly, nightly toolchain)

**Running locally**:
```bash
cargo +nightly miri test -p astraweave-ecs --lib -- --test-threads=1
# Flags: -Zmiri-symbolic-alignment-check -Zmiri-strict-provenance
```

**Report**: `docs/current/MIRI_VALIDATION_REPORT.md`

### Kani Formal Verification

**Purpose**: Model-checks critical properties for ALL possible inputs (unlike tests which check specific inputs).

**Validated crates** (3): `astraweave-ecs`, `astraweave-math`, `astraweave-sdk`

**Proof locations**:
- `astraweave-ecs/tests/mutation_resistant_comprehensive_tests.rs` — BlobVec, Entity bit encoding, EntityAllocator proof mirrors
- `astraweave-sdk/src/lib_kani.rs` — C ABI function proofs

**CI**: `.github/workflows/kani.yml` (uses `model-checking/kani-github-action@v1`)

**Running locally**:
```bash
cargo kani --package astraweave-sdk
cargo kani --package astraweave-ecs
```

### Unsafe Code Requirements

Any new `unsafe` code MUST:
1. Pass Miri (`cargo +nightly miri test -p <crate>`)
2. Have a corresponding Kani proof or Kani-mirror test
3. Include a `// SAFETY:` comment explaining the invariant
4. Be validated in CI before merge

---

## Key Files to Check

| Need | File(s) |
|------|---------|
| Public APIs | Each crate's `src/lib.rs` |
| Workspace deps | Root `Cargo.toml` |
| Build config | `.cargo/config.toml` (aliases, profiles, sccache) |
| CI tasks | `.vscode/tasks.json` |
| Exclusions | `check-all` alias in `.cargo/config.toml` |
| Miri validation | `docs/current/MIRI_VALIDATION_REPORT.md` |
| Kani proofs | `astraweave-sdk/src/lib_kani.rs`, `astraweave-ecs/tests/mutation_resistant_comprehensive_tests.rs` |

---

*Last updated: February 8, 2026*
