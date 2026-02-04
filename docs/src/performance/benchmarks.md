# Performance Benchmarks

> **Data Source**: All benchmarks are executed via Criterion.rs statistical benchmarking. Results sourced from [Master Benchmark Report](../../masters/MASTER_BENCHMARK_REPORT.md) v5.55 (January 2026).
>
> **Reproducibility**: Every measurement can be reproduced with documented commands.

## Executive Summary

| Metric | Value | Updated |
|--------|-------|---------|
| **Total Benchmarks** | ~1,500 across 76 sections | Jan 2026 |
| **Criterion Result Directories** | 1,700+ | Jan 2026 |
| **Measurement Tool** | Criterion.rs + Real Ollama validation | — |
| **Engine Status** | ✅ Production Ready (Grade A+) | Jan 2026 |

### Key Validation Results (January 2026)

| System | Benchmark | Target | Actual | Margin |
|--------|-----------|--------|--------|--------|
| **ECS** | Entity spawn (100) | <50µs | 15.0µs | 70% under |
| **ECS** | Entity spawn (1000) | <500µs | 106.7µs | 79% under |
| **AI** | GOAP planning (full) | <10µs | 286ns | 97% under |
| **AI** | GOAP planning (cache) | <1µs | 9.8ns | 99% under |
| **Frame** | p50 @ 1k entities | <16.67ms | 1.27ms | 92% under |
| **Frame** | p99 @ 1k entities | <16.67ms | 2.42ms | 85% under |

**Key Finding**: Engine has **85% headroom** at p99 for 1,000 entities @ 60 FPS.

---

## Best Performers

These benchmarks represent AstraWeave's highest-performing operations, many achieving sub-nanosecond latency:

### Sub-Nanosecond Operations (< 1 ns)

| Operation | Latency | Throughput | Notes |
|-----------|---------|------------|-------|
| Multi-Agent Per-Agent (amortized) | 12-20 ps | 50-83 trillion/sec | #1 fastest |
| Navigation Sliver Triangles | 99-104 ps | 10 billion/sec | Degenerate geometry handling |
| Multi-Agent Validation Per-Plan | 0.29-0.31 ns | 3.2-3.4 billion/sec | Plan validation |
| Pan Mode Switching | 418 ps | — | Audio state |
| State Transitions | 0.49-0.51 ns | — | Editor gizmo state |
| Emotion Blending | 0.55 ns | — | Affective computing |
| Multi-Agent Feedback Per-Agent | 0.73-0.76 ns | 1.3 billion/sec | Agent feedback |
| MSAA Resize 720p | 582-645 ps | — | Render target resize |
| UI Settings Navigation | 696 ps | — | UI lookup |
| Clear Frame | 0.72 ns | — | Frame buffer clear |
| Weather Light Attenuation | 730-783 ps | 22.8 billion/frame | Weather query |
| Room Overlap Check | 571-629 ps | — | Collision detection |
| Frustum AABB Inside | 889-915 ps | — | Frustum culling |
| GPU Budget Check | 890 ps-1.05 ns | 17 billion/frame | Memory budget |

### Sub-10ns Operations

| Operation | Latency | Notes |
|-----------|---------|-------|
| SparseSet Lookup (1000 entities) | 1.56 ns | O(1) at scale, 37× faster than BTreeMap |
| SIMD Movement (per entity) | 1.73 ns | 2.26× faster than naive |
| Quat Multiply | 1.34 ns | glam SIMD-optimized |
| Quat Slerp | 2.10 ns | Rotation interpolation |
| Context Switching | 2.38 ns | 7M switches/frame capacity |
| GOAP Next Action (no enemies) | 3.46-3.56 ns | Idle detection FREE |
| Component Deserialize | 3.50 ns | Postcard ECS deserialization |
| Physics Stage (per agent) | 3.63 ns | 7,580× faster than perception |
| RAG Engine Creation | 4.61 ns | Zero-cost abstraction |
| Mat4 Multiply | 4.28 ns | glam SIMD matrix |
| GOAP Next Action (close) | 4.68-5.11 ns | Tactical decision |
| GOAP Next Action (far) | 7.04-7.86 ns | Strategic decision |
| SparseSet Insert (per entity) | 9.9 ns | 13× faster than BTreeMap |

---

## Core Systems

### ECS Performance (astraweave-ecs)

> Fixed in v5.53: BlobVec lazy initialization restored full performance (50-68% improvement).

| Benchmark | Result | Budget Used |
|-----------|--------|-------------|
| Entity spawn empty (10k) | 645µs | Excellent |
| Entity spawn with Position (10k) | 5.6ms | Production-ready |
| Entity despawn empty (10k) | 287µs | Fixed |
| Entity despawn with components (10k) | 2.5ms | 68% faster |
| Component iteration (10k) | 273µs | Excellent |
| Archetype transition (10k) | 5.6ms | Within budget |

**60 FPS Capacity**:

| Entity Count | ECS Time | Budget Used |
|--------------|----------|-------------|
| 1,000 | ~85µs | 0.51% |
| 5,000 | ~529µs | 3.17% |
| 10,000 | ~1ms | ~6% |

### AI Performance (astraweave-ai)

| Benchmark | Result | Notes |
|-----------|--------|-------|
| GOAP planning (cache hit) | 9.8 ns | 99% under target |
| GOAP planning (cache miss) | 286 ns | 97% under target |
| GOAP next action (no enemies) | 3.5 ns | 4.7B ops/frame |
| GOAP next action (close) | 5.1 ns | 3.5B ops/frame |
| GOAP next action (far) | 7.9 ns | 2.4B ops/frame |
| Multi-agent (10 agents) | 1.34-1.39 µs | 66-68% faster (v5.49) |
| AIArbiter GOAP control | 101.7 ns | 982× faster than target |
| AIArbiter LLM polling | 575 ns | 86× faster than target |
| AIArbiter mode transition | 221.9 ns | 45× faster than target |

