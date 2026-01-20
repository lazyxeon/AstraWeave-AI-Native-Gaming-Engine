//! Benchmarks for astraweave-blend hashing operations.
//!
//! Tests content hashing performance across various data sizes
//! and usage patterns using SHA-256 (the actual hash algorithm used).

#![allow(unused_imports)]

use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::hint::black_box;
use sha2::{Digest, Sha256};

/// Compute SHA-256 hash of data (matches ConversionCache::hash_file)
fn compute_hash(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

// ============================================================================
// BASIC HASH BENCHMARKS
// ============================================================================

fn bench_hash_empty(c: &mut Criterion) {
    c.bench_function("hash_empty", |b| {
        b.iter(|| black_box(compute_hash(black_box(&[]))))
    });
}

fn bench_hash_small(c: &mut Criterion) {
    let data = b"small data";
    
    c.bench_function("hash_10_bytes", |b| {
        b.iter(|| black_box(compute_hash(black_box(data))))
    });
}

fn bench_hash_medium(c: &mut Criterion) {
    let data = vec![0u8; 1024];  // 1 KB
    
    c.bench_function("hash_1kb", |b| {
        b.iter(|| black_box(compute_hash(black_box(&data))))
    });
}

fn bench_hash_large(c: &mut Criterion) {
    let data = vec![0u8; 1024 * 1024];  // 1 MB
    
    c.bench_function("hash_1mb", |b| {
        b.iter(|| black_box(compute_hash(black_box(&data))))
    });
}

// ============================================================================
// SCALED SIZE BENCHMARKS
// ============================================================================

fn bench_hash_varied_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_varied_sizes");
    
    // Powers of 2 from 64 bytes to 16 MB
    let sizes: Vec<usize> = (6..=24).map(|p| 1 << p).collect();
    
    for size in sizes {
        let data = vec![0xABu8; size];
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} bytes", size)),
            &data,
            |b, data| b.iter(|| black_box(compute_hash(black_box(data)))),
        );
    }
    
    group.finish();
}

fn bench_hash_realistic_file_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_realistic_files");
    
    // Typical .blend file sizes
    let sizes = vec![
        ("simple_100kb", 100 * 1024),
        ("character_1mb", 1024 * 1024),
        ("scene_10mb", 10 * 1024 * 1024),
        ("complex_50mb", 50 * 1024 * 1024),
    ];
    
    for (name, size) in sizes {
        let data = vec![0xCDu8; size];
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &data,
            |b, data| b.iter(|| black_box(compute_hash(black_box(data)))),
        );
    }
    
    group.finish();
}

// ============================================================================
// HASH COMPARISON BENCHMARKS
// ============================================================================

fn bench_hash_comparison_equal(c: &mut Criterion) {
    let data = b"test data for hashing";
    let hash1 = compute_hash(data);
    let hash2 = compute_hash(data);
    
    c.bench_function("hash_comparison_equal", |b| {
        b.iter(|| black_box(black_box(&hash1) == black_box(&hash2)))
    });
}

fn bench_hash_comparison_different(c: &mut Criterion) {
    let hash1 = compute_hash(b"data1");
    let hash2 = compute_hash(b"data2");
    
    c.bench_function("hash_comparison_different", |b| {
        b.iter(|| black_box(black_box(&hash1) == black_box(&hash2)))
    });
}

// ============================================================================
// HASH CLONE AND COPY BENCHMARKS
// ============================================================================

fn bench_hash_clone(c: &mut Criterion) {
    let hash = compute_hash(b"test data");
    
    c.bench_function("hash_clone", |b| {
        b.iter(|| black_box(black_box(&hash).clone()))
    });
}

fn bench_hash_to_string(c: &mut Criterion) {
    let hash = compute_hash(b"test data");
    
    c.bench_function("hash_to_string", |b| {
        b.iter(|| black_box(format!("{:?}", black_box(&hash))))
    });
}

// ============================================================================
// HASH SET/MAP BENCHMARKS
// ============================================================================

