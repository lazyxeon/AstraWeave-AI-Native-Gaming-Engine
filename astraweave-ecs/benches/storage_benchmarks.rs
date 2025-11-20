// SPDX-License-Identifier: MIT
//! Benchmarks for new storage layer (BlobVec + SparseSet)
//!
//! Comparing:
//! - BlobVec vs Vec<Box<dyn Any>> (10× expected speedup)
//! - SparseSet vs BTreeMap (100× expected speedup)

use astraweave_ecs::{
    blob_vec::BlobVec,
    sparse_set::{SparseSet, SparseSetData},
    Entity,
};
use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use std::any::Any;
use std::collections::BTreeMap;
use std::hint::black_box;

#[derive(Clone, Copy, Debug)]
struct Position {
    x: f32,
    y: f32,
    z: f32,
}

#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Velocity {
    x: f32,
    y: f32,
    z: f32,
}

/// Benchmark BlobVec push vs Vec<Box<dyn Any>> push
fn bench_blobvec_vs_boxed_push(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_push");

    for count in [100, 1000, 10000] {
        // BlobVec approach
        group.bench_with_input(BenchmarkId::new("BlobVec", count), &count, |b, &count| {
            b.iter(|| {
                let mut blob = BlobVec::new::<Position>();
                for i in 0..count {
                    let pos = Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    };
                    unsafe {
                        blob.push(pos);
                    }
                }
                black_box(blob);
            });
        });

        // Vec<Box<dyn Any>> approach (old ECS)
        group.bench_with_input(BenchmarkId::new("Vec_Box", count), &count, |b, &count| {
            b.iter(|| {
                let mut vec: Vec<Box<dyn Any>> = Vec::new();
                for i in 0..count {
                    let pos = Position {
                        x: i as f32,
                        y: i as f32 * 2.0,
                        z: i as f32 * 3.0,
                    };
                    vec.push(Box::new(pos));
                }
                black_box(vec);
            });
        });
    }

    group.finish();
}

