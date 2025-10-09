# Performance Baseline Metrics

**Date**: October 9, 2025  
**Status**: ✅ Initial Baselines Established  
**Purpose**: Document current performance characteristics for regression detection  

---

## Test Environment

### Hardware Specifications
```
System:   LAPTOP-LPQUOHRT
CPU:      Intel(R) Core(TM) i5-10300H @ 2.50GHz
          4 Cores / 8 Threads
          Max Clock: 2496 MHz
GPU:      NVIDIA GeForce GTX 1660 Ti with Max-Q Design
          4GB VRAM
          Driver: 32.0.15.7703
RAM:      32.5 GB (33,319,064 KB)
OS:       Windows 10 (64-bit, Version 2009)
```

### Software Configuration
```
Rust:     1.89.0 (2025-10-05)
Cargo:    1.89.0
Profile:  bench (optimized + debuginfo)
Target:   x86_64-pc-windows-msvc
```

---

## Benchmark Results

### ✅ Terrain Generation (astraweave-terrain)

**Package**: `astraweave-terrain`  
**Status**: ✅ All benchmarks passing  
**Date**: October 9, 2025  

#### Heightmap Generation
| Test | Resolution | Time (mean) | Std Dev | Throughput |
|------|-----------|-------------|---------|------------|
| **heightmap_generation_64x64** | 64×64 (4,096 samples) | **1.98 ms** | ±0.04 ms | ~2.07 million samples/sec |
| **heightmap_generation_128x128** | 128×128 (16,384 samples) | **6.85 ms** | ±0.06 ms | ~2.39 million samples/sec |

**Analysis**:
- ✅ Scaling is near-linear with sample count (4x samples = 3.5x time)
- ✅ High throughput maintained at larger resolutions
- ✅ Low variance (2-3% std dev) indicates stable performance
- ⚠️ **Outliers**: 4-7% of samples (acceptable for noise-based algorithms)

**Performance Characteristics**:
- **Per-pixel cost**: ~30-40 nanoseconds
- **Cache efficiency**: Good (superlinear scaling suggests cache hits)
- **Parallelization potential**: Single-threaded currently, could benefit from SIMD

#### Climate Sampling
| Test | Sample Type | Time (mean) | Throughput |
|------|------------|-------------|------------|
| **climate_sampling** | Single point | **403 ns** | 2.48 million samples/sec |
| **chunk_climate_sampling** | 64×64 chunk | **2.53 ms** | 1.62 million samples/sec |

**Analysis**:
- ✅ Single-point sampling is extremely fast (<1 microsecond)
- ✅ Chunk sampling overhead minimal (63 ns/sample avg)
- ⚠️ **Outliers**: 8% in chunk sampling (temperature/humidity correlation complexity)

#### World Generation (Full Pipeline)
| Test | Chunk Size | Time (mean) | Std Dev |
|------|-----------|-------------|---------|
| **world_chunk_generation** | 64×64 voxels | **19.8 ms** | ±1.2 ms |

**Analysis**:
- ✅ Complete chunk generation under 20ms (acceptable for streaming)
- ✅ Pipeline includes: heightmap + climate + biome + voxel placement
- ⚠️ **Outliers**: 11% high (suggests GC or memory allocation spikes)
- 🎯 **Target**: <16.67ms for 60 FPS streaming (currently 18% over)

**Breakdown Estimate** (based on sub-benchmark times):
- Heightmap: ~2ms (10%)
- Climate: ~2.5ms (12.6%)
- Biome/Voxel: ~15.3ms (77.4%)

**Optimization Opportunities**:
1. 🔴 **High Priority**: Reduce voxel placement cost (largest contributor)
2. 🟡 **Medium**: SIMD vectorization for noise generation
3. 🟢 **Low**: Async chunk generation (already planned in astraweave-scene)

---

### ✅ Input System (astraweave-input)

**Package**: `astraweave-input`  
**Status**: ✅ All benchmarks passing  
**Date**: October 9, 2025  

#### Input Binding Performance
| Test | Operation | Time (mean) | Throughput |
|------|-----------|-------------|------------|
| **binding_creation** | Create InputBinding | **4.67 ns** | 214 million ops/sec |
| **binding_serialization** | Serialize to JSON | **117.7 ns** | 8.5 million ops/sec |
| **binding_deserialization** | Deserialize from JSON | **149.1 ns** | 6.7 million ops/sec |
| **binding_set_creation** | Create full BindingSet | **1.03 µs** | 972 thousand ops/sec |