fn bench_hash_hashmap_insert(c: &mut Criterion) {
    use std::collections::HashMap;
    
    let mut group = c.benchmark_group("hash_hashmap_insert");
    
    for count in [100, 1000, 10000].iter() {
        let hashes: Vec<String> = (0..*count)
            .map(|i| compute_hash(format!("data_{}", i).as_bytes()))
            .collect();
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &hashes,
            |b, hashes| {
                b.iter(|| {
                    let mut map: HashMap<_, _> = HashMap::new();
                    for (i, hash) in black_box(hashes).iter().enumerate() {
                        map.insert(hash.clone(), i);
                    }
                    black_box(map)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_hash_hashmap_lookup(c: &mut Criterion) {
    use std::collections::HashMap;
    
    let mut group = c.benchmark_group("hash_hashmap_lookup");
    
    for count in [100, 1000, 10000].iter() {
        let hashes: Vec<String> = (0..*count)
            .map(|i| compute_hash(format!("data_{}", i).as_bytes()))
            .collect();
        
        let map: HashMap<_, _> = hashes.iter()
            .enumerate()
            .map(|(i, h)| (h.clone(), i))
            .collect();
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &(hashes, map),
            |b, (hashes, map)| {
                b.iter(|| {
                    let mut found = 0;
                    for hash in black_box(hashes) {
                        if map.get(hash).is_some() {
                            found += 1;
                        }
                    }
                    black_box(found)
                })
            },
        );
    }
    
    group.finish();
}

fn bench_hash_hashset_contains(c: &mut Criterion) {
    use std::collections::HashSet;
    
    let mut group = c.benchmark_group("hash_hashset_contains");
    
    for count in [100, 1000, 10000].iter() {
        let hashes: Vec<String> = (0..*count)
            .map(|i| compute_hash(format!("data_{}", i).as_bytes()))
            .collect();
        
        let set: HashSet<_> = hashes.iter().cloned().collect();
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &(hashes, set),
            |b, (hashes, set)| {
                b.iter(|| {
                    let mut found = 0;
                    for hash in black_box(hashes) {
                        if set.contains(hash) {
                            found += 1;
                        }
                    }
                    black_box(found)
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// INCREMENTAL HASHING BENCHMARKS
// ============================================================================

fn bench_hash_many_small(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_many_small");
    
    // Simulate hashing many small chunks (like file metadata)
    for count in [10, 100, 1000].iter() {
        let chunks: Vec<Vec<u8>> = (0..*count)
            .map(|i| format!("chunk_{:06}", i).into_bytes())
            .collect();
        
        group.throughput(Throughput::Elements(*count as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &chunks,
            |b, chunks| {
                b.iter(|| {
                    let hashes: Vec<_> = black_box(chunks)
                        .iter()
                        .map(|c| compute_hash(c))
                        .collect();
                    black_box(hashes)
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// INCREMENTAL HASHER BENCHMARKS
// ============================================================================

fn bench_incremental_hasher(c: &mut Criterion) {
    let mut group = c.benchmark_group("incremental_hasher");
    
    // Test incremental hashing (simulates streaming file reads)
    let chunk_sizes = vec![
        ("8kb_chunks", 8192, 128),     // 1MB total
        ("64kb_chunks", 65536, 16),    // 1MB total
        ("1mb_chunks", 1048576, 1),    // 1MB total
    ];
    
    for (name, chunk_size, num_chunks) in chunk_sizes {
        let chunk = vec![0xEFu8; chunk_size];
        
        group.throughput(Throughput::Bytes((chunk_size * num_chunks) as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(name),
            &(chunk, num_chunks),
            |b, (chunk, num_chunks)| {
                b.iter(|| {
                    let mut hasher = Sha256::new();
                    for _ in 0..*num_chunks {
                        hasher.update(black_box(chunk));
                    }
                    black_box(hex::encode(hasher.finalize()))
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// THROUGHPUT SUMMARY BENCHMARKS
// ============================================================================

fn bench_hash_throughput_mb_per_sec(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_throughput");
    group.sample_size(50);
    
    // Large data to get accurate throughput
    let sizes = vec![
        1024 * 1024,        // 1 MB
        10 * 1024 * 1024,   // 10 MB
        100 * 1024 * 1024,  // 100 MB
    ];
    
    for size in sizes {
        let data = vec![0xEFu8; size];
        
        group.throughput(Throughput::Bytes(size as u64));
        group.bench_with_input(
            BenchmarkId::from_parameter(format!("{} MB", size / (1024 * 1024))),
            &data,
            |b, data| b.iter(|| black_box(compute_hash(black_box(data)))),
        );
    }
    
    group.finish();
}

// ============================================================================
// COLLISION RESISTANCE CHECK
// ============================================================================

fn bench_hash_collision_check(c: &mut Criterion) {
    // Generate many hashes and check for collisions (should find none)
    let mut group = c.benchmark_group("hash_collision_check");
    
    for count in [1000, 10000].iter() {
        let hashes: Vec<String> = (0..*count)
            .map(|i| compute_hash(format!("unique_data_{}", i).as_bytes()))
            .collect();
        
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &hashes,
            |b, hashes| {
                b.iter(|| {
                    use std::collections::HashSet;
                    let set: HashSet<_> = black_box(hashes).iter().collect();
                    // All hashes should be unique
                    black_box(set.len() == hashes.len())
                })
            },
        );
    }
    
    group.finish();
}

// ============================================================================
// CRITERION GROUPS
// ============================================================================

criterion_group!(
    basic_benches,
    bench_hash_empty,
    bench_hash_small,
    bench_hash_medium,
    bench_hash_large,
);

criterion_group!(
    size_benches,
    bench_hash_varied_sizes,
    bench_hash_realistic_file_sizes,
);

criterion_group!(
    comparison_benches,
    bench_hash_comparison_equal,
    bench_hash_comparison_different,
);

criterion_group!(
    clone_benches,
    bench_hash_clone,
    bench_hash_to_string,
);

criterion_group!(
    collection_benches,
    bench_hash_hashmap_insert,
    bench_hash_hashmap_lookup,
    bench_hash_hashset_contains,
);

criterion_group!(
    incremental_benches,
    bench_hash_many_small,
    bench_incremental_hasher,
);

criterion_group!(
    throughput_benches,
    bench_hash_throughput_mb_per_sec,
);

criterion_group!(
    utility_benches,
    bench_hash_collision_check,
);

criterion_main!(
    basic_benches,
    size_benches,
    comparison_benches,
    clone_benches,
    collection_benches,
    incremental_benches,
    throughput_benches,
    utility_benches,
);
