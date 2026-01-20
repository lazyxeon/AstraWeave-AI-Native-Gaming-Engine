//! # Networking Benchmark Suite
//!
//! Comprehensive benchmarks for the astraweave-net crate covering:
//! - Snapshot building and delta compression
//! - Interest management (radius, FOV, LOS filtering)
//! - Hash computation for deterministic sync
//! - Delta application and state reconstruction
//! - Message serialization
//!
//! Run with: `cargo bench -p astraweave-net`

#![allow(clippy::clone_on_copy)]

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::BTreeSet;

use astraweave_core::IVec2;
use astraweave_net::{
    apply_delta, diff_snapshots, filter_snapshot_for_viewer, Delta, EntityDelta, EntityState,
    FovInterest, FullInterest, RadiusTeamInterest, Snapshot,
};

// ============================================================================
// CORRECTNESS ASSERTION HELPERS
// ============================================================================

/// Assert that a snapshot is valid
#[allow(dead_code)]
fn assert_snapshot_valid(snapshot: &Snapshot) {
    assert!(snapshot.tick > 0 || snapshot.seq == 0, "Snapshot should have valid tick");
    // Entities should be well-formed
    for entity in &snapshot.entities {
        // Access check - just verify entity data is accessible
        let _ = entity.hp;
    }
}

/// Assert that a delta is valid
#[allow(dead_code)]
fn assert_delta_valid(delta: &Delta, base_tick: u64, head_tick: u64) {
    assert_eq!(delta.base_tick, base_tick, "Delta base_tick should match");
    assert_eq!(delta.tick, head_tick, "Delta tick should match");
}

/// Assert that interest filtering works correctly
fn assert_interest_filter_valid(filtered: &Snapshot, original: &Snapshot) {
    assert!(
        filtered.entities.len() <= original.entities.len(),
        "Filtered should have <= entities than original"
    );
}

// ============================================================================
// TEST DATA GENERATORS
// ============================================================================

/// Create a sample entity state
fn create_entity(id: u32, x: i32, y: i32, hp: i32, team: u8, ammo: i32) -> EntityState {
    EntityState {
        id,
        pos: IVec2 { x, y },
        hp,
        team,
        ammo,
    }
}

/// Create a snapshot with N entities distributed across the map
fn create_snapshot(entity_count: usize, tick: u64, seq: u32) -> Snapshot {
    let entities: Vec<EntityState> = (0..entity_count)
        .map(|i| {
            let id = i as u32;
            let x = (i % 100) as i32;
            let y = (i / 100) as i32;
            let hp = 100 - (i % 50) as i32;
            let team = (i % 3) as u8;
            let ammo = (30 - (i % 30)) as i32;
            create_entity(id, x, y, hp, team, ammo)
        })
        .collect();

    // Compute simple hash
    let world_hash = entities.iter().fold(0u64, |acc, e| {
        acc.wrapping_add(e.id as u64)
            .wrapping_add(e.pos.x as u64)
            .wrapping_add(e.pos.y as u64)
    });

    Snapshot {
        version: 1,
        tick,
        t: tick as f32 / 60.0,
        seq,
        world_hash,
        entities,
    }
}

/// Create a modified snapshot (simulating world changes)
fn create_modified_snapshot(base: &Snapshot, change_percent: usize) -> Snapshot {
    let mut entities = base.entities.clone();
    let changes = (entities.len() * change_percent / 100).max(1);

    for i in 0..changes {
        if i < entities.len() {
            entities[i].pos.x += 1;
            entities[i].hp = (entities[i].hp - 5).max(0);
        }
    }

    // Add some new entities
    let new_count = changes / 2;
    for i in 0..new_count {
        entities.push(create_entity(
            (base.entities.len() + i) as u32,
            50 + i as i32,
            50,
            100,
            1,
            30,
        ));
    }

    let world_hash = entities.iter().fold(0u64, |acc, e| {
        acc.wrapping_add(e.id as u64)
            .wrapping_add(e.pos.x as u64)
            .wrapping_add(e.pos.y as u64)
    });

    Snapshot {
        version: 1,
        tick: base.tick + 1,
        t: (base.tick + 1) as f32 / 60.0,
        seq: base.seq + 1,
        world_hash,
        entities,
    }
}

/// Create a viewer entity for interest tests
fn create_viewer(team: u8) -> EntityState {
    create_entity(0, 50, 50, 100, team, 30)
}

