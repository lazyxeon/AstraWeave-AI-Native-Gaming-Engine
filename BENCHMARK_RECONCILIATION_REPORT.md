# AstraWeave Benchmark Reconciliation Report

**Date**: November 12, 2025  
**Purpose**: Reconcile Master Benchmark Report claims (575 benchmarks) with actual runnable benchmarks  
**Status**: ⚠️ **SIGNIFICANT DISCREPANCY IDENTIFIED**

---

## Executive Summary

**Master Benchmark Report Claims**: **575 benchmarks across 37 crates**  
**Actual Runnable Benchmarks**: **29 benchmarks across 3 crates**  
**Discrepancy**: **546 benchmarks (94.9%) are aspirational/non-existent**

### Breakdown

| Category | Claimed | Actual | Gap | Notes |
|----------|---------|--------|-----|-------|
| **Compilation Status** | 37 crates | 21 compile | 16 broken | 43% fail to compile |
| **Criterion Output** | 575 benchmarks | 29 results | 546 missing | 94.9% don't execute |
| **Functional Suites** | 37 crates | 3 crates | 34 empty | Only weaving, stress, math run |

---

## Root Cause Analysis

### 1. Master Benchmark Report is Aspirational (Not Actual)

The Master Benchmark Report documents **benchmarks that SHOULD exist** based on the project's design goals, not benchmarks that currently execute successfully.

**Evidence**:
- Claims 575 benchmarks, but only 29 Criterion output directories exist
- Claims benchmarks for crates that fail to compile (e.g., astraweave-ai, astraweave-render)
- Claims benchmarks for crates with empty `benches/` directories

**Example: astraweave-nav**
- **Claimed**: 18 benchmarks (2.44 µs short path, 142k QPS @ 100 triangles)
- **Actual**: Compiles successfully, but **ZERO Criterion output**
- **Why**: Benchmark file exists but doesn't execute (missing `criterion_main!` or broken setup)

### 2. Three Types of "Benchmarks" in the Codebase

#### Type A: **Working Benchmarks** (29 total)
These produce Criterion output and export to the dashboard:

**astraweave-weaving** (18 results):
- Rendering Culling (2)
- Enemy Archetype (5 parametric)
- Player Abilities (4 parametric)
- Player Activation (2)
- Quest Tracking (4 parametric)
- Integrated Systems (3 parametric)

**astraweave-stress-test** (5 results):
- Frame Time Baseline (1)
- Shader Compilation (1)
- Texture Operations (3 parametric)

**astraweave-math** (4 results):
- Vec3 Dot Scalar (1)
- Vec3 Dot SIMD (1)
- Vec3 Dot Throughput Scalar (1)
- Vec3 Dot Throughput SIMD (1)

**aw-save** (2 results - NEW from latest run):
- Serialization benchmarks (2)

#### Type B: **Placeholder Benchmarks** (21 crates)
These have `[[bench]]` in Cargo.toml and compile, but produce **ZERO output**:

- astraweave-behavior
- astraweave-context
- astraweave-core (✅ **FIXED in this session!** - now compiles, waiting for first run)
- astraweave-ecs
- astraweave-input
- astraweave-llm
- astraweave-memory
- astraweave-nav
- astraweave-net-ecs
- astraweave-pcg
- astraweave-persistence-ecs
- astraweave-persona
- astraweave-physics
- astraweave-rag
- astraweave-sdk
- astract
- aw-save
- aw_build

**Why no output?**
- Missing `criterion_group!` macro calls
- Missing `criterion_main!` macro calls
- Empty benchmark functions (just stubs)
- Setup code exists but no actual benchmark invocations

#### Type C: **Broken Benchmarks** (8 crates)
These fail to compile:

1. **astraweave-ai** (claimed 30+ benchmarks)
   - **Errors**: IVec2 type mismatches, missing CAiController::new(), dispatch_planner signature changed, ActionStep variant errors
   - **Fix Complexity**: HIGH (12+ distinct compilation errors, API mismatches)

2. **astraweave-render** (claimed 21 benchmarks)
   - **Errors**: Missing tracing import in astraweave-asset dependency
   - **Fix Complexity**: MEDIUM (dependency issue)

3. **astraweave-audio** (claimed ???)
   - **Errors**: Not tested in detail
   - **Fix Complexity**: UNKNOWN

4. **astraweave-terrain** (claimed ???)
   - **Errors**: Not tested in detail
   - **Fix Complexity**: UNKNOWN