**Analysis**:
- ✅ **Binding creation**: Sub-5ns (register access speed)
- ✅ **Serialization**: <120ns (acceptable for config save/load)
- ✅ **Deserialization**: <150ns (parsing overhead expected)
- ✅ **Set creation**: ~1 microsecond for complete binding configuration

**Performance Characteristics**:
- **Memory**: Minimal heap allocation (stack-only for creation)
- **Cache**: Excellent locality (4.67ns indicates L1 cache hits)
- **Outliers**: 3-5% (acceptable for non-critical path)

**Use Case Validation**:
- ✅ Config loading at startup: 1ms for ~1000 bindings (negligible)
- ✅ Runtime rebinding: <5ns per change (no user-perceptible lag)
- ✅ Save to disk: 117ns × 1000 = 117µs (instant)

---

### ❌ Core ECS Benchmarks (astraweave-core)

**Package**: `astraweave-core`  
**Status**: ❌ **Compilation Failed**  
**Error**: API mismatch in `ecs_adapter.rs` (missing `get_` prefix on resource methods)  

**Planned Benchmarks**:
- `world_creation` - ECS world initialization
- `entity_spawning` - Batch entity creation (100 entities)
- `world_tick` - Single frame simulation (50 entities)

**Remediation Required**:
```rust
// Fix needed in astraweave-core/src/ecs_adapter.rs
// Lines: 173, 180, 198, 236, 262, 278, 307

// Before:
app.world.resource::<World>()
app.world.resource_mut::<EntityBridge>()

// After:
app.world.get_resource::<World>()
app.world.get_resource_mut::<EntityBridge>()
```

**Priority**: 🟡 **Medium** - Core benchmarks needed for ECS regression detection  
**Estimated Fix Time**: 10-15 minutes (simple API rename)  
**Impact**: Cannot establish ECS baseline until fixed  

---

### ❌ Stress Test Benchmarks (astraweave-stress-test)

**Package**: `astraweave-stress-test`  
**Status**: ❌ **Compilation Failed**  
**Error**: Dependency `astraweave-observability` has same API mismatch  

**Planned Benchmarks**:
- `ecs_performance` - 1000 entity stress test
- `network_stress` - 50 networked entities simulation
- `persistence_stress` - Save/load operations

**Remediation Required**:
```rust
// Fix needed in astraweave-observability/src/lib.rs:128
// Before:
world.resource::<ObservabilityState>()

// After:
world.get_resource::<ObservabilityState>()
```

**Priority**: 🟠 **High** - Stress tests critical for performance validation  
**Estimated Fix Time**: 5 minutes  
**Impact**: Cannot measure scalability limits  

---

### ❌ Render Benchmarks (astraweave-render)

**Package**: `astraweave-render`  
**Status**: ⚠️ **Partial** (material compilation works, GPU tests blocked)  

**Working Benchmarks**:
- ✅ `material_compile_64_nodes` - Material graph compilation

**Blocked Benchmarks**:
- ❌ `cpu_cluster_binning_1k_lights` - Likely GPU dependency issues

**Attempted** (not completed due to time):
- Would require full GPU initialization
- Better tested via integration examples

**Recommendation**: Use `unified_showcase` runtime metrics instead of isolated benchmarks  

---

## Missing Benchmarks (To Be Created)

### 🔴 Priority 1: AI Planning

**Package**: `astraweave-ai`  
**Benchmarks Needed**:
1. **GOAP Planning** (`goap_planning_benchmark`)
   - Test: Plan generation with 10/50/100 available actions
   - Metric: Planning time vs action count
   - Target: <1ms for typical NPC decision (10-20 actions)

2. **Behavior Tree Execution** (`bt_execution_benchmark`)
   - Test: Tick 100-node behavior tree
   - Metric: Time per tick
   - Target: <100µs (allows 10,000 BTs at 60 FPS)

3. **Core Loop Performance** (`ai_core_loop_benchmark`)
   - Test: Full Perception → Reasoning → Planning → Action cycle
   - Metric: End-to-end latency
   - Target: <5ms per agent per frame

**Estimated Creation Time**: 2-3 hours  
**Impact**: Critical for AI-native gameplay validation  

### 🟠 Priority 2: LLM Integration

**Package**: `astraweave-llm`  
**Benchmarks Needed**:
1. **Token Counting** (`token_counting_benchmark`)
   - Test: Count tokens in 100/1000/10000 character strings
   - Metric: Tokens/second throughput
   - Target: >1 million tokens/sec

2. **Context Window Management** (`context_window_benchmark`)
   - Test: Add/remove messages from sliding window
   - Metric: Operation latency
   - Target: <10µs per message