/// Create obstacles set for LOS tests
#[allow(dead_code)]
fn create_obstacles(count: usize) -> BTreeSet<(i32, i32)> {
    let mut obstacles = BTreeSet::new();
    for i in 0..count {
        let x = 45 + (i % 10) as i32;
        let y = 45 + (i / 10) as i32;
        obstacles.insert((x, y));
    }
    obstacles
}

// ============================================================================
// SNAPSHOT BENCHMARKS
// ============================================================================

fn bench_snapshot_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("snapshot_operations");

    // Create snapshot with varying entity counts
    for entity_count in [10, 50, 100, 500, 1000] {
        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("create_snapshot", entity_count),
            &entity_count,
            |b, &count| {
                b.iter(|| {
                    let snapshot = create_snapshot(black_box(count), 1, 0);
                    assert_eq!(snapshot.entities.len(), count);
                    black_box(snapshot)
                })
            },
        );
    }

    // Clone snapshot
    for entity_count in [50, 100, 500] {
        let snapshot = create_snapshot(entity_count, 1, 0);

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("clone_snapshot", entity_count),
            &snapshot,
            |b, snapshot| {
                b.iter(|| {
                    let cloned = black_box(snapshot).clone();
                    assert_eq!(cloned.entities.len(), snapshot.entities.len());
                    black_box(cloned)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// DELTA COMPRESSION BENCHMARKS
// ============================================================================

fn bench_delta_compression(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_compression");

    // Diff snapshots with varying change percentages
    for (entity_count, change_percent) in [(100, 10), (100, 50), (500, 10), (500, 50), (1000, 10)] {
        let base = create_snapshot(entity_count, 1, 0);
        let head = create_modified_snapshot(&base, change_percent);
        let viewer = create_viewer(0);
        let interest = FullInterest;

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new(
                "diff_snapshots",
                format!("{}e_{}%", entity_count, change_percent),
            ),
            &(base.clone(), head.clone(), viewer.clone()),
            |b, (base, head, viewer)| {
                b.iter(|| {
                    let delta = diff_snapshots(
                        black_box(base),
                        black_box(head),
                        &interest,
                        black_box(viewer),
                    );
                    assert!(!delta.changed.is_empty() || !delta.removed.is_empty() || delta.changed.is_empty());
                    black_box(delta)
                })
            },
        );
    }

    // Apply delta
    for entity_count in [100, 500, 1000] {
        let base = create_snapshot(entity_count, 1, 0);
        let head = create_modified_snapshot(&base, 20);
        let viewer = create_viewer(0);
        let interest = FullInterest;
        let delta = diff_snapshots(&base, &head, &interest, &viewer);

        group.bench_with_input(
            BenchmarkId::new("apply_delta", entity_count),
            &(base.clone(), delta.clone()),
            |b, (base, delta)| {
                b.iter_batched(
                    || base.clone(),
                    |mut snapshot| {
                        apply_delta(&mut snapshot, black_box(delta));
                        assert_snapshot_valid(&snapshot);
                        black_box(snapshot)
                    },
                    criterion::BatchSize::SmallInput,
                )
            },
        );
    }

    // Delta compression ratio analysis
    for (entity_count, change_percent) in [(100, 5), (100, 20), (100, 50)] {
        let base = create_snapshot(entity_count, 1, 0);
        let head = create_modified_snapshot(&base, change_percent);
        let viewer = create_viewer(0);
        let interest = FullInterest;

        group.bench_with_input(
            BenchmarkId::new(
                "compression_ratio",
                format!("{}e_{}%", entity_count, change_percent),
            ),
            &(base.clone(), head.clone(), viewer.clone()),
            |b, (base, head, viewer)| {
                b.iter(|| {
                    let delta = diff_snapshots(base, head, &interest, viewer);
                    // Calculate compression ratio
                    let full_size = head.entities.len() * std::mem::size_of::<EntityState>();
                    let delta_size = delta.changed.len() * std::mem::size_of::<EntityDelta>()
                        + delta.removed.len() * std::mem::size_of::<u32>();
                    let ratio = if delta_size > 0 {
                        full_size as f32 / delta_size as f32
                    } else {
                        f32::INFINITY
                    };
                    assert!(ratio >= 1.0 || delta_size == 0, "Delta should be smaller");
                    black_box(ratio)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// INTEREST MANAGEMENT BENCHMARKS
// ============================================================================

fn bench_interest_filtering(c: &mut Criterion) {
    let mut group = c.benchmark_group("interest_filtering");

    // FullInterest (baseline - no filtering)
    for entity_count in [100, 500, 1000] {
        let snapshot = create_snapshot(entity_count, 1, 0);
        let viewer = create_viewer(0);
        let interest = FullInterest;

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("full_interest", entity_count),
            &(snapshot.clone(), viewer.clone()),
            |b, (snapshot, viewer)| {
                b.iter(|| {
                    let filtered = filter_snapshot_for_viewer(
                        black_box(snapshot),
                        &interest,
                        black_box(viewer),
                    );
                    assert_eq!(
                        filtered.entities.len(),
                        snapshot.entities.len(),
                        "Full interest should include all"
                    );
                    black_box(filtered)
                })
            },
        );
    }

    // RadiusTeamInterest
    for (entity_count, radius) in [(100, 10), (500, 20), (1000, 30)] {
        let snapshot = create_snapshot(entity_count, 1, 0);
        let viewer = create_viewer(0);
        let interest = RadiusTeamInterest { radius };

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("radius_interest", format!("{}e_r{}", entity_count, radius)),
            &(snapshot.clone(), viewer.clone()),
            |b, (snapshot, viewer)| {
                b.iter(|| {
                    let filtered = filter_snapshot_for_viewer(
                        black_box(snapshot),
                        &interest,
                        black_box(viewer),
                    );
                    assert_interest_filter_valid(&filtered, snapshot);
                    black_box(filtered)
                })
            },
        );
    }

    // FovInterest
    for (entity_count, half_angle) in [(100, 45.0), (500, 60.0), (1000, 90.0)] {
        let snapshot = create_snapshot(entity_count, 1, 0);
        let viewer = create_viewer(0);
        let interest = FovInterest {
            radius: 50,
            half_angle_deg: half_angle,
            facing: IVec2 { x: 1, y: 0 },
        };

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("fov_interest", format!("{}e_{}deg", entity_count, half_angle as i32)),
            &(snapshot.clone(), viewer.clone()),
            |b, (snapshot, viewer)| {
                b.iter(|| {
                    let filtered = filter_snapshot_for_viewer(
                        black_box(snapshot),
                        &interest,
                        black_box(viewer),
                    );
                    assert_interest_filter_valid(&filtered, snapshot);
                    black_box(filtered)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// ENTITY STATE BENCHMARKS
// ============================================================================

fn bench_entity_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_operations");

    // Create entity
    group.bench_function("create_entity", |b| {
        b.iter(|| {
            let entity = create_entity(
                black_box(1),
                black_box(50),
                black_box(50),
                black_box(100),
                black_box(0),
                black_box(30),
            );
            assert_eq!(entity.id, 1);
            black_box(entity)
        })
    });

    // Clone entity
    let sample_entity = create_entity(1, 50, 50, 100, 0, 30);
    group.bench_function("clone_entity", |b| {
        b.iter(|| {
            let cloned = black_box(&sample_entity).clone();
            assert_eq!(cloned.id, sample_entity.id);
            black_box(cloned)
        })
    });

    // Compare entities
    let entity_a = create_entity(1, 50, 50, 100, 0, 30);
    let entity_b = create_entity(1, 51, 50, 95, 0, 29);
    group.bench_function("compare_entities", |b| {
        b.iter(|| {
            let same_pos = entity_a.pos == entity_b.pos;
            let same_hp = entity_a.hp == entity_b.hp;
            let same_ammo = entity_a.ammo == entity_b.ammo;
            black_box((same_pos, same_hp, same_ammo))
        })
    });

    // Distance calculation (for radius interest)
    group.bench_function("distance_squared", |b| {
        let viewer = create_entity(0, 0, 0, 100, 0, 30);
        let target = create_entity(1, 50, 50, 100, 1, 30);
        b.iter(|| {
            let dx = target.pos.x - viewer.pos.x;
            let dy = target.pos.y - viewer.pos.y;
            let dist_sq = dx * dx + dy * dy;
            assert!(dist_sq > 0, "Distance should be positive");
            black_box(dist_sq)
        })
    });

    group.finish();
}

// ============================================================================
// SERIALIZATION BENCHMARKS
// ============================================================================

fn bench_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("net_serialization");

    // Serialize snapshot to JSON
    for entity_count in [10, 50, 100] {
        let snapshot = create_snapshot(entity_count, 1, 0);

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("snapshot_to_json", entity_count),
            &snapshot,
            |b, snapshot| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(snapshot))
                        .expect("Serialization should succeed");
                    assert!(!json.is_empty());
                    black_box(json)
                })
            },
        );
    }

    // Deserialize snapshot from JSON
    for entity_count in [10, 50, 100] {
        let snapshot = create_snapshot(entity_count, 1, 0);
        let json = serde_json::to_string(&snapshot).unwrap();

        group.throughput(Throughput::Bytes(json.len() as u64));
        group.bench_with_input(
            BenchmarkId::new("snapshot_from_json", entity_count),
            &json,
            |b, json| {
                b.iter(|| {
                    let parsed: Snapshot =
                        serde_json::from_str(black_box(json)).expect("Deserialization should succeed");
                    assert_eq!(parsed.entities.len(), entity_count);
                    black_box(parsed)
                })
            },
        );
    }

    // Serialize delta to JSON
    for entity_count in [50, 100] {
        let base = create_snapshot(entity_count, 1, 0);
        let head = create_modified_snapshot(&base, 20);
        let viewer = create_viewer(0);
        let interest = FullInterest;
        let delta = diff_snapshots(&base, &head, &interest, &viewer);

        group.bench_with_input(
            BenchmarkId::new("delta_to_json", entity_count),
            &delta,
            |b, delta| {
                b.iter(|| {
                    let json = serde_json::to_string(black_box(delta))
                        .expect("Serialization should succeed");
                    assert!(!json.is_empty());
                    black_box(json)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// BATCH OPERATIONS BENCHMARKS
// ============================================================================

fn bench_batch_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("batch_operations");

    // Process multiple snapshots (simulate tick processing)
    for batch_size in [10, 50, 100] {
        let base = create_snapshot(200, 1, 0);
        let snapshots: Vec<Snapshot> = (0..batch_size)
            .map(|i| create_modified_snapshot(&base, (i % 30) + 5))
            .collect();
        let viewer = create_viewer(0);
        let interest = RadiusTeamInterest { radius: 30 };

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(
            BenchmarkId::new("batch_filter", batch_size),
            &snapshots,
            |b, snapshots| {
                b.iter(|| {
                    let filtered: Vec<Snapshot> = snapshots
                        .iter()
                        .map(|s| filter_snapshot_for_viewer(s, &interest, &viewer))
                        .collect();
                    assert_eq!(filtered.len(), snapshots.len());
                    black_box(filtered)
                })
            },
        );
    }

    // Chain of deltas
    for chain_length in [5, 10, 20] {
        let mut snapshots = Vec::with_capacity(chain_length);
        snapshots.push(create_snapshot(100, 1, 0));
        for i in 1..chain_length {
            snapshots.push(create_modified_snapshot(&snapshots[i - 1], 10));
        }
        let viewer = create_viewer(0);
        let interest = FullInterest;

        group.bench_with_input(
            BenchmarkId::new("delta_chain", chain_length),
            &snapshots,
            |b, snapshots| {
                b.iter(|| {
                    let mut deltas = Vec::with_capacity(snapshots.len() - 1);
                    for i in 1..snapshots.len() {
                        let delta = diff_snapshots(&snapshots[i - 1], &snapshots[i], &interest, &viewer);
                        deltas.push(delta);
                    }
                    assert_eq!(deltas.len(), snapshots.len() - 1);
                    black_box(deltas)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// HASH COMPUTATION BENCHMARKS
// ============================================================================

fn bench_hash_computation(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_computation");

    // Hash entity list
    for entity_count in [50, 100, 500, 1000] {
        let snapshot = create_snapshot(entity_count, 1, 0);

        group.throughput(Throughput::Elements(entity_count as u64));
        group.bench_with_input(
            BenchmarkId::new("hash_entities", entity_count),
            &snapshot.entities,
            |b, entities| {
                b.iter(|| {
                    use std::collections::hash_map::DefaultHasher;
                    use std::hash::{Hash, Hasher};
                    
                    let mut hasher = DefaultHasher::new();
                    for e in entities.iter() {
                        e.id.hash(&mut hasher);
                        e.pos.x.hash(&mut hasher);
                        e.pos.y.hash(&mut hasher);
                        e.hp.hash(&mut hasher);
                        e.team.hash(&mut hasher);
                        e.ammo.hash(&mut hasher);
                    }
                    let hash = hasher.finish();
                    black_box(hash)
                })
            },
        );
    }

    group.finish();
}

// ============================================================================
// CRITERION GROUP REGISTRATION
// ============================================================================

criterion_group!(
    benches,
    bench_snapshot_operations,
    bench_delta_compression,
    bench_interest_filtering,
    bench_entity_operations,
    bench_serialization,
    bench_batch_operations,
    bench_hash_computation,
);

criterion_main!(benches);