### Physics Performance (astraweave-physics)

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Character move | 43.8-52.0 ns | 12-26% faster (v5.48) |
| Rigid body transform lookup | 14.8-15.4 ns | 10× faster than character |
| Raycast empty scene | 26.3-31.5 ns | 8-23% faster (v5.48) |
| Rigid body batch (100) | 47µs | Excellent |
| Spatial hash collision | 99.96% fewer checks | Grid optimization |

### Fluids Performance (astraweave-fluids)

> A+ grade, 2,404 tests

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Particle operations (1K-10K) | 5.3-110µs | 100-322 Melem/s |
| Spatial hashing | 163µs-5.6ms | 38-62% improvement |
| SPH kernels (100K) | 171-223µs | poly6/spiky/viscosity |
| Density/pressure (2-5K) | 3.5-10.5ms | — |
| Simulation step (1K) | 1.8-3.0ms | — |
| Multi-step | 450-500µs | 45-57% faster |
| GPU data prep | 0.9-2.6 ns | Sub-nanosecond! |

### Rendering Performance (astraweave-render)

| Category | Benchmark | Result |
|----------|-----------|--------|
| **Culling** | Frustum AABB inside | 889-915 ps |
| **Culling** | AABB contains point | 951 ps-1.01 ns |
| **MSAA** | Mode check | 795-842 ps |
| **MSAA** | Resize 720p | 582-645 ps |
| **Camera** | View matrix | 4.42-5.36 ns |
| **Camera** | Toggle mode | 1.72-2.29 ns |
| **Instancing** | Savings calc | 1.43-1.52 ns |
| **Weather** | Particle update | 1.95-2.04 ns |
| **Weather** | Light attenuation | 730-783 ps |

### Animation Performance (astraweave-render)

| Benchmark | Result | Notes |
|-----------|--------|-------|
| vec3_lerp | 1.69-1.83 ns | 57% faster (v5.46) |
| quat_to_rotation | 1.63-1.73 ns | 36% faster (v5.46) |
| Tween update | 22.1 ns | — |
| Spring update | 14.2 ns | 1.6× faster than tween |

### Navigation Performance (astraweave-nav)

| Benchmark | Result | Notes |
|-----------|--------|-------|
| Sliver triangles | 99-104 ps | Sub-nanosecond! |
| Impossible paths (fast-fail) | 3.7-24.9 µs | — |
| Maze stress | 1.6-108 µs | — |
| Pathfind short | 7.5 µs | Excellent |

---

## Frame Budget Analysis

**Target**: 60 FPS = 16.67ms per frame

### Budget Breakdown (1,000 entities)

| System | Time | Budget % | Status |
|--------|------|----------|--------|
| ECS Core | 85 µs | 0.51% | ✅ |
| AI (500 agents) | 471 µs | 2.83% | ✅ |
| Physics (100 rigid bodies) | 47 µs | 0.28% | ✅ |
| Core game loop (5000 entities) | 529 µs | 3.17% | ✅ |
| **p50 Total** | 1.27 ms | 7.6% | ✅ |
| **p99 Total** | 2.42 ms | 14.5% | ✅ |
| **Headroom** | 14.25 ms | **85%** | ✅ |

### Scalability Projections

| Entity Count | p99 Estimate | Feasibility |
|--------------|--------------|-------------|
| 1,000 | 2.42 ms | ✅ 85% headroom |
| 5,000 | ~8-10 ms | ✅ 40-50% headroom |
| 10,000 | ~15-18 ms | ⚠️ Near budget |
| 20,000+ | >30 ms | ❌ Requires 30 FPS |

---

## Running Benchmarks

### Full Suite

```bash
# Run all Criterion benchmarks
cargo bench --workspace

# Run with odyssey automation (captures logs)
./scripts/benchmark_odyssey.ps1 -OutDir benchmark_results/$(Get-Date -Format 'yyyy-MM-dd')
```

### Per-Crate

```bash
# ECS benchmarks
cargo bench -p astraweave-ecs

# AI benchmarks
cargo bench -p astraweave-ai

# Physics benchmarks
cargo bench -p astraweave-physics

# Render benchmarks
cargo bench -p astraweave-render
```

### Generating HTML Reports

```bash
# Open Criterion HTML report
cargo bench -p astraweave-ecs -- --save-baseline main
# Reports at: target/criterion/*/report/index.html
```

---

## Benchmark Philosophy

AstraWeave treats benchmarks as **verification artifacts**, not marketing numbers:

1. **Reproducibility**: Every claimed measurement has a command that reproduces it
2. **Raw Logs**: All runs capture raw output for auditing
3. **Statistical Rigor**: Criterion.rs provides confidence intervals
4. **Adversarial Testing**: 22 adversarial benchmark sections stress edge cases
5. **Real Hardware**: No synthetic workloads—real game scenarios

See [Methodology](./methodology.md) for detailed measurement practices.

---

## See Also

- [Methodology](./methodology.md) - How benchmarks are measured
- [Optimization Guide](./optimization.md) - Performance improvement techniques
- [Performance Budgets](./budgets.md) - Frame budget allocation
- [Master Benchmark Report](../../masters/MASTER_BENCHMARK_REPORT.md) - Complete raw data