5. **astraweave-ui** (claimed ???)
   - **Errors**: egui/winit version conflicts
   - **Fix Complexity**: HIGH (requires version upgrades)

6. **aw_editor** (claimed ???)
   - **Errors**: egui/winit version conflicts
   - **Fix Complexity**: HIGH (requires version upgrades)

7. **astraweave-llm-eval** (claimed ???)
   - **Errors**: Not tested in detail
   - **Fix Complexity**: UNKNOWN

8. **astraweave-prompts** (claimed 22 benchmarks)
   - **Errors**: Not tested in detail
   - **Fix Complexity**: UNKNOWN

---

## Detailed Crate-by-Crate Analysis

### P0 Crates (Core Engine)

#### astraweave-core
- **Claimed**: 40+ benchmarks (full game loop, perception, planning, physics)
- **Actual**: 0 results (✅ NOW COMPILES after this session!)
- **Status**: ⚠️ **FIXED BUT NOT RUN YET**
- **Fixes Applied**:
  - Added missing imports (WorldSnapshot, RuleOrchestrator, ActionStep, etc.)
  - Fixed RuleOrchestrator instantiation (unit struct, use `RuleOrchestrator` not `.new()`)
  - Fixed Orchestrator trait import (need `propose_plan` method)
  - Fixed Pose initialization (added rotation fields)
  - Fixed TakeCover position unwrapping (Option<IVec2>)
  - Fixed mutable borrow in benchmark setup (use `iter_with_setup`)
- **Next Run Expected**: 8-12 new benchmark results (full_game_loop suite)

#### astraweave-ecs
- **Claimed**: 50+ benchmarks (world creation, entity spawn, query iteration)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output
- **Why**: Likely placeholder benchmarks (no `criterion_main!`)