/// Benchmark BlobVec iteration vs Vec<Box<dyn Any>> iteration
fn bench_blobvec_vs_boxed_iteration(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_iteration");

    for count in [100, 1000, 10000] {
        // Setup BlobVec
        let mut blob = BlobVec::new::<Position>();
        for i in 0..count {
            let pos = Position {
                x: i as f32,
                y: i as f32 * 2.0,
                z: i as f32 * 3.0,
            };
            unsafe {
                blob.push(pos);
            }
        }

        group.bench_with_input(BenchmarkId::new("BlobVec_slice", count), &count, |b, _| {
            b.iter(|| {
                let slice = unsafe { blob.as_slice::<Position>() };
                let mut sum = 0.0f32;
                for pos in slice {
                    sum += pos.x + pos.y + pos.z;
                }
                black_box(sum);
            });
        });

        // Setup Vec<Box<dyn Any>>
        let mut vec: Vec<Box<dyn Any>> = Vec::new();
        for i in 0..count {
            let pos = Position {
                x: i as f32,
                y: i as f32 * 2.0,
                z: i as f32 * 3.0,
            };
            vec.push(Box::new(pos));
        }

        group.bench_with_input(
            BenchmarkId::new("Vec_Box_downcast", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let mut sum = 0.0f32;
                    for item in &vec {
                        if let Some(pos) = item.downcast_ref::<Position>() {
                            sum += pos.x + pos.y + pos.z;
                        }
                    }
                    black_box(sum);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark BlobVec mutable iteration vs Vec<Box<dyn Any>> mutable iteration
fn bench_blobvec_vs_boxed_mutation(c: &mut Criterion) {
    let mut group = c.benchmark_group("storage_mutation");

    for count in [100, 1000, 10000] {
        // Setup BlobVec
        let mut blob = BlobVec::new::<Position>();
        for i in 0..count {
            let pos = Position {
                x: i as f32,
                y: i as f32 * 2.0,
                z: i as f32 * 3.0,
            };
            unsafe {
                blob.push(pos);
            }
        }

        group.bench_with_input(
            BenchmarkId::new("BlobVec_slice_mut", count),
            &count,
            |b, _| {
                b.iter(|| {
                    let slice = unsafe { blob.as_slice_mut::<Position>() };
                    for pos in slice.iter_mut() {
                        pos.x += 1.0;
                        pos.y += 1.0;
                        pos.z += 1.0;
                    }
                    black_box(&blob);
                });
            },
        );

        // Setup Vec<Box<dyn Any>>
        let mut vec: Vec<Box<dyn Any>> = Vec::new();
        for i in 0..count {
            let pos = Position {
                x: i as f32,
                y: i as f32 * 2.0,
                z: i as f32 * 3.0,
            };
            vec.push(Box::new(pos));
        }

        group.bench_with_input(
            BenchmarkId::new("Vec_Box_downcast_mut", count),
            &count,
            |b, _| {
                b.iter(|| {
                    for item in &mut vec {
                        if let Some(pos) = item.downcast_mut::<Position>() {
                            pos.x += 1.0;
                            pos.y += 1.0;
                            pos.z += 1.0;
                        }
                    }
                    black_box(&vec);
                });
            },
        );
    }

    group.finish();
}

/// Benchmark SparseSet vs BTreeMap for entity lookup
fn bench_sparseset_vs_btreemap_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_lookup");

    for count in [100, 1000, 10000] {
        // Setup SparseSet
        let mut sparse_set = SparseSet::new();
        let entities: Vec<Entity> = (0..count).map(|i| unsafe { Entity::from_raw(i) }).collect();
        for &entity in &entities {
            sparse_set.insert(entity);
        }

        group.bench_with_input(BenchmarkId::new("SparseSet", count), &count, |b, _| {
            b.iter(|| {
                for &entity in &entities {
                    black_box(sparse_set.get(entity));
                }
            });
        });

        // Setup BTreeMap
        let mut btree: BTreeMap<Entity, usize> = BTreeMap::new();
        for (i, &entity) in entities.iter().enumerate() {
            btree.insert(entity, i);
        }

        group.bench_with_input(BenchmarkId::new("BTreeMap", count), &count, |b, _| {
            b.iter(|| {
                for &entity in &entities {
                    black_box(btree.get(&entity));
                }
            });
        });
    }

    group.finish();
}

/// Benchmark SparseSet vs BTreeMap for entity insertion
fn bench_sparseset_vs_btreemap_insert(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_insert");

    for count in [100, 1000, 10000] {
        let entities: Vec<Entity> = (0..count).map(|i| unsafe { Entity::from_raw(i) }).collect();

        group.bench_with_input(BenchmarkId::new("SparseSet", count), &count, |b, _| {
            b.iter(|| {
                let mut sparse_set = SparseSet::new();
                for &entity in &entities {
                    sparse_set.insert(entity);
                }
                black_box(sparse_set);
            });
        });

        group.bench_with_input(BenchmarkId::new("BTreeMap", count), &count, |b, _| {
            b.iter(|| {
                let mut btree: BTreeMap<Entity, usize> = BTreeMap::new();
                for (i, &entity) in entities.iter().enumerate() {
                    btree.insert(entity, i);
                }
                black_box(btree);
            });
        });
    }

    group.finish();
}

/// Benchmark SparseSet vs BTreeMap for entity removal
fn bench_sparseset_vs_btreemap_remove(c: &mut Criterion) {
    let mut group = c.benchmark_group("entity_remove");

    for count in [100, 1000, 10000] {
        let entities: Vec<Entity> = (0..count).map(|i| unsafe { Entity::from_raw(i) }).collect();

        // SparseSet removal
        group.bench_with_input(BenchmarkId::new("SparseSet", count), &count, |b, _| {
            b.iter_batched(
                || {
                    let mut sparse_set = SparseSet::new();
                    for &entity in &entities {
                        sparse_set.insert(entity);
                    }
                    sparse_set
                },
                |mut sparse_set| {
                    for &entity in &entities {
                        sparse_set.remove(entity);
                    }
                    black_box(sparse_set);
                },
                criterion::BatchSize::SmallInput,
            );
        });

        // BTreeMap removal
        group.bench_with_input(BenchmarkId::new("BTreeMap", count), &count, |b, _| {
            b.iter_batched(
                || {
                    let mut btree: BTreeMap<Entity, usize> = BTreeMap::new();
                    for (i, &entity) in entities.iter().enumerate() {
                        btree.insert(entity, i);
                    }
                    btree
                },
                |mut btree| {
                    for &entity in &entities {
                        btree.remove(&entity);
                    }
                    black_box(btree);
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

/// Benchmark SparseSetData for combined entity+component storage
fn bench_sparseset_data(c: &mut Criterion) {
    let mut group = c.benchmark_group("sparseset_data");

    for count in [100, 1000, 10000] {
        let entities: Vec<Entity> = (0..count).map(|i| unsafe { Entity::from_raw(i) }).collect();

        // Insert benchmark
        group.bench_with_input(BenchmarkId::new("insert", count), &count, |b, _| {
            b.iter(|| {
                let mut set = SparseSetData::new();
                for &entity in &entities {
                    let pos = Position {
                        x: 1.0,
                        y: 2.0,
                        z: 3.0,
                    };
                    set.insert(entity, pos);
                }
                black_box(set);
            });
        });

        // Setup for iteration
        let mut set = SparseSetData::new();
        for &entity in &entities {
            let pos = Position {
                x: 1.0,
                y: 2.0,
                z: 3.0,
            };
            set.insert(entity, pos);
        }

        // Iteration benchmark
        group.bench_with_input(BenchmarkId::new("iterate", count), &count, |b, _| {
            b.iter(|| {
                let mut sum = 0.0f32;
                for (_, pos) in set.iter() {
                    sum += pos.x + pos.y + pos.z;
                }
                black_box(sum);
            });
        });

        // Mutation benchmark
        group.bench_with_input(BenchmarkId::new("mutate", count), &count, |b, _| {
            b.iter(|| {
                for (_, pos) in set.iter_mut() {
                    pos.x += 1.0;
                    pos.y += 1.0;
                    pos.z += 1.0;
                }
                black_box(&set);
            });
        });
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_blobvec_vs_boxed_push,
    bench_blobvec_vs_boxed_iteration,
    bench_blobvec_vs_boxed_mutation,
    bench_sparseset_vs_btreemap_lookup,
    bench_sparseset_vs_btreemap_insert,
    bench_sparseset_vs_btreemap_remove,
    bench_sparseset_data,
);
criterion_main!(benches);