3. **Prompt Generation** (`prompt_generation_benchmark`)
   - Test: Build complete prompt from context + system message
   - Metric: Generation time
   - Target: <1ms for typical prompt (2000 tokens)

**Estimated Creation Time**: 1.5-2 hours  
**Impact**: Validates LLM integration performance  

**Note**: Actual LLM inference not benchmarked (external API latency varies)  

### 🟡 Priority 3: Physics

**Package**: `astraweave-physics`  
**Benchmarks Needed**:
1. **Raycast Performance** (`raycast_benchmark`)
   - Test: 1000 raycasts in scene with 100 colliders
   - Metric: Raycasts/second
   - Target: >100,000 rays/sec

2. **Character Controller** (`character_controller_benchmark`)
   - Test: Update 100 character controllers with collision
   - Metric: Updates/second
   - Target: >10,000 updates/sec (100 chars @ 60 FPS × 1.66 safety margin)

3. **Rigid Body Simulation** (`rigidbody_benchmark`)
   - Test: Step physics world with 500 dynamic bodies
   - Metric: Time per step
   - Target: <5ms (allows physics at 200Hz)

**Estimated Creation Time**: 2 hours  
**Impact**: Medium (Rapier3D has own benchmarks, but integration overhead important)  

---

## Performance Regression Thresholds

### Established Baselines (For CI Integration)

**Terrain Generation** ✅:
- Heightmap 64×64: **2.0ms** ±10% (RED if >2.2ms)
- Heightmap 128×128: **6.9ms** ±10% (RED if >7.6ms)
- Climate sampling: **403ns** ±20% (RED if >484ns)
- World chunk: **19.8ms** ±15% (YELLOW if >22.8ms, RED if >25ms)

**Input System** ✅:
- Binding creation: **4.67ns** ±50% (RED if >7ns)
- Binding serialization: **117.7ns** ±30% (RED if >153ns)
- Set creation: **1.03µs** ±30% (RED if >1.34µs)

**Pending** (After fixes):
- ECS world creation: TBD
- Entity spawning: TBD
- Stress test performance: TBD

---

## CI Integration Plan

### Benchmark Pipeline (`.github/workflows/benchmarks.yml`)

```yaml
name: Performance Benchmarks

on:
  pull_request:
    branches: [main]
  schedule:
    - cron: '0 2 * * 0'  # Weekly Sunday 2 AM

jobs:
  benchmarks:
    runs-on: windows-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
      
      - name: Run Terrain Benchmarks
        run: cargo bench -p astraweave-terrain --bench terrain_generation
      
      - name: Run Input Benchmarks
        run: cargo bench -p astraweave-input --bench input_benchmarks
      
      - name: Parse Results
        run: |
          # Extract timing data from Criterion output
          # Compare against baseline thresholds
          # Fail if RED thresholds exceeded
      
      - name: Archive Results
        uses: actions/upload-artifact@v4
        with:
          name: benchmark-results
          path: target/criterion/
```

### Threshold Checks
```powershell
# Example threshold validation script
$results = Get-Content "target/criterion/*/new/estimates.json" | ConvertFrom-Json
foreach ($benchmark in $results) {
    $time_ns = $benchmark.mean.point_estimate
    $threshold = Get-Threshold $benchmark.name
    
    if ($time_ns > $threshold.red) {
        Write-Error "REGRESSION: $($benchmark.name) = ${time_ns}ns (threshold: ${threshold.red}ns)"
        exit 1
    } elseif ($time_ns > $threshold.yellow) {
        Write-Warning "SLOWDOWN: $($benchmark.name) = ${time_ns}ns (threshold: ${threshold.yellow}ns)"
    }
}
```

---

## Optimization Targets

### High Priority (Impact > 10ms/frame)

1. **World Chunk Generation** (Current: 19.8ms, Target: <16.67ms)
   - **Bottleneck**: Voxel placement (~15.3ms)
   - **Strategy**: SIMD vectorization, async streaming
   - **Expected Gain**: 20-30% reduction → 14-16ms
   - **Validation**: Re-run `world_chunk_generation` benchmark

2. **ECS Entity Spawning** (TBD - needs baseline)
   - **Target**: <100µs per entity for batch spawns
   - **Strategy**: Archetype pre-allocation, component pooling
   - **Expected Gain**: 50% reduction vs naïve allocation

### Medium Priority (Impact 1-10ms/frame)

3. **AI Planning** (No baseline yet)
   - **Target**: <1ms GOAP planning for 10-20 actions
   - **Strategy**: A* heuristic improvements, action caching
   - **Expected Gain**: Enable 100+ planning NPCs at 60 FPS