#### astraweave-ai
- **Claimed**: 30+ benchmarks (GOAP, BehaviorTree, Arbiter, multi-agent pipeline)
- **Actual**: 0 results
- **Status**: ❌ **DOES NOT COMPILE**
- **Errors** (12+ distinct):
  1. `goap_vs_rule_bench.rs`: Empty `criterion_group!(benches)` (no functions listed)
  2. `multi_agent_pipeline.rs`: `glam::ivec2()` produces `glam::IVec2`, but code expects `astraweave_core::IVec2`
  3. `multi_agent_pipeline.rs`: `CAiController::new()` doesn't exist (no such method)
  4. `multi_agent_pipeline.rs`: `dispatch_planner()` signature changed (3 args → 2 args)
  5. `multi_agent_pipeline.rs`: `ActionStep::Wait` is struct variant (needs `{ duration: _ }`)
  6. `arbiter_bench.rs`: Unused imports (warnings)
  7. Binary targets (`analyze-plan`, `visualize-plan`, `validate-goals`): Import `astraweave_ai::goap` (doesn't exist)
- **Fix Complexity**: **HIGH** (extensive API drift, requires code refactoring, not just imports)

#### astraweave-physics
- **Claimed**: 30+ benchmarks (raycasts, character controller, rigid body)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output
- **Why**: Likely placeholder benchmarks

### P1 Crates (Rendering, Audio, Navigation)

#### astraweave-render
- **Claimed**: 21 benchmarks (vertex compression, LOD, instancing)
- **Actual**: 0 results
- **Status**: ❌ **DOES NOT COMPILE** (dependency issue)
- **Errors**: Missing `tracing` and `VecDeque` imports in `astraweave-asset`
- **Fix Complexity**: MEDIUM (already applied automatic fix for VecDeque, but more issues remain)

#### astraweave-nav
- **Claimed**: 18 benchmarks (pathfinding, baking, throughput)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output
- **Why**: Benchmark file exists with functions, but no execution

#### astraweave-audio
- **Claimed**: ??? benchmarks
- **Actual**: 0 results
- **Status**: ❌ **DOES NOT COMPILE**
- **Fix Complexity**: UNKNOWN

### P2 Crates (LLM, Memory, Context, Persona, Prompts, RAG)

#### astraweave-memory
- **Claimed**: 9 benchmarks (all sub-10µs)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output

#### astraweave-context
- **Claimed**: 17 benchmarks (all sub-200µs)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output

#### astraweave-persona
- **Claimed**: 22 benchmarks (544 ps verification - fastest in AstraWeave!)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output

#### astraweave-prompts
- **Claimed**: 22 benchmarks (all sub-10µs)
- **Actual**: 0 results
- **Status**: ❌ **DOES NOT COMPILE**
- **Fix Complexity**: UNKNOWN

#### astraweave-rag
- **Claimed**: 22 benchmarks (2.18 ns engine creation)
- **Actual**: 0 results
- **Status**: ✅ Compiles, ⚠️ No Criterion output

---

## Recommendations

### Option 1: ✅ **Update Master Benchmark Report to Reflect Reality** (RECOMMENDED)

**Action**: Clearly label claimed benchmarks as "TARGET" or "PLANNED" vs "ACTUAL"

**Rationale**:
- The Master Benchmark Report is currently **misleading** - it claims 575 benchmarks exist when only 29 actually run
- Marking aspirational benchmarks as "PLANNED" preserves their value as design goals while being honest about current state
- Prevents future confusion and wasted debugging effort

**Example Format**:
```markdown
### astraweave-nav (18 benchmarks PLANNED, 0 ACTUAL)

**Target Performance** (from design specs):
- Pathfind Short: 2.44 µs (142k QPS @ 100 triangles)
- Baking 100 triangles: <10 ms
- Baking 10k triangles: 473 ms (must be async/precomputed)

**Current Status**: ❌ Benchmark file exists but produces zero Criterion output  
**Blocker**: Missing `criterion_main!` macro or setup code issues  
**Priority**: P1 (Navigation critical for gameplay)
```

### Option 2: Implement All Placeholder Benchmarks (20-40 hours)

**Effort Estimate**: 20-40 hours across 21 crates

**Priority Order**:
1. **astraweave-nav** (18 benchmarks) - Navigation critical, design already complete
2. **astraweave-pcg** (10-15 benchmarks) - PCG benchmarks exist but don't execute
3. **astraweave-physics** (30+ benchmarks) - Physics benchmarks exist but don't execute
4. **P2 Crates** (memory, context, persona, prompts, rag) - 92 total benchmarks claimed

**Blockers**:
- Need to understand why existing benchmark code doesn't execute
- May require refactoring benchmark setup code
- Some crates may need API updates to match current codebase

### Option 3: Fix Broken Benchmark Crates (8-12 hours)

**High Priority** (4-6 hours):
1. **astraweave-ai** - Fix 12+ compilation errors (HIGH complexity, needs API updates)
2. **astraweave-render** - Fix dependency issues (MEDIUM complexity)

**Low Priority** (4-6 hours):
3. **astraweave-audio** - Unknown errors (investigate)
4. **astraweave-terrain** - Unknown errors (investigate)
5. **astraweave-prompts** - Unknown errors (investigate)
6. **astraweave-llm-eval** - Unknown errors (investigate)

**Deferred** (egui/winit version conflicts):
7. **astraweave-ui** - Requires version upgrades
8. **aw_editor** - Requires version upgrades

---

## Progress This Session

### ✅ Completed
1. **Audited Master Benchmark Report** - Identified 575 claimed vs 29 actual discrepancy
2. **Fixed astraweave-core benchmarks** - Added all missing imports, fixed API issues (6 fixes applied)
3. **Created comprehensive analysis** - Documented all 29 crates, categorized Type A/B/C
4. **Improved compilation rate** - 20/29 → 21/29 crates now compile (5% improvement)

### ⏸️ Deferred (Time Constraint)
1. **astraweave-ai fixes** - 12+ compilation errors (HIGH complexity, 4-6 hours estimated)
2. **Placeholder benchmark implementation** - 21 crates need actual Criterion code (20-40 hours)
3. **Master Benchmark Report updates** - Recommend clear "PLANNED" vs "ACTUAL" labeling

---

## Conclusion

**The Master Benchmark Report is aspirational documentation, not a record of actual benchmarks.**

- **Only 29 benchmarks actually execute** (5% of 575 claimed)
- **21 crates compile but produce zero output** (placeholder benchmarks)
- **8 crates don't compile** (compilation errors blocking ~100+ benchmarks)

**Recommendation**: Update Master Benchmark Report to clearly distinguish between:
- ✅ **ACTUAL** - Benchmarks that currently execute and export data
- 🎯 **PLANNED** - Benchmarks designed but not yet implemented
- ⚠️ **BLOCKED** - Benchmarks blocked by compilation errors

This preserves the document's value as a design specification while being honest about current implementation status.

**Current Dashboard State**: ✅ **WORKING CORRECTLY** - Displays exactly 29 actual benchmark results. No bugs in dashboard, export, or runner scripts. The discrepancy is in the Master Benchmark Report's claims.
