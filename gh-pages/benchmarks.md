---
layout: default
title: Performance Benchmarks
---

# Performance Benchmarks

All benchmarks run on production hardware using Criterion.rs. Values represent median latencies.

## ECS Performance

| Operation | Latency | Throughput |
|-----------|---------|------------|
| World creation | 25.8 ns | 38.8M/sec |
| Entity spawn | 420 ns | 2.38M/sec |
| Per-entity tick | <1 ns | >1B/sec |
| Component add | ~100 ns | 10M/sec |

## AI Planning

| Mode | Latency | Capacity @ 60 FPS |
|------|---------|-------------------|
| Behavior tree tick | 57–253 ns | 66,000 agents |
| GOAP (cache hit) | 1.01 µs | 16,200 agents |
| GOAP (cache miss) | 47.2 µs | 340 agents |
| Arbiter GOAP control | 101.7 ns | 160,000 agents |
| Arbiter LLM polling | 575.3 ns | 28,000 agents |
| Arbiter mode transition | 221.9 ns | 73,000 agents |
| Full arbiter cycle | 313.7 ns | 51,000 agents |

**Validated**: 12,700+ agents running full AI at 60 FPS. 6.48M anti-cheat validation checks/sec.

## Physics

| Operation | Latency |
|-----------|---------|
| Character move | 114 ns |
| Rigid body step | 2.97 µs |
| Full physics tick | 6.52 µs |
| Spatial hash (FxHashMap) | 3.77 ms |
| Collision pair reduction | 99.96% |

## Rendering

| Metric | Value |
|--------|-------|
| Frame time (1K entities) | 2.70 ms |
| FPS (1K entities) | 370 |
| Budget headroom vs 60 FPS | 84% |
| Vertex compression | 37.5% memory reduction |
| Instancing | 10–100× draw call reduction |

## SIMD Math

| Operation | Scalar | SIMD | Speedup |
|-----------|--------|------|---------|
| Vec3 dot product | — | — | 2.1× |
| Mat4 multiply | — | — | 2.5× |
| Quat slerp | — | — | 1.75× |
| Batch movement (10K) | 20.6 µs | 9.9 µs | 2.08× |

## Fluid Simulation

| Test | Coverage |
|------|----------|
| Total tests | 2,404 |
| SPH kernel functions | ✅ |
| Pressure solver | ✅ |
| Viscosity | ✅ |
| Surface tension | ✅ |
| Boundary conditions | ✅ |

## 60 FPS Budget Analysis

At 60 FPS, each frame has 16.67 ms of budget:

| System | Time | Budget % |
|--------|------|----------|
| Physics (full tick) | 6.52 µs | 0.04% |
| AI (12,700 agents) | ~4 ms | 24% |
| Rendering (1K entities) | 2.70 ms | 16% |
| **Total** | **~6.7 ms** | **40%** |
| **Remaining headroom** | **~10 ms** | **60%** |

[← Back to Home](index.html)