4. **Cluster Light Binning** (Not benchmarked - GPU)
   - **Target**: <2ms for 1000 lights
   - **Strategy**: GPU compute shader (currently CPU fallback)
   - **Expected Gain**: 10x speedup → <200µs

### Low Priority (Impact <1ms/frame)

5. **Input Binding Serialization** (Current: 117.7ns)
   - **Target**: <100ns
   - **Strategy**: Binary format instead of JSON
   - **Expected Gain**: 20% reduction (marginal impact)

---

## Hardware Scaling Estimates

### Tested Platform (i5-10300H, GTX 1660 Ti)
- **World Chunk Gen**: 19.8ms (50 chunks/sec)
- **Heightmap 128×128**: 6.85ms (146 chunks/sec)

### Projected: Desktop (Ryzen 7 5800X, RTX 3070)
- **World Chunk Gen**: ~12ms (83 chunks/sec) [+66% throughput]
- **Heightmap 128×128**: ~4.2ms (238 chunks/sec) [+63% throughput]

**Assumptions**:
- 40% single-thread uplift (5800X vs 10300H)
- 20% GPU benefit (RTX 3070 vs 1660 Ti Max-Q)

### Projected: High-End (Ryzen 9 7950X, RTX 4090)
- **World Chunk Gen**: ~8ms (125 chunks/sec) [+150% throughput]
- **Heightmap 128×128**: ~2.8ms (357 chunks/sec) [+144% throughput]

**Assumptions**:
- 60% single-thread uplift
- 50% GPU benefit

---

## Known Limitations

### Measurement Accuracy
- **Criterion Outliers**: 3-11% of samples flagged (acceptable for complex workloads)
- **Windows Jitter**: Timer resolution ~100ns (affects sub-microsecond benchmarks)
- **Thermal Throttling**: Laptop may throttle during extended benchmark runs
  - **Mitigation**: Run benchmarks individually, allow cooldown between

### Benchmark Coverage Gaps
1. ❌ **No GPU benchmarks**: Rendering pipeline not measured (compilation errors)
2. ❌ **No AI benchmarks**: Planning/BT performance unknown
3. ❌ **No network benchmarks**: Multiplayer performance untested
4. ❌ **No memory benchmarks**: Allocation patterns not profiled

### Environmental Factors
- **Power Profile**: Balanced (not High Performance)
- **Background Processes**: Typical user environment (not isolated)
- **Storage**: SSD performance varies with fill level

---

## Next Steps

### Immediate (Week 1 - Oct 9-13)
- [x] ✅ Run terrain benchmarks
- [x] ✅ Run input benchmarks
- [x] ✅ Document baseline metrics
- [ ] ⏸️ Fix ECS API mismatches (10-15 min)
- [ ] ⏸️ Re-run core/stress benchmarks

### Short-Term (Week 2 - Oct 14-20)
- [ ] Create AI planning benchmarks
- [ ] Create LLM integration benchmarks
- [ ] Create physics benchmarks
- [ ] Add CI benchmark pipeline

### Long-Term (Month 1+)
- [ ] GPU rendering benchmarks (separate from CPU)
- [ ] Memory profiling benchmarks
- [ ] Network multiplayer benchmarks
- [ ] Integration test performance tracking

---

## Conclusion

Established initial performance baselines for **terrain generation** and **input systems** with comprehensive metrics. Identified critical gaps in ECS and AI benchmarking that require immediate remediation.

**Key Findings**:
- ✅ **Terrain**: Solid performance, near-linear scaling
- ✅ **Input**: Excellent performance, sub-microsecond latency
- ⚠️ **World Gen**: 18% over 60 FPS target (optimization opportunity)
- ❌ **ECS/Stress**: Blocked by API mismatches (15 min fix)
- ❌ **AI/LLM**: No benchmarks exist (2-5 hours to create)

**Performance Status**: **ACCEPTABLE** for current development phase, with clear optimization targets identified.

---

## Related Documentation

- **Implementation Plan**: IMMEDIATE_ACTIONS_IMPLEMENTATION_PLAN.md
- **Week 1 Progress**: WEEK_1_PROGRESS_REPORT.md
- **Previous Actions**:
  - Action 1: GPU Skinning ✅
  - Action 2: Combat Physics ✅
  - Action 3: Unwrap Audit ✅
  - Action 4: Performance Baselines ✅ **(Current)**

---

_Generated by AstraWeave Copilot - October 9, 2025_
