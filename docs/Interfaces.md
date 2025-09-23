# AstraWeave Interfaces (MVP Contracts)

Purpose: Provide short, copy-pastable contracts so implementation can proceed consistently across crates, tests, and IPC.

Versioning: All structs/messages include a `version: u16` field. Breaking changes bump minor, with migration notes.

## Core AI Contracts

```rust
// Workhorse snapshot filtered by perception rules (LOS, range, noise)
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct WorldSnapshot {
    pub version: u16,
    pub tick: u64,
    pub time_ms: u64,
    pub self_id: u64,
    pub rng_seed: u64,
    pub region: String,
    pub player: Option<EntityView>,
    pub nearby_entities: Vec<EntityView>,
    pub objectives: Vec<ObjectiveHint>,
    pub hazards: Vec<HazardHint>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct EntityView {
    pub id: u64,
    pub kind: String,            // e.g., "npc", "projectile", "pickup"
    pub pos: [f32; 3],
    pub vel: [f32; 3],
    pub health: Option<HealthView>,
    pub faction: Option<u8>,
    pub los_visible: bool,       // true means redacted fields have been filled
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HealthView { pub current: u16, pub max: u16 }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ObjectiveHint { pub desc: String, pub pos: [f32; 3], pub priority: f32 }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct HazardHint { pub desc: String, pub pos: [f32; 3], pub radius: f32, pub cost: f32 }

// Planner output – deterministic sequence of atomic actions
#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct PlanIntent {
    pub version: u16,
    pub agent_id: u64,
    pub issued_tick: u64,
    pub steps: Vec<ActionStep>,
    pub justification: Option<String>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ActionStep {
    pub verb: String,                    // must exist in ToolRegistry
    pub args: serde_json::Value,         // verb-specific args schema
    pub min_duration_ms: u32,
    pub max_duration_ms: u32,
}

// Tool registry & validation constraints
bitflags::bitflags! {
    #[derive(serde::Serialize, serde::Deserialize)]
    pub struct ConstraintFlags: u32 {
        const LOS = 1<<0; const COOLDOWN = 1<<1; const STAMINA = 1<<2; const NAV = 1<<3; const BUDGET = 1<<4; const PHYSICS = 1<<5;
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolSpec { pub name: &'static str, pub args: Vec<ArgSpec>, pub constraints: ConstraintFlags }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ArgSpec { pub name: &'static str, pub ty: &'static str, pub optional: bool }

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct ToolRegistry { pub version: u16, pub tools: Vec<ToolSpec> }

// Orchestrator contract (async + cancellable + telemetry)
#[async_trait::async_trait]
pub trait Orchestrator {
    async fn plan(
        &self,
        snapshot: WorldSnapshot,
        budget_ms: u32,
        cancel: CancellationToken,
        telemetry: TelemetrySink,
    ) -> Result<PlanIntent, OrchestratorError>;

    fn name(&self) -> &'static str;
}

// Lightweight cancellation + telemetry
#[derive(Clone, Default)]
pub struct TelemetrySink; // implement no-op + feature gated exporters

#[derive(Clone, Default)]
pub struct CancellationToken { pub cancelled: std::sync::Arc<std::sync::atomic::AtomicBool> }
impl CancellationToken { pub fn is_cancelled(&self) -> bool { self.cancelled.load(std::sync::atomic::Ordering::Relaxed) } }

#[derive(thiserror::Error, Debug)]
pub enum OrchestratorError { #[error("budget exceeded")] BudgetExceeded, #[error("internal error: {0}")] Internal(String) }
```

## Navigation & Physics

```rust
// Nav baking job – inputs for tiled bake; output handled via crate API
pub struct NavBakeJob {
    pub cell_size: f32,
    pub agent_radius: f32,
    pub agent_height: f32,
    pub include_dynamic: bool,
}

pub struct NavAgent { pub radius: f32, pub height: f32, pub max_slope_deg: f32 }

pub struct CharacterControllerTickInput {
    pub desired_move_ws: [f32; 3],
    pub jump: bool,
    pub crouch: bool,
    pub dt: f32,
}

pub struct CharacterControllerTickOutput {
    pub new_pos_ws: [f32; 3],
    pub on_ground: bool,
    pub hit_slope_limit: bool,
    pub step_up_down: f32,
}
```

## Materials & Shaders

```rust
// Material Graph → WGSL compiler entry points (synchronous API; can be moved to async later)
pub struct MaterialGraph { /* nodes, connections, params */ }

pub struct CompileOptions { pub defines: Vec<(String, String)>, pub target: ShaderTarget }
pub enum ShaderTarget { Surface, Shadow, PostProcess }

pub struct CompileResult { pub wgsl: String, pub warnings: Vec<String> }

pub trait MaterialCompiler { fn compile(&self, graph: &MaterialGraph, opts: &CompileOptions) -> anyhow::Result<CompileResult>; }
```

## Networking Envelopes (Rust model)

```rust
#[derive(serde::Serialize, serde::Deserialize)]
pub struct MsgHeader { pub version: u16, pub session_id: u64, pub seq: u64, pub sent_ms: u64 }

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(tag="type", content="data")]
pub enum NetMsg {
    Input(InputMsg),
    Intent(PlanIntent),
    Snapshot(WorldSnapshot),
    ReplayFrame(ReplayFrame),
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct InputMsg { pub player_id: u64, pub buttons: u32, pub axes: [f32; 4] }

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ReplayFrame { pub tick: u64, pub rng_seed: u64, pub hash: u64 }
```

## Protobuf Schemas (snapshot + intent)

```proto
syntax = "proto3";
package astraweave;

message Header { uint32 version = 1; uint64 tick = 2; uint64 time_ms = 3; }

message Vec3 { float x = 1; float y = 2; float z = 3; }

message EntityView { uint64 id = 1; string kind = 2; Vec3 pos = 3; Vec3 vel = 4; uint32 faction = 5; bool los_visible = 6; uint32 health_cur = 7; uint32 health_max = 8; }

message WorldSnapshot { uint32 version = 1; uint64 tick = 2; uint64 time_ms = 3; uint64 self_id = 4; repeated EntityView nearby = 5; }

message ActionStep { string verb = 1; string args_json = 2; uint32 min_ms = 3; uint32 max_ms = 4; }

message PlanIntent { uint32 version = 1; uint64 agent_id = 2; uint64 issued_tick = 3; repeated ActionStep steps = 4; string justification = 5; }
```

Notes
- These are interface sketches; exact fields may differ per crate but should preserve versioning, determinism, and validation boundaries.
- Keep JSON/Proto schemas round-trippable; add migrations in `astraweave-memory` when fields change.
